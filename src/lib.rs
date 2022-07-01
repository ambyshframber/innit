//! innit, a simple INI parser
//! 
//! # Usage
//! 
//! Create an [`IniDocument`] either from a string or empty.
//! 
//! ```
//! # use innit::*;
//! let ini = r"foo = bar
//! ## comment
//! ; comment
//! baz=bop
//! [section1]
//! foo = baz";
//! let document = IniDocument::from_string(ini);
//! println!("{:?}", document);
//! assert!(document.is_ok());
//! 
//! let doc2 = IniDocument::empty();
//! ```
//! 
//! You can get, insert, and remove key/value pairs in any section of the document.
//! The opening section is referred to with the empty string, and as a result new sections with the empty string as their name cannot be created.
//! 
//! ```
//! # use innit::*;
//! # let ini = r"foo = bar
//! # baz=bop
//! # [section1]
//! # foo = baz";
//! # let document = IniDocument::from_string(ini).unwrap();
//! assert_eq!(document.get("foo", ""), Some("bar"));
//! assert_eq!(document.get("foo", "section1"), Some("baz"));
//! ```
//! 
//! innit's version of INI is a stringly typed system, which means the only datatype is the string,
//! which means you'll have to parse integer or other structured data on a value-by-value basis.
//! It also means that you can mix and match multiple datatypes in the same document really easily, even more easily than something like JSON.
//! It ALSO also means that you don't need quotes or any quote escaping.
//! See [the wikipedia page on INI](https://en.wikipedia.org/wiki/INI_file) for more info.
//! 
//! innit is case sensitive by default, unlike the original MS-DOS and subsequent Windows implementations.
//! The `case_insensitive` feature enables use of the case insensitive methods.

#![deny(missing_docs)]
#![allow(clippy::comparison_to_empty)]

use std::collections::HashMap;
use thiserror::Error;

/// A parsed or generated INI document.
/// 
/// Under the hood, this is just a nested hashmap. The outer layer represents the document sections,
/// where the opening unnamed section is referred to with the empty string.
/// The inner layer represents keys and values inside a section.
/// 
/// Currently, comments are not preserved in any way.
#[derive(Debug, PartialEq, Default)]
pub struct IniDocument {
    sections: HashMap<String, HashMap<String, String>>
}
impl IniDocument {
    /// Create a new empty `IniDocument`.
    pub fn empty() -> IniDocument {
        IniDocument {
            sections: HashMap::new()
        }
    }
    /// Determine if an `IniDocument` is empty. A document that contains sections but no keys is considered empty.
    pub fn is_empty(&self) -> bool {
        if self.sections.is_empty() {
            true
        }
        else {
            !self.sections.iter().any(|(_, s)| !s.is_empty())
            // get a true if any section is not empty, then not it
        }
    }
    /// Insert a key into a given section. Returns the old value if it exists.
    pub fn insert<T, U, V>(&mut self, key: T, value: U, section: V) -> Option<String>
    where T: Into<String>, U: Into<String>, V: Into<String> {
        let section: String = section.into();
        if let Some(section) = self.sections.get_mut(&section) {
            section.insert(key.into(), value.into())
        }
        else {
            let mut h = HashMap::new();
            h.insert(key.into(), value.into());
            self.sections.insert(section, h);
            None
        }
    }
    /// Get a reference to a value in a given section.
    pub fn get<T: AsRef<str>>(&self, key: T, section: T) -> Option<&str> {
        let key = key.as_ref();
        let section = section.as_ref();
        if let Some(s) = self.sections.get(section) {
            s.get(key).map(|s| s.as_str())
        }
        else {
            None
        }
    }
    /// Get an entire document section, as a hashmap.
    pub fn get_section<T: AsRef<str>>(&self, section: T) -> Option<&HashMap<String, String>> {
        self.sections.get(section.as_ref())
    }
    /// Remove a key/value pair in a given section. Returns the value, if it existed.
    pub fn remove<T: AsRef<str>>(&mut self, key: T, section: T) -> Option<String> {
        let key = key.as_ref();
        let section = section.as_ref();
        if let Some(s) = self.sections.get_mut(section) {
            s.remove(key)
        }
        else {
            None
        }
    }
    /// Remove an entire section. Returns the section, if it existed.
    pub fn remove_section<T: AsRef<str>>(&mut self, section: T) -> Option<HashMap<String, String>> {
        let section = section.as_ref();
        self.sections.remove(section)
    }

    /// Parse a document from a string. Comments are not preserved when writing back to a string, so watch out!
    /// 
    /// Inline comments are not supported.
    pub fn from_string<T: AsRef<str>>(s: T) -> Result<IniDocument, InnitError> {
        let s = s.as_ref();
        let mut document = IniDocument::empty();
        let mut cur_section = "";
        for (lnum, line) in s.split(LINE_DELIM).enumerate() {
            let line = line.trim();
            if !string_is_comment_or_empty(line) { // ignore comments outright
                if let Some(name) = string_is_section_start(line) {
                    if name == "" {
                        return Err(InnitError::EmptyStringSection(lnum + 1))
                    }
                    cur_section = name
                }
                else {
                    let (k, v) = parse_k_v(line).ok_or_else(|| InnitError::MissingEquals(line.into(), lnum + 1))?;
                    document.insert(k, v, cur_section);
                }
            }
        }

        Ok(document)
    }
    /// Turn a document back into its string representation. Ordering of sections, keys and values is not preserved, due to limitations of Rust's hashmap struct.
    pub fn to_string(&self) -> String {
        let mut ret = String::new();

        if let Some(start) = self.sections.get("") {
            ret.push_str(&fmt_hashmap(start))
        }

        for (k, v) in &self.sections {
            if k == "" {
                continue
            }
            ret.push_str(&format!("[{}]{}", k, LINE_DELIM));
            ret.push_str(&fmt_hashmap(v))
        }

        ret
    }
}

//#[cfg(feature = "case_insensitive")]
impl IniDocument {
    /// Get a reference to a value in a given section, using case-insensitive matching.
    pub fn get_case_insensitive<T: AsRef<str>>(&self, key: T, section: T) -> Option<&str> {
        let section = section.as_ref().to_lowercase();
        for (name, data) in &self.sections {
            if name.to_lowercase() == section {
                let key = key.as_ref().to_lowercase();
                for (k, v) in data {
                    if k.to_lowercase() == key {
                        return Some(v)
                    }
                }
            }
        }
        None
    }
    /// Get a section, using case-insensitive matching.
    pub fn get_section_case_insensitive<T: AsRef<str>>(&self,section: T) -> Option<&HashMap<String, String>> {
        let section = section.as_ref().to_lowercase();
        for (name, data) in &self.sections {
            if name.to_lowercase() == section {
                return Some(data)
            }
        }
        None
    }

    /// Remove a key/value pair in a given section, using case-insensitive matching. Returns the value, if it existed.
    pub fn remove_case_insensitive<T: AsRef<str>>(&mut self, key: T, section: T) -> Option<String> {
        let section = section.as_ref().to_lowercase();
        let mut exists = false;
        let mut actual_section = String::new(); // store these back outside to appease the borrow checker
        let mut actual_key = String::new();

        'outer: for (name, data) in self.sections.iter_mut() {
            if name.to_lowercase() == section {
                actual_section = name.to_string();
                let key = key.as_ref().to_lowercase();
                for (k, _) in data {
                    if k.to_lowercase() == key {
                        actual_key = k.to_string();
                        exists = true;
                        break 'outer
                    }
                }
            }
        }
        if exists {
            self.sections.get_mut(&actual_section).unwrap().remove(&actual_key)
        }
        else {
            None
        }
    }
    /// Remove a section, using case-insensitive matching. Returns the section, if it existed.
    pub fn remove_section_case_insensitive<T: AsRef<str>>(&mut self, section: T) -> Option<HashMap<String, String>> {
        let section = section.as_ref().to_lowercase();
        let mut exists = false;
        let mut actual_section = String::new(); // store this back outside to appease the borrow checker

        for (name, _) in self.sections.iter_mut() {
            if name.to_lowercase() == section {
                actual_section = name.to_string();
                exists = true;
                break
            }
        }
        if exists {
            self.sections.remove(&actual_section)
        }
        else {
            None
        }
    }
}

/// format a hashmap
fn fmt_hashmap(h: &HashMap<String, String>) -> String {
    let mut ret = String::new();

    for (k, v) in h {
        ret.push_str(&format!("{} = {}{}", k, v, LINE_DELIM))
    }

    ret
}

#[cfg(feature = "crlf")]
const LINE_DELIM: &str = "\r\n";
#[cfg(not(feature = "crlf"))]
const LINE_DELIM: &str = "\n";

fn string_is_comment_or_empty(s: &str) -> bool {
    s.is_empty()|| s.starts_with('#') || s.starts_with(';')
}
/// returns Some if it is
fn string_is_section_start(s: &str) -> Option<&str> {
    if s.starts_with('[') && s.ends_with(']') {
        Some(&s[1..s.len() - 1])
    }
    else {
        None
    }
}
fn parse_k_v(s: &str) -> Option<(&str, &str)> {
    let split = s.split_once('=')?;
    Some((split.0.trim(), split.1.trim()))
}

/// The error returned from the document parse method.
/// 
/// The numbers inside the variants are the line numbers on which the error occured.
#[derive(Debug, Error, PartialEq)]
pub enum InnitError {
    /// A line inside a section was missing an equals sign, and is therefore an invalid key/value pair.
    #[error("bad k/v pair `{0}` on line {1}")]
    MissingEquals(String, usize),
    /// A section was defined with the empty string as the name.
    #[error("section with empty string as name on line {0}")]
    EmptyStringSection(usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let ini = r"foo = bar
# comment
; comment
baz=bop
[section1]
foo = baz";

        let document = IniDocument::from_string(ini);
        assert!(document.is_ok());
        let document = document.unwrap();
        assert_eq!(document.get("foo", ""), Some("bar"));
        assert_eq!(document.get("baz", ""), Some("bop"));
        assert_eq!(document.get("scrunkle", ""), None);
        assert_eq!(document.get("foo", "section1"), Some("baz"));

        let ini_back = document.to_string();
        println!("{}", ini_back)
    }

    #[test]
    fn errors() {
        let ini = "beans";
        let document = IniDocument::from_string(ini);
        assert_eq!(document, Err(InnitError::MissingEquals("beans".into(), 1)))
    }

    #[cfg(feature = "case_insensitive")]
    #[test]
    fn ci() {
        let ini = r"foo = bar
# comment
; comment
BAZ=bop
[section1]
foo = baz";
        let document = IniDocument::from_string(ini);
        assert!(document.is_ok());
        let document = document.unwrap();

        assert_eq!(document.get_case_insensitive("FOO", ""), Some("bar"));
        assert_eq!(document.get_case_insensitive("baz", ""), Some("bop"));
        assert_eq!(document.get_case_insensitive("foo", "SECtion1"), Some("baz"));
    }
}
