# trivial_i18n
Trivially simple no-std proc-marco i18n processor, with 0 runtime dependencies.

The generated code supports language switching at runtime.
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

## Working with RustRover
RustRover will cache invocations of proc macros. This will cause problems because rust rover will
not realize that you have added a new key to a properties file. This is a known problem
in RustRover that has not been fixed for over a year by now. A lot of other crates that, for 
example, generate code based on a schema file also have this problem.

The only known workaround requires restarting the IDE. This is not that great.

To mitigate this, as a workaround for this bug, you can optionally add a "serial" number to the proc macro invocation.
You can increment this number to fool RustRover into re-evaluating the proc macro whenever you added or removed 
a key from the properties file.

Example:
```rust
mod i18n {
    enum SupportedLanguages {
        English,
        German
    }
    
    trivial_i18n::i18n! {
        1234; //Completely ignored and serves no function other than to trick rust rover, must, however, be a u128 number.
        SupportedLanguages;
        English="i18n/ENGLISH.properties";
        German="i18n/GERMAN.properties";
    }
}
```

It's up to you if you wish to keep this workaround number in your code or remove it before releasing/publishing
your software. It has no impact on the generated code.

## Simple Templating
Java has a class called MessageFormat.
It is often used together with resource bundles,
while MessageFormat has a lot of features, most remain unused.
It's primarily used for templating using simple substitution.

This crate automatically supports this simple substitution; 
however, it does not support more complex MessageFormat arguments.

Example
```
GREETING=Hello {0}! Today is {1}! Have a nice day!
```

```rust
fn test() {
    let formatted : String = i18n::GREETING.format(("John", "Tuesday"));
    assert_eq!("Hello John! Today is Tuesday! Have a nice day!", &formatted);
    let formatted2 = i18n::GREETING.format(&["John", "Tuesday"]);
    assert_eq!(&formatted, &formatted2);
    
    //If you are uninterested in using the format function, you can also access the key normally like any other regular key.
    assert_eq!("Hello {0}! Today is {1}! Have a nice day!", i18n::GREETING.as_str());
}
```

In addition to the `format(arg_tuple)` fn there are also the following functions:
* `format_with(arg_tuple, &mut core::fmt::Formatter) -> core::fmt::Result` - which is intended to be used within Display/Debug implementations.
* `format_into<T: core::fmt::Write>(arg_tuple, &mut T) -> core::fmt::Result` - which is intended to be used to append to a String or similar target buffers.

Example for `format_with`:
```rust
//Naturally this only makes sense for a more complex struct.
struct Dummy;

impl core::fmt::Display for Dummy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        i18n::GREETING.format_with(("John", "Tuesday"), f)
    }
}

fn test() {
    let formatted = format!("Greeting: {}", Dummy);
    assert_eq!("Hello John! Today is Tuesday! Have a nice day!", &formatted);
}
```

Example for `format_into`:
```rust
fn test() {
    let mut buffer = String::new();
    buffer.push_str("Greeting: ");
    
    _= i18n::TWO_PARAM_REVERSE.format_into(("John", "Tuesday"), &mut buffer);
    assert_eq!("Greeting: Hello John! Today is Tuesday! Have a nice day!", &buffer);
}
```

In general, the format functions accepts a slice of any length, a tuple of a matching length, or an array of a matching length.
The implementation does NOT panic if the slice is too small. It will simply substitute the indices which would be too large with empty string.
If the number of elements is known, then using tuples or arrays instead of slices is preferable because the compiler emits a compiler error if the number 
of elements in the tuple does NOT match the number of expected parameters.

The elements in the Tuple/Array/Slice can be any element that implements Display.

### Single parameter templating
Unfortunately, the format function does not accept a non-tuple single Display argument due to rust trait constraints.
This means to format a message with exactly one argument you have to use this syntax:

Example
```
GREETING=Hello {0}! Have a nice day!
```

```rust
fn test() {
    //The comma after "John" is important, it won't compile without it.
    let formatted : String = i18n::GREETING.format(("John", ));
    //Or pass it as a 1 element array...
    let formatted : String = i18n::GREETING.format(["John"]);
    assert_eq!("Hello John! Have a nice day!");
}
```

## Implementing custom traits for i18n values
```rust
pub trait MyTrait {
    fn dummy_fn(&self);
}

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
    
    // The macro will always generate a struct named I18NValue<usize> 
    // You can implement traits of your UI framework here like this, 
    // to allow for direct use of I18NValue in your framework.
    impl<const FORMAT_ARG_COUNT: usize> MyTrait for I18NValue<FORMAT_ARG_COUNT> {
        fn dummy_fn(&self) {
            _= self.as_str();
            _= self.format(["Whatever...", "Parameters..."]);
            //...
            todo!()
        }
    }
}
```

## Custom structs as format arguments.
```
GREETING=Hello {0}! Today is {1}! Have a nice day!
```

```rust
pub struct MyStruct {
    //...
}

pub struct MyStruct2 {
    //...
}

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
    
    // The macro will always generate a trait named I18NFormatParameter<usize> 
    // All types that implement this trait can be used in format/format_with/format_into as first parameter.
    //
    // If index is greater than what the struct can "handle" then the impl should do nothing as shown here.
    //
    // It is up to you if you implement this trait for a ref or the struct directly, 
    // most of the time the ref make more sense.
    impl<const FORMAT_ARG_COUNT: usize> I18NFormatParameter<FORMAT_ARG_COUNT> for &MyStruct {
        fn format_parameter(&self, index: usize, f: &mut Formatter<'_>) -> std::fmt::Result {
            assert!(index < FORMAT_ARG_COUNT); //This is guaranteed.
            
            match index {
                0 => f.write_str("John"),
                1 => f.write_str("Tuesday"),
                _ => Ok(()),
            }
        }
    }


    // If you know that your struct will only ever handle 2 parameters for example then you can also
    // remove the generic like this:
    // This will make any call to format/format_with/format_into with MyStruct2 
    // to a I18N value that does not accept exactly 2 parameters a compiler error.
    impl I18NFormatParameter<2> for &MyStruct2 {
        fn format_parameter(&self, index: usize, f: &mut Formatter<'_>) -> std::fmt::Result {
            match index {
                0 => f.write_str("John"),
                1 => f.write_str("Tuesday"),
                _ => unreachable!(), //This is guaranteed.
            }
        }
    }
}

fn test() {
    let formatted : String = i18n::GREETING.format(&MyStruct);
    let formatted : String = i18n::GREETING.format(&MyStruct2);
    assert_eq!("Hello John! Have a nice day!");
}
```
## How do I use this with my no-std crate?
You need to import alloc as well as some types that are not present in no-std.
After doing that, all other things behave exactly as with the standard library.

The proc macro does not generate those imports because they require `extern crate alloc` 
which is not desirable in crates that use the standard library.

This shows the minimal no-std example that compiles:
```rust
#![no_std]
extern crate alloc;

mod i18n {
    use alloc::string::String;
    use alloc::string::ToString;
    
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
    pub fn set_i18n_language(language: SupportedLanguages) {
        SELECTION.store(match language {
            SupportedLanguages::English => 0,
            SupportedLanguages::German => 1,
            _ => 0,
        } as u32, core::sync::atomic::Ordering::Relaxed);
    }

    pub static HELLO_WORLD: I18NValue<0> = I18NValue(&[("Hello World!",&[("Hello World!", usize::MAX), ]),("Hallo Welt!",&[("Hallo Welt!", usize::MAX), ]),]);
    pub static WELD_SEAM: I18NValue<0> = I18NValue(&[("Weld seam",&[("Weld seam", usize::MAX), ]),("Schweißnaht",&[("Schweißnaht", usize::MAX), ]),]);
    pub static MOUNTAIN: I18NValue<0> = I18NValue(&[("Mountain",&[("Mountain", usize::MAX), ]),("Mountain",&[("Mountain", usize::MAX), ]),]);
}
```