# innit

A simple INI parser

# Usage

Create an `IniDocument` either from a string or empty.

```
let ini = r"foo = bar
# comment
; comment
baz=bop
[section1]
foo = baz";
let document = IniDocument::from_string(ini);
println!("{:?}", document);
assert!(document.is_ok());

let doc2 = IniDocument::empty();
```

You can get, insert, and remove key/value pairs in any section of the document.
The opening section is referred to with the empty string, and as a result new sections with the empty string as their name cannot be created.

```
assert_eq!(document.get("foo", ""), Some("bar"));
assert_eq!(document.get("foo", "section1"), Some("baz"));
```
