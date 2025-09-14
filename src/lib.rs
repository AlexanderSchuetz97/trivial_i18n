use linked_hash_map::LinkedHashMap;
use proc_macro::{TokenStream, TokenTree};
use proc_macro::token_stream::IntoIter;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
struct Variant {
    name: String,
    path: String,
    fallbacks: Vec<String>,
    properties: HashMap<String, String>,
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

    eprintln!("language {}", &language_name);
    eprintln!("default language {}", &default_variant);
    eprintln!("default language path {}", &default_path);

    let mut variants = LinkedHashMap::new();

    variants.insert(
        default_variant.clone(),
        Variant {
            name: default_variant.clone(),
            path: default_path,
            fallbacks: vec![],
            properties: Default::default(),
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

    for (_name, v) in &variants {
        eprintln!("{:?}", v)
    }

    let mut output = String::with_capacity(0x4_00_00);
    output.push_str("static SELECTION: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);\n");
    output.push_str("#[derive(Debug, Copy, Clone)]");
    output.push_str(
        format!(
            "pub struct I18NValue(&'static [&'static str; {}]);\n",
            variants.len()
        )
        .as_str(),
    );
    output.push_str("impl I18NValue {\n");
    output.push_str("pub fn as_str(&self) -> &'static str {\n");
    output.push_str("self.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize]\n");
    output.push_str("}\n");
    output.push_str("pub const fn default_str(&self) -> &'static str {\n");
    output.push_str("self.0[0]\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl AsRef<str> for I18NValue {\n");
    output.push_str("fn as_ref(&self) -> &str {\n");
    output.push_str("self.as_str()\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl From<I18NValue> for String {\n");
    output.push_str("fn from(value: I18NValue) -> String {\n");
    output.push_str("value.as_str().to_string()\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl From<I18NValue> for &'static str {\n");
    output.push_str("fn from(value: I18NValue) -> &'static str {\n");
    output.push_str("value.as_str()\n");
    output.push_str("}\n");
    output.push_str("}\n");

    output.push_str("impl core::fmt::Display for I18NValue {\n");
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

    for k in variants.get(&default_variant).unwrap().properties.keys() {
        output.push_str(format!("pub static {k}: I18NValue = I18NValue(&[").as_str());
        for (_, value) in variants.iter() {
            let prop_val = value
                .properties
                .get(k)
                .unwrap()
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t")
                .replace("\"", "\\\"")
                .replace("\\", "\\\\");
            output.push_str("\"");
            output.push_str(prop_val.as_str());
            output.push_str("\",");
        }
        output.push_str("]);\n");
    }

    eprintln!("{}", &output);
    output.parse().unwrap()
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
