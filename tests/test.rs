#![allow(uncommon_codepoints)]

use crate::i18n::Language;

pub mod i18n {
    pub enum Language {
        English,
        UsaEnglish,
        German,
        SwissGerman,
        Other,
    }

    trivial_i18n::i18n! {
        10;
        Language;
        English="tests/english.properties";
        UsaEnglish="tests/us_english.properties";
        German="tests/german.properties";
        SwissGerman="tests/swiss_german.properties",German;
    }
}


#[test]
pub fn test() {
    i18n::set_i18n_language(Language::English);
    assert_eq!("Colour", i18n::COLOR.as_str());
    assert_eq!("Street", i18n::STREET.as_str());
    assert_eq!("Only in english", i18n::ENG_ONLY.as_str());

    assert_eq!("Test 123 Test", i18n::FORMAT_GER_ONLY.format(("beep",)));
    assert_eq!("Test 123 Test", i18n::FORMAT_GER_ONLY.format(&("beep",)));
    assert_eq!("Test 123 Test", i18n::FORMAT_GER_ONLY.format(["beep"]));
    assert_eq!("Test 123 Test", i18n::FORMAT_GER_ONLY.format(&["beep"]));
    assert_eq!("Test 123 Test", i18n::FORMAT_GER_ONLY.format(["beep", "two"].as_slice()));
    assert_eq!("Test 123 Test", i18n::FORMAT_GER_ONLY.as_str());

    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "bop"]));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&["beep", "bop"]));
    assert_eq!("Test1: two Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "two", "_ignored"].as_slice()));
    assert_eq!("Test1:  Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test2: {0}", i18n::TWO_PARAM_REVERSE.as_str());

    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"].as_slice()));
    assert_eq!("Test1:  Test4: ", i18n::TWO_PARAM_SKIP.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test4: {4}", i18n::TWO_PARAM_SKIP.as_str());

    assert_eq!("ABCティーポットABC", i18n::JAPAN_MOON_RUNES.as_str());
    assert_eq!("\"\"''&%/=?)(&%/))(&&§E&)   \0", i18n::ESCAPE_CHARACTERS.as_str());

    assert_eq!("123", i18n::_bad_DOT_key.as_str());
    assert_eq!("456", i18n::_123.as_str());
    assert_eq!("678", i18n::_123_0.as_str());
    assert_eq!("123", i18n::_123_1.as_str());
    assert_eq!("𝕊", i18n::𝕊.as_str());
    assert_eq!("😀", i18n::___0.as_str());
    assert_eq!("_", i18n::__.as_str());

    i18n::set_i18n_language(Language::UsaEnglish);
    assert_eq!("Color", i18n::COLOR.as_str());
    assert_eq!("Street", i18n::STREET.as_str());
    assert_eq!("Only in english", i18n::ENG_ONLY.as_str());

    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "bop"]));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&["beep", "bop"]));
    assert_eq!("Test1: two Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "two", "_ignored"].as_slice()));
    assert_eq!("Test1:  Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test2: {0}", i18n::TWO_PARAM_REVERSE.as_str());

    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "bop"]));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&["beep", "bop"]));
    assert_eq!("Test1: two Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "two", "_ignored"].as_slice()));
    assert_eq!("Test1:  Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test2: {0}", i18n::TWO_PARAM_REVERSE.as_str());


    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"].as_slice()));
    assert_eq!("Test1:  Test4: ", i18n::TWO_PARAM_SKIP.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test4: {4}", i18n::TWO_PARAM_SKIP.as_str());


    i18n::set_i18n_language(Language::German);
    assert_eq!("Farbe", i18n::COLOR.as_str());
    assert_eq!("Straße", i18n::STREET.as_str());
    assert_eq!("Only in english", i18n::ENG_ONLY.as_str());

    assert_eq!("Test beep Test", i18n::FORMAT_GER_ONLY.format(("beep",)));
    assert_eq!("Test bop Test", i18n::FORMAT_GER_ONLY.format(&("bop",)));
    assert_eq!("Test beep Test", i18n::FORMAT_GER_ONLY.format(["beep"]));
    assert_eq!("Test bop Test", i18n::FORMAT_GER_ONLY.format(&["bop"]));
    assert_eq!("Test mop Test", i18n::FORMAT_GER_ONLY.format(["mop", "two"].as_slice()));
    assert_eq!("Test {0} Test", i18n::FORMAT_GER_ONLY.as_str());

    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "bop"]));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&["beep", "bop"]));
    assert_eq!("Test1: two Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "two", "_ignored"].as_slice()));
    assert_eq!("Test1:  Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test2: {0}", i18n::TWO_PARAM_REVERSE.as_str());


    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"].as_slice()));
    assert_eq!("Test1:  Test4: ", i18n::TWO_PARAM_SKIP.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test4: {4}", i18n::TWO_PARAM_SKIP.as_str());


    i18n::set_i18n_language(Language::SwissGerman);
    assert_eq!("Farbe", i18n::COLOR.as_str());
    assert_eq!("Strasse", i18n::STREET.as_str());
    assert_eq!("Only in english", i18n::ENG_ONLY.as_str());

    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "bop"]));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&["beep", "bop"]));
    assert_eq!("Test1: two Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "two", "_ignored"].as_slice()));
    assert_eq!("Test1:  Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test2: {0}", i18n::TWO_PARAM_REVERSE.as_str());


    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"].as_slice()));
    assert_eq!("Test1:  Test4: ", i18n::TWO_PARAM_SKIP.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test4: {4}", i18n::TWO_PARAM_SKIP.as_str());

    i18n::set_i18n_language(Language::Other);
    assert_eq!("Colour", i18n::COLOR.as_str());
    assert_eq!("Street", i18n::STREET.as_str());
    assert_eq!("Only in english", i18n::ENG_ONLY.as_str());

    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&("beep", "bop")));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "bop"]));
    assert_eq!("Test1: bop Test2: beep", i18n::TWO_PARAM_REVERSE.format(&["beep", "bop"]));
    assert_eq!("Test1: two Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep", "two", "_ignored"].as_slice()));
    assert_eq!("Test1:  Test2: beep", i18n::TWO_PARAM_REVERSE.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test2: {0}", i18n::TWO_PARAM_REVERSE.as_str());


    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&("_ignored", "beep", "_ignored", "_ignored", "bap")));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(&["_ignored", "beep", "_ignored", "_ignored", "bap"]));
    assert_eq!("Test1: beep Test4: bap", i18n::TWO_PARAM_SKIP.format(["_ignored", "beep", "_ignored", "_ignored", "bap"].as_slice()));
    assert_eq!("Test1:  Test4: ", i18n::TWO_PARAM_SKIP.format(["beep"].as_slice()));
    assert_eq!("Test1: {1} Test4: {4}", i18n::TWO_PARAM_SKIP.as_str());
}