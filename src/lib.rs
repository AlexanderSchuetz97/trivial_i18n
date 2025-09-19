use crate::x::I18NFormatParameter;
use linked_hash_map::LinkedHashMap;
use proc_macro::token_stream::IntoIter;
use proc_macro::{TokenStream, TokenTree};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Display, Write};
use std::fs::File;
use std::io::BufReader;
use std::mem;
use std::ops::{Deref, Index};
use std::path::Path;

#[derive(Debug, Clone)]
struct Variant {
    name: String,
    path: String,
    fallbacks: Vec<String>,
    properties: HashMap<String, String>,
    complex_properties: HashMap<String, Vec<(String, usize)>>
}

fn parse_path(input: &mut IntoIter) -> String {
    let mut language_name = String::new();

    while let Some(next) = input.next() {
        match next {
            TokenTree::Group(_) => {
                panic!(
                    "Trying to parse language enum name, got TokenTree::Group which cant possibly be contained in a valid rust type path."
                );
            }
            TokenTree::Punct(png) => {
                if png.as_char() == ';' {
                    break;
                }
                language_name.push(png.as_char());
            }
            other => {
                language_name.push_str(other.to_string().as_str());
            }
        }
    }

    language_name
}

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {

    let mut token_iter : IntoIter = input.into_iter();

    let mut language_name = parse_path(&mut token_iter);

    if language_name.parse::<u128>().is_ok() {
        language_name = parse_path(&mut token_iter);
    }

    if language_name.is_empty() {
        panic!("Trying to parse language name but no language name supplied.");
    }

    let Some(TokenTree::Ident(lit)) = token_iter.next() else {
        panic!("Trying to parse language default enum name, a ident, but got non ident.");
    };

    let default_variant = lit.to_string();

    let Some(TokenTree::Punct(p)) = token_iter.next() else {
        panic!("Trying to parse = after default language enum name, but got non Punct TokenTree");
    };

    if p.as_char() != '=' {
        panic!(
            "Trying to parse = after default language enum name, but got {}",
            p.as_char()
        );
    }

    if default_variant.is_empty() {
        panic!("Trying to parse language default variant but got empty token tree.");
    }

    let Some(TokenTree::Literal(lit)) = token_iter.next() else {
        panic!("Trying to parse language default file path, a literal, but got non literal.");
    };

    let Some(TokenTree::Punct(p)) = token_iter.next() else {
        panic!("Trying to parse ; after language default file path, but got non Punct TokenTree");
    };

    if p.as_char() != ';' {
        panic!(
            "Trying to parse ; after language default file path, but got {}",
            p.as_char()
        );
    }

    let default_path = lit.to_string();

    let mut variants = LinkedHashMap::new();

    variants.insert(
        default_variant.clone(),
        Variant {
            name: default_variant.clone(),
            path: default_path,
            fallbacks: vec![],
            properties: Default::default(),
            complex_properties: Default::default(),
        },
    );

    while let Some(next) = token_iter.next() {
        let TokenTree::Ident(lit) = next else {
            panic!("Trying to parse language enum name, a Ident, but got non Ident TokenTree.");
        };

        let variant_name = lit.to_string();

        let Some(TokenTree::Punct(p)) = token_iter.next() else {
            panic!(
                "Trying to parse = after language enum name {language_name}, but got non Punct TokenTree"
            );
        };

        if p.as_char() != '=' {
            panic!(
                "Trying to parse = after language enum name {language_name}, but got {}",
                p.as_char()
            );
        }

        let Some(TokenTree::Literal(lit)) = token_iter.next() else {
            panic!(
                "Trying to parse language file path of language {language_name}, a literal, but got non literal."
            );
        };

        let variant_path = lit.to_string();
        let mut fallbacks = Vec::new();

        loop {
            let Some(TokenTree::Punct(p)) = token_iter.next() else {
                panic!(
                    "Trying to parse ; after language {variant_name} file path {variant_path}, but got non Punct TokenTree"
                );
            };

            if p.as_char() == ';' {
                break;
            }

            if p.as_char() != ',' {
                panic!(
                    "Trying to parse ; or , after language {variant_name} file path {variant_path}, but got {}",
                    p.as_char()
                );
            }

            match token_iter.next() {
                Some(TokenTree::Ident(lit)) => {
                    fallbacks.push(lit.to_string());
                }
                _ => {
                    panic!(
                        "Trying to parse fallback language name for language {language_name}, a Ident, but got non Ident TokenTree."
                    );
                }
            }
        }

        variants.insert(
            variant_name.clone(),
            Variant {
                name: variant_name,
                path: variant_path,
                fallbacks,
                properties: Default::default(),
                complex_properties: Default::default(),
            },
        );
    }

    for (_, variant) in variants.iter_mut() {
        let path = Path::new(&variant.path[1..variant.path.len() - 1]);
        let mut prop_file_reader = BufReader::new(
            File::open(path).expect(
                format!(
                    "Failed to open file {} for language {}",
                    variant.path, variant.name
                )
                .as_str(),
            ),
        );

        variant.properties = match jprop::parse_utf8_to_map(&mut prop_file_reader) {
            Ok(props) => props,
            Err(e) => panic!("Failed to parse .properties file: {}, {}", variant.path, e),
        }
    }

    validate_fallbacks_exist(&mut variants);
    validate_all_keys_in_default_language(&default_variant, &mut variants);
    resolve_fallbacks_properties(&default_variant, &mut variants);
    sort_properties(&mut variants);
    let complexity = find_max_complexity(&variants);
    let all_complexity = find_all_complexity(&variants);

    let mut output = String::with_capacity(0x4_00_00);
    output.push_str("static SELECTION: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);\n");

    output.push_str("pub trait I18NFormatParameter<const MAX_INDEX: usize> {\n");
    output.push_str("fn format_parameter(&self, idx: usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;\n");
    output.push_str("}\n");

    output.push_str("impl I18NFormatParameter<0> for () {\n");
    output.push_str("fn format_parameter(&self, idx: usize, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("Ok(())\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl<const MAX_INDEX: usize, T: core::fmt::Display> I18NFormatParameter<MAX_INDEX> for &[T] {\n");
    output.push_str("fn format_parameter(&self, idx: usize, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("let Some(dsp) = self.get(idx) else {\n");
    output.push_str("return Ok(());\n");
    output.push_str("};\n");
    output.push_str("core::fmt::Display::fmt(dsp, f)\n");
    output.push_str("}\n");
    output.push_str("}\n");

    for k in all_complexity {
        if k == 0 {
            continue;
        }
        fn mk_arg_impl(output: &mut String, k: usize, prefix: &str) {
            output.push_str("impl<");

            for n in 0..k {
                output.push_str(&format!("D{}: core::fmt::Display, ", n));
            }

            output.push_str("> I18NFormatParameter<");
            output.push_str(k.to_string().as_str());
            output.push_str("> for ");
            output.push_str(prefix);
            output.push_str("(");

            for n in 0..k {
                output.push_str(&format!("D{}, ", n));
            }
            output.push_str(") {\n");
            output.push_str("fn format_parameter(&self, idx: usize, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            output.push_str("match(idx) {\n");
            for n in 0..k {
                output.push_str(n.to_string().as_str());
                output.push_str(" => core::fmt::Display::fmt(&self.");
                output.push_str(n.to_string().as_str());
                output.push_str(", f),");
            }
            output.push_str("_=> Ok(())\n");
            output.push_str("}\n");
            output.push_str("}\n");
            output.push_str("}\n");
        }

        mk_arg_impl(&mut output, k, "");
        mk_arg_impl(&mut output, k, "&");
    }

    output.push_str("#[derive(Debug, Copy, Clone)]\n");
    output.push_str(
        format!(
            "pub struct I18NValue<const MAX_INDEX: usize>(&'static [(&'static str, &'static [(&'static str, usize)]); {}]);\n",
            variants.len()
        )
        .as_str(),
    );
    output.push_str("impl<const MAX_INDEX: usize> I18NValue<MAX_INDEX> {\n");
    output.push_str("pub fn as_str(&self) -> &'static str {\n");
    output.push_str("self.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize].0\n");
    output.push_str("}\n");
    output.push_str("pub const fn default_str(&self) -> &'static str {\n");
    output.push_str("self.0[0].0\n");
    output.push_str("}\n");

    output.push_str("pub fn format_with<T: >(&self, arg: impl I18NFormatParameter<MAX_INDEX>, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {\n");
    output.push_str("for (prefix, arg_index) in self.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize].1 {\n");
    output.push_str("let idx = *arg_index;\n");
    output.push_str("f.write_str(prefix)?;\n");
    output.push_str("if idx != usize::MAX {\n");
    output.push_str("arg.format_parameter(idx, f)?;\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str("Ok(())\n");
    output.push_str("}\n");

    output.push_str("pub fn format(&self, arg: impl I18NFormatParameter<MAX_INDEX>) -> String {\n");
    output.push_str("struct FMT<'a, const M: usize, T: I18NFormatParameter<M>>(&'a I18NValue<M>, T);\n");
    output.push_str("impl<const M: usize, T: I18NFormatParameter<M>> core::fmt::Display for FMT<'_, M, T> {\n");
    output.push_str("fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("for (prefix, arg_index) in self.0.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize].1 {\n");
    output.push_str("let idx = *arg_index;\n");
    output.push_str("f.write_str(prefix)?;\n");
    output.push_str("if idx != usize::MAX {\n");
    output.push_str("self.1.format_parameter(idx, f)?;\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str("Ok(())\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str("let formatter = FMT(self, arg);\n");
    output.push_str("ToString::to_string(&formatter)\n");
    output.push_str("}\n");


    output.push_str("}\n");

    output.push_str("impl<const MAX_INDEX: usize> AsRef<str> for I18NValue<MAX_INDEX> {\n");
    output.push_str("fn as_ref(&self) -> &str {\n");
    output.push_str("self.as_str()\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl<const MAX_INDEX: usize> From<I18NValue<MAX_INDEX>> for String {\n");
    output.push_str("fn from(value: I18NValue<MAX_INDEX>) -> String {\n");
    output.push_str("value.as_str().to_string()\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl<const MAX_INDEX: usize> From<I18NValue<MAX_INDEX>> for &'static str {\n");
    output.push_str("fn from(value: I18NValue<MAX_INDEX>) -> &'static str {\n");
    output.push_str("value.as_str()\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl<const MAX_INDEX: usize> core::fmt::Display for I18NValue<MAX_INDEX> {\n");
    output.push_str(" fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {\n");
    output.push_str("f.write_str(self.as_str())\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str(format!("pub fn set_i18n_language(language: {language_name}) {{\n").as_str());
    output.push_str("SELECTION.store(match language {\n");
    for (idx, key) in variants.keys().enumerate() {
        output.push_str(format!("{language_name}::{key} => {idx},\n").as_str());
    }
    output.push_str("_ => 0,\n");
    output.push_str("} as u32, core::sync::atomic::Ordering::Relaxed);\n");
    output.push_str("}\n");

    let keys_sorted: BTreeSet<String> = variants.get(&default_variant).unwrap().properties.keys().cloned().collect();

    for k in &keys_sorted {
        let comp = complexity.get(k).unwrap().clone();
        output.push_str(format!("pub static {k}: I18NValue<{comp}> = I18NValue(&[").as_str());
        for (_, value) in variants.iter() {
            let prop_val = escape_string_for_source(value
                .properties
                .get(k)
                .unwrap());

            output.push_str("(");
            output.push_str("\"");
            output.push_str(prop_val.as_str());
            output.push_str("\",");
            if let Some(complex) = value.complex_properties.get(k) {
                output.push_str("&[");
                for (prefix, index) in complex {
                    let prefix = escape_string_for_source(prefix);
                    output.push_str("(\"");
                    output.push_str(prefix.as_str());
                    if *index == usize::MAX {
                        output.push_str("\", usize::MAX), ");
                    } else {
                        output.push_str("\", ");
                        output.push_str(index.to_string().as_str());
                        output.push_str("), ");
                    }
                }
                output.push_str("]");
            } else {
                output.push_str("&[]");
            }

            output.push_str("),");
        }
        output.push_str("]);\n");
    }

    eprintln!("{}", &output);
    output.parse().unwrap()
}

fn escape_string_for_source(input: &str) -> String {
    input.replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
        .replace("\"", "\\\"")
        .replace("\\", "\\\\")
}

fn validate_fallbacks_exist(variants: &mut LinkedHashMap<String, Variant>) {
    for variant in variants.values() {
        for fallback in &variant.fallbacks {
            if !variants.contains_key(fallback) {
                panic!(
                    "Language '{}' has fallback '{}' which does not exist.",
                    variant.name, fallback
                );
            }
        }
    }
}

fn validate_all_keys_in_default_language(
    default_variant: &str,
    variants: &mut LinkedHashMap<String, Variant>,
) {
    let default_variant_value = variants.get(default_variant).unwrap();

    for variant in variants.values() {
        for k in variant.properties.keys() {
            if !default_variant_value.properties.contains_key(k) {
                panic!(
                    "Language '{}' has a key called '{}' which does not exist in the default language '{}'. The default language must contain all keys!",
                    variant.name, k, default_variant
                );
            }
        }
    }
}

fn resolve_fallbacks_properties(
    default_variant: &str,
    variants: &mut LinkedHashMap<String, Variant>,
) {
    let mut cl = variants.clone();
    let default_variant_value = cl.get(default_variant).unwrap().clone();

    for (_, variant) in variants.iter_mut() {
        'next_prop: for (k, default_value) in default_variant_value.properties.iter() {
            if variant.properties.contains_key(k) {
                continue;
            }

            for fallback in &variant.fallbacks {
                if let Some(fallback_value) = cl.get(fallback).unwrap().properties.get(k) {
                    variant
                        .properties
                        .insert(k.to_string(), fallback_value.to_string());
                    continue 'next_prop;
                }
            }

            variant
                .properties
                .insert(k.to_string(), default_value.to_string());
        }

        //Replace the processed lang in the lookup map, we process them in natural order.
        cl.insert(variant.name.clone(), variant.clone());
    }
}

fn sort_properties(variants: &mut LinkedHashMap<String, Variant>) {
    let mut complex = HashSet::new();
    for (_, variant) in variants.iter_mut() {
        for (k, v) in variant.properties.iter() {
            let mut g = v.chars();
            let mut buf = String::new();
            while let Some(n) = g.next() {
                if n != '{' {
                    continue;
                }

                let Some(n) = g.next() else {
                    continue;
                };

                if !n.is_ascii_digit() {
                    continue;
                }

                buf.push(n);

                while let Some(n) = g.next() {
                    if n.is_ascii_digit() {
                        buf.push(n);
                        continue;
                    }

                    if n == '}' {
                        if buf.parse::<usize>().is_ok() {
                            complex.insert(k.to_string());
                        }
                    }
                    buf.clear();
                    break;
                }
            }
        }
    }


    for (_, variant) in variants.iter_mut() {
        for (k, v) in variant.properties.iter() {
            let mut res = Vec::new();
            let mut iter = v.chars();
            let mut kbuf = String::new();
            while let Some(n) = iter.next() {
                if n != '{' {
                    kbuf.push(n);
                    continue;
                }

                let Some(n) = iter.next() else {
                    kbuf.push('{');
                    continue;
                };

                if !n.is_ascii_digit() {
                    kbuf.push('{');
                    kbuf.push(n);
                    continue;
                }

                let mut nbuf = String::new();
                nbuf.push(n);

                while let Some(n) = iter.next() {
                    if n.is_ascii_digit() {
                        nbuf.push(n);
                        continue;
                    }

                    if n == '}' {
                        if let Ok(idx) = nbuf.parse::<usize>() {
                            res.push((mem::take(&mut kbuf), idx));
                            break;
                        }
                    }

                    kbuf.push('{');
                    kbuf.push_str(nbuf.as_str());
                    kbuf.push(n);
                    break;
                }
            }

            if !kbuf.is_empty() {
                res.push((mem::take(&mut kbuf), usize::MAX));
            }

            variant.complex_properties.insert(k.to_string(), res);
        }
    }
}

fn find_max_complexity(variants: &LinkedHashMap<String, Variant>) -> HashMap<String, usize> {
    let mut res = HashMap::new();
    for (_, variant) in variants.iter() {
        for (k, v) in variant.properties.iter() {
            res.insert(k.to_string(), 0);
        }
    }

    for (_, variant) in variants.iter() {
        for (k, v) in variant.complex_properties.iter() {
            let max = res.get_mut(k).unwrap();
            for (_, param) in v {
                if *param == usize::MAX {
                    continue;
                }

                if *max < (*param) + 1 {
                    *max = (*param) + 1;
                }
            }
        }
    }

    res
}

fn find_all_complexity(variants: &LinkedHashMap<String, Variant>) -> BTreeSet<usize> {
    let mut res = BTreeSet::new();
    res.insert(0);

    for (_, variant) in variants.iter() {
        for (k, v) in variant.complex_properties.iter() {
            let mut max = 0;
            for (_, param) in v {
                if *param == usize::MAX {
                    continue;
                }

                if max < (*param) + 1 {
                    max = (*param) + 1;
                }
            }

            res.insert(max);
        }
    }

    res
}


mod global_state {
    pub enum Language {
        English,
        German
    }
}
mod x {
    static SELECTION: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    pub trait I18NFormatParameter<const MAX_INDEX: usize> {
        fn format_parameter(&self, idx: usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
    }
    impl I18NFormatParameter<0> for () {
        fn format_parameter(&self, idx: usize, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }
    impl<const MAX_INDEX: usize, T: core::fmt::Display> I18NFormatParameter<MAX_INDEX> for &[T] {
        fn format_parameter(&self, idx: usize, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
            let Some(dsp) = self.get(idx) else {
                return Ok(());
            };
            core::fmt::Display::fmt(dsp, f)
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct I18NValue<const MAX_INDEX: usize>(&'static [(&'static str, &'static [(&'static str, usize)]); 2]);
    impl<const MAX_INDEX: usize> I18NValue<MAX_INDEX> {
        pub fn as_str(&self) -> &'static str {
            self.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize].0
        }
        pub const fn default_str(&self) -> &'static str {
            self.0[0].0
        }
        pub fn format_with<T: >(&self, arg: impl I18NFormatParameter<MAX_INDEX>, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            for (prefix, arg_index) in self.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize].1 {
                let idx = *arg_index;
                f.write_str(prefix)?;
                if idx != usize::MAX {
                    arg.format_parameter(idx, f)?;
                }
            }
            Ok(())
        }
        pub fn format(&self, arg: impl I18NFormatParameter<MAX_INDEX>) -> String {
            struct FMT<'a, const M: usize, T: I18NFormatParameter<M>>(&'a I18NValue<M>, T);
            impl<const M: usize, T: I18NFormatParameter<M>> core::fmt::Display for FMT<'_, M, T> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                    for (prefix, arg_index) in self.0.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize].1 {
                        let idx = *arg_index;
                        f.write_str(prefix)?;
                        if idx != usize::MAX {
                            self.1.format_parameter(idx, f)?;
                        }
                    }
                    Ok(())
                }
            }
            let formatter = FMT(self, arg);
            ToString::to_string(&formatter)
        }
    }
    impl<const MAX_INDEX: usize> AsRef<str> for I18NValue<MAX_INDEX> {
        fn as_ref(&self) -> &str {
            self.as_str()
        }
    }
    impl<const MAX_INDEX: usize> From<I18NValue<MAX_INDEX>> for String {
        fn from(value: I18NValue<MAX_INDEX>) -> String {
            value.as_str().to_string()
        }
    }
    impl<const MAX_INDEX: usize> From<I18NValue<MAX_INDEX>> for &'static str {
        fn from(value: I18NValue<MAX_INDEX>) -> &'static str {
            value.as_str()
        }
    }
    impl<const MAX_INDEX: usize> core::fmt::Display for I18NValue<MAX_INDEX> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.as_str())
        }
    }
    pub fn set_i18n_language(language: crate::global_state::Language) {
        SELECTION.store(match language {
            crate::global_state::Language::English => 0,
            crate::global_state::Language::German => 1,
            _ => 0,
        } as u32, core::sync::atomic::Ordering::Relaxed);
    }

    pub static HELLO_WORLD: I18NValue<0> = I18NValue(&[("Hello World!",&[("Hello World!", usize::MAX), ]),("Hallo Welt!",&[("Hallo Welt!", usize::MAX), ]),]);
    pub static WELD_SEAM: I18NValue<0> = I18NValue(&[("Weld seam",&[("Weld seam", usize::MAX), ]),("Schweißnaht",&[("Schweißnaht", usize::MAX), ]),]);
    pub static MOUNTAIN: I18NValue<0> = I18NValue(&[("Mountain",&[("Mountain", usize::MAX), ]),("Mountain",&[("Mountain", usize::MAX), ]),]);

}
