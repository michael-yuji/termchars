# termchar

A Rust crate to deal with string with control sequences, colors, and unicode all together.

String is hard on almost every programming languages due to its complexity, since Rust is a system language, the design of the standard library is force the use in the context of system and getting as close in memory as possible. For example the `len()` api will return the length as it bytes instead of codepoint or graphemes.

If you want to deal with String, in the context of write to files or deal with data, you should use the [unicode-segmentation](https://docs.rs/unicode-segmentation/latest/unicode_segmentation/index.html) crate. 

Unfortunately for people who want to print to the terminal, you have another unique issue, [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters), the escape codes are the reason your terminal can scroll up and down and implement "live" UI like progress bar. Essentially, these are special seq of string to tell your terminal what to do, including formatting and prining color. These control sequence is not supported by the unicode library and will report incorrect characters count if the `String` contains these special sequences. Oh, have I mentioned about invisible ascii characters?

This crate is not design to priority on super fast performance nor extreme flexible feature set. This crate exists to only to aid dealing with String. It is using unicode-segmenation underneath and is aware of (some) the ascii escape sequences hence can produce a much reliable count on characters. It also comes with api to trunacate a formatted string to a "human-visible" width. Which is useful if you want to print a table with nice alignment to the terminal. 
