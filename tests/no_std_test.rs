#![no_std]
extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;

pub enum Language {
    English,
    UsaEnglish,
    German,
    SwissGerman,
    Other,
}

trivial_i18n::i18n! {
        8;
        Language;
        English="tests/english.properties";
        UsaEnglish="tests/us_english.properties";
        German="tests/german.properties";
        SwissGerman="tests/swiss_german.properties",German;
}
#[test]
pub fn test() {
    //EMPTY We just test that this compiles.
}