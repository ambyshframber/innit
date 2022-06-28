use std::collections::HashMap;

#[derive(Debug)]
pub struct IniDocument {
    sections: HashMap<String, HashMap<String, String>>
}
impl IniDocument {
    pub fn empty() -> IniDocument {
        IniDocument {
            sections: HashMap::new()
        }
    }
    pub fn insert(&mut self, key: &str, value: &str, section: &str) {
        if let Some(section) = self.sections.get_mut(section) {
            section.insert(key.to_string(), value.to_string());
        }
        else {
            let mut h = HashMap::new();
            h.insert(key.to_string(), value.to_string());
            self.sections.insert(section.to_string(), h);
        }
    }
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

    pub fn from_string<T: AsRef<str>>(s: T) -> Result<IniDocument, InnitError> {
        let s = s.as_ref();
        let mut document = IniDocument::empty();
        let mut cur_section = "";
        for line in s.split(LINE_DELIM) {
            if !string_is_comment_or_empty(line) { // ignore comments outright
                if let Some(name) = string_is_section_start(line) {
                    cur_section = name
                }
                else {
                    let (k, v) = parse_k_v(line)?;
                    document.insert(k, v, cur_section)
                }
            }
        }

        Ok(document)
    }
    pub fn to_string(&self) -> String {
        let mut ret = String::new();

        if let Some(start) = self.sections.get("") {
            ret.push_str(&fmt_hashmap(&start))
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
    let s = s.trim();
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
fn parse_k_v(s: &str) -> Result<(&str, &str), InnitError> {
    let split = s.split_once('=').ok_or(InnitError::MissingEquals(s.into()))?;
    Ok((split.0.trim(), split.1.trim()))
}

#[derive(Debug)]
pub enum InnitError {
    MissingEquals(String)
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
        assert_eq!(ini_back, r"foo = bar
baz = bop
[section1]
foo = baz
")
    }
}
