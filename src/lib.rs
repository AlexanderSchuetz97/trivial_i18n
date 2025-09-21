//! # `trivial_i18n`
//! Trivially simple no-std proc-marco i18n processor, with 0 runtime dependencies.
#![deny(
    clippy::correctness,
    clippy::perf,
    clippy::complexity,
    clippy::style,
    clippy::nursery,
    clippy::pedantic,
    clippy::clone_on_ref_ptr,
    clippy::decimal_literal_representation,
    clippy::float_cmp_const,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::unwrap_used,
    clippy::cargo_common_metadata,
    clippy::used_underscore_binding
)]
use linked_hash_map::LinkedHashMap;
use proc_macro::token_stream::IntoIter;
use proc_macro::{TokenStream, TokenTree};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::mem;
use std::path::Path;

#[derive(Debug, Clone)]
struct Variant {
    /// Language name
    name: String,
    /// Path to prop file
    path: String,
    /// Fallback languages
    fallbacks: Vec<String>,
    /// Raw properties key, value
    properties: HashMap<String, String>,
    /// Key->Vec<constant string prefix, index of format argument>
    /// If index is `usize::MAX` then that means it's a suffix.
    properties_split_by_format_args: HashMap<String, Vec<(String, usize)>>,
}

fn parse_path(input: &mut IntoIter) -> String {
    let mut language_name = String::new();

    for next in input.by_ref() {
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
    let mut token_iter: IntoIter = input.into_iter();

    let mut language_name = parse_path(&mut token_iter);

    if language_name.parse::<u128>().is_ok() {
        language_name = parse_path(&mut token_iter);
    }

    assert!(!language_name.is_empty(), "Trying to parse language name but no language name supplied.");

    let Some(TokenTree::Ident(lit)) = token_iter.next() else {
        panic!("Trying to parse language default enum name, a ident, but got non ident.");
    };

    let default_variant = lit.to_string();

    let Some(TokenTree::Punct(p)) = token_iter.next() else {
        panic!("Trying to parse = after default language enum name, but got non Punct TokenTree");
    };

    assert!((p.as_char() == '='), 
            "Trying to parse = after default language enum name, but got {}",
            p.as_char()
        );

    assert!(!default_variant.is_empty(), "Trying to parse language default variant but got empty token tree.");

    let Some(TokenTree::Literal(lit)) = token_iter.next() else {
        panic!("Trying to parse language default file path, a literal, but got non literal.");
    };

    let Some(TokenTree::Punct(p)) = token_iter.next() else {
        panic!("Trying to parse ; after language default file path, but got non Punct TokenTree");
    };

    assert!((p.as_char() == ';'), 
            "Trying to parse ; after language default file path, but got {}",
            p.as_char()
        );

    let default_path = lit.to_string();

    let mut variants = LinkedHashMap::new();

    variants.insert(
        default_variant.clone(),
        Variant {
            name: default_variant.clone(),
            path: default_path,
            fallbacks: vec![],
            properties: Default::default(),
            properties_split_by_format_args: Default::default(),
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

        assert!((p.as_char() == '='), 
                "Trying to parse = after language enum name {language_name}, but got {}",
                p.as_char()
            );

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

            assert!((p.as_char() == ','), 
                    "Trying to parse ; or , after language {variant_name} file path {variant_path}, but got {}",
                    p.as_char()
                );

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
                properties_split_by_format_args: Default::default(),
            },
        );
    }

    for (_, variant) in &mut variants {
        let path = Path::new(&variant.path[1..variant.path.len() - 1]);
        let mut prop_file_reader = BufReader::new(
            File::open(path).unwrap_or_else(|_| panic!("Failed to open file {} for language {}",
                    variant.path, variant.name)),
        );

        variant.properties = match jprop::parse_utf8_to_map(&mut prop_file_reader) {
            Ok(props) => props,
            Err(e) => panic!("Failed to parse .properties file: {}, {}", variant.path, e),
        }
    }

    validate_fallbacks_exist(&mut variants);
    validate_all_keys_in_default_language(&default_variant, &mut variants);
    resolve_fallbacks_properties(&default_variant, &mut variants);
    parse_property_values_for_substitution_format(&mut variants);
    let complexity = find_max_format_index_per_key(&variants);
    let all_complexity = find_all_format_indices(&variants);

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
                output.push_str(&format!("D{n}: core::fmt::Display, "));
            }

            output.push_str("> I18NFormatParameter<");
            output.push_str(k.to_string().as_str());
            output.push_str("> for ");
            output.push_str(prefix);
            output.push('(');

            for n in 0..k {
                output.push_str(&format!("D{n}, "));
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
    output.push_str(
        "struct FMT<'a, const M: usize, T: I18NFormatParameter<M>>(&'a I18NValue<M>, T);\n",
    );
    output.push_str(
        "impl<const M: usize, T: I18NFormatParameter<M>> core::fmt::Display for FMT<'_, M, T> {\n",
    );
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

    let keys_sorted: BTreeSet<String> = variants
        .get(&default_variant)
        .unwrap()
        .properties
        .keys()
        .cloned()
        .collect();

    for k in &keys_sorted {
        let comp = *complexity.get(k).unwrap();
        output.push_str(format!("pub static {k}: I18NValue<{comp}> = I18NValue(&[").as_str());
        for (_, value) in &variants {
            let prop_val = escape_string_for_source(value.properties.get(k).unwrap());

            output.push('(');
            output.push('"');
            output.push_str(prop_val.as_str());
            output.push_str("\",");
            if let Some(complex) = value.properties_split_by_format_args.get(k) {
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
                output.push(']');
            } else {
                output.push_str("&[]");
            }

            output.push_str("),");
        }
        output.push_str("]);\n");
    }

    eprintln!("{}", &output);
    match output.parse::<TokenStream>() {
        Ok(e) => e,
        Err(r) => panic!("Generated rust source code is invalid\n {output}\n error={r}"),
    }
}

/// Escapes some characters that cant be in a rust string without escaping.
/// This function is probably incomplete.
fn escape_string_for_source(input: &str) -> String {
    input
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\"', "\\\"")
        .replace('\\', "\\\\")
}

/// Checks that all fallback languages exist, panics otherwise.
fn validate_fallbacks_exist(variants: &LinkedHashMap<String, Variant>) {
    for variant in variants.values() {
        for fallback in &variant.fallbacks {
            assert!(
                variants.contains_key(fallback),
                "Language '{}' has fallback '{}' which does not exist.",
                variant.name,
                fallback
            );
        }
    }
}

/// checks that all keys are in the default language, panics otherwise.
fn validate_all_keys_in_default_language(
    default_variant: &str,
    variants: &LinkedHashMap<String, Variant>,
) {
    let default_variant_value = variants.get(default_variant)
        .expect("unreachable: validate_all_keys_in_default_language -> variants.get default_variant is none");

    for variant in variants.values() {
        for k in variant.properties.keys() {
            assert!(
                default_variant_value.properties.contains_key(k),
                "Language '{}' has a key called '{}' which does not exist in the default language '{}'. The default language must contain all keys!",
                variant.name,
                k,
                default_variant
            );
        }
    }
}

/// Resolves all fallback property values
fn resolve_fallbacks_properties(
    default_variant: &str,
    variants: &mut LinkedHashMap<String, Variant>,
) {
    let mut cl = variants.clone();

    let default_variant_value = cl
        .get(default_variant)
        .expect("unreachable: resolve_fallbacks_properties -> fallback.get default_variant is None")
        .clone();

    for (_, variant) in variants.iter_mut() {
        'next_prop: for (k, default_value) in &default_variant_value.properties {
            if variant.properties.contains_key(k) {
                continue;
            }

            for fallback in &variant.fallbacks {
                if let Some(fallback_value) = cl
                    .get(fallback)
                    .expect("unreachable: resolve_fallbacks_properties -> fallback.get is None")
                    .properties
                    .get(k)
                {
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

/// Parses all property values for templating format arguments.
fn parse_property_values_for_substitution_format(variants: &mut LinkedHashMap<String, Variant>) {
    for (_, variant) in variants.iter_mut() {
        for (k, v) in &variant.properties {
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

                for n in iter.by_ref() {
                    if n.is_ascii_digit() {
                        nbuf.push(n);
                        continue;
                    }

                    if n == '}'
                        && let Ok(idx) = nbuf.parse::<usize>()
                    {
                        res.push((mem::take(&mut kbuf), idx));
                        break;
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

            variant
                .properties_split_by_format_args
                .insert(k.to_string(), res);
        }
    }
}

/// Gets the maximum format index for every key.
/// Maximum refers to across all languages.
fn find_max_format_index_per_key(
    variants: &LinkedHashMap<String, Variant>,
) -> HashMap<String, usize> {
    let mut res = HashMap::new();
    for variant in variants.values() {
        for k in variant.properties.keys() {
            res.insert(k.to_string(), 0);
        }
    }

    for (_, variant) in variants {
        for (k, v) in &variant.properties_split_by_format_args {
            let max = res.get_mut(k).expect("infallible");
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

/// Finds all format indices used by all keys in all languages.
fn find_all_format_indices(variants: &LinkedHashMap<String, Variant>) -> BTreeSet<usize> {
    let mut res = BTreeSet::new();
    res.insert(0);

    for (_, variant) in variants {
        for v in variant.properties_split_by_format_args.values() {
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
