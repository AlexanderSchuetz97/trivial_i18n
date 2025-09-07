# trivial_i18n
Trivially simple no-std compile time i18n processor, with 0 runtime dependencies.

## Usage
```rust
mod i18n {
    //You can name this whatever you want
    pub enum SupportedLanguages {
        English,
        German,
        SwissGerman,
        French
    }
    
    //This is a proc-macro that will generate static fields for each resource key.
    trivial_i18n::i18n! {
        //The first element in the macro is the name of the Enum you want to use for language switching.
        //If the enum is in another module, you should qualify this name like this: crate::<path>::<name>
        SupportedLanguages;
        // The first language is the default language, all texts must exist there
        // This is checked at compile time. the "Path" behind the = is the path to a .properties 
        // file containing all resource keys relative to your Cargo.toml.
        English="i18n/ENGLISH.properties";
        //This is a secondary language, any keys that don't exist here will be taken from English
        German="i18n/GERMAN.properties";
        //This is yet another secondary language. Note tho that it falls back to German instead of English for missing keys.
        //You can keep specifying further fallbacks with more ',' if you so desire.
        //If a key cannot be found in any of the fallbacks, then naturally it will take them from English again.
        SwissGerman="i18n/SWISS_GERMAN.properties",German;
        //Note that 'French' is in the enum, but is mising here. For French, we will take all keys from English.
    }
}

#[test]
fn test() {
    use i18n::SupportedLanguages;
    
    //Note that this is entirely optional; by default, English is selected.
    i18n::set_i18n_language(SupportedLanguages::English);
    // Note that i18n::HELLO_WORLD and all other resource keys is Copy, Display, ToString, AsRef<str> by default. 
    // If you need to implement other traits for your message that can be implemented from any of those, 
    // then you can manually do so below the macro invocation.
    println!("{}", i18n::HELLO_WORLD); //Prints "Hello World!"
    let my_str: &'static str = i18n::HELLO_WORLD.as_str();
    assert_eq!("Hello World!", i18n::HELLO_WORLD.as_str());
    assert_eq!("Weld seam", i18n::WELD_SEAM.as_str());
    assert_eq!("Mountain", i18n::MOUNTAIN.as_str());
    
    set_i18n_language(SupportedLanguages::German);
    assert_eq!("Hallo Welt!", i18n::HELLO_WORLD.as_str()); // German spelling
    assert_eq!("Schweißnaht", i18n::WELD_SEAM.as_str()); // German spelling
    assert_eq!("Mountain", i18n::MOUNTAIN.as_str()); //No German translation, English it is
    
    set_i18n_language(SupportedLanguages::SwissGerman);
    assert_eq!("Hallo Welt!", i18n::HELLO_WORLD.as_str()); //Spelling from fallback 'German'
    assert_eq!("Schweissnaht", i18n::WELD_SEAM.as_str()); //Swiss german spelling.
    assert_eq!("Mountain", i18n::MOUNTAIN.as_str()); //No German translation, English it is

    i18n::set_i18n_language(SupportedLanguages::French);
    assert_eq!("Hello World!", i18n::HELLO_WORLD.as_str()); //French doesn't have any translations, English it is.
    assert_eq!("Weld seam", i18n::WELD_SEAM.as_str()); //French doesn't have any translations, English it is.
    assert_eq!("Mountain", i18n::MOUNTAIN.as_str()); //French doesn't have any translations, English it is.
}
```

i18n/ENGLISH.properties:
```
HELLO_WORLD=Hello World!
WELD_SEAM=Weld seam
MOUNTAIN=Mountain
```

i18n/GERMAN.properties:
```
HELLO_WORLD=Hallo Welt!
WELD_SEAM=Schweißnaht
```

i18n/SWISS_GERMAN.properties:
```
WELD_SEAM=Schweissnaht
```
## Runtime requirements
The compilation target needs to support AtomicU32 as well as Alloc. STD is not required.

There are no other runtime dependencies.

## What does the proc-macro generate?
This
```rust
mod i18n {
    enum SupportedLanguages {
        English,
        German
    }
    
    trivial_i18n::i18n! {
        SupportedLanguages;
        English="i18n/ENGLISH.properties";
        German="i18n/GERMAN.properties";
    }
}
```
expands to
```rust
mod i18n {
    enum SupportedLanguages {
        English,
        German
    }

    static SELECTION: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    
    #[derive(Debug, Copy, Clone)]
    pub struct I18NValue(&'static [&'static str; 2]);
    
    impl I18NValue {
        pub fn as_str(&self) -> &'static str {
            self.0[SELECTION.load(core::sync::atomic::Ordering::Relaxed) as usize]
        }
        pub const fn default_str(&self) -> &'static str {
            self.0[0]
        }
    }
    
    impl AsRef<str> for I18NValue {
        fn as_ref(&self) -> &str {
            self.as_str()
        }
    }
    impl From<I18NValue> for String {
        fn from(value: I18NValue) -> String {
            value.as_str().to_string()
        }
    }
    
    impl From<I18NValue> for &'static str {
        fn from(value: I18NValue) -> &'static str {
            value.as_str()
        }
    }
    
    impl core::fmt::Display for I18NValue {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.as_str())
        }
    }
    
    pub fn set_i18n_language(language: SupportedLanguages) {
        SELECTION.store(match language {
            SupportedLanguages::English => 0,
            SupportedLanguages::German => 1,
            _ => 0,
        } as u32, core::sync::atomic::Ordering::Relaxed);
    }
    
    pub static HELLO_WORLD: I18NValue = I18NValue(&["Hello World!","Hallo Welt!"]);
    pub static WELD_SEAM: I18NValue = I18NValue(&["Weld seam","Schweißnaht"]);
    pub static MOUNTAIN: I18NValue = I18NValue(&["Mountain","Mountain"]);
}
```
## Why a ".properties" file
Because that is what Java Resource Bundles use and translation companies usually know how to deal with it
since it has been around for the better part of 20 years by now.

It's also really simple to write and parse.
You can give this file to a person that has zero programming knowledge, and he will
be able to edit it with Notepad. The same cannot be said for JSON or YAML as it is much easier
to do syntax errors in those formats.

I am not opposed to adding support for different key-value resource file formats as long as they are
somewhat standardized and not 'custom'. There is no real reason for the proc macro to not support
whatever key->value file format there is. Either make a pull request or open an issue on GitHub.

## Future work
Adding support for templating. Java Resource Bundles support simple insertion templating,
for example, a key/value like this:
```
GREETING=Hello {0}! Have a nice day!
```
Would receive a String parameter, in this case the name of a person.
This still has to be supported by trivial_i18n.
