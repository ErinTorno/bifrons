use std::{default::default, collections::HashMap};

use serde::{Deserialize, Serialize};

pub const ENGLISH: &'static str = "english";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Lines(pub HashMap<String, String>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineDictionary {
    pub lines: HashMap<String, Lines>,
    pub current: String,
}
impl Default for LineDictionary {
    fn default() -> Self {
        LineDictionary { lines: default(), current: ENGLISH.to_string() }
    }
}
#[allow(dead_code)]
impl LineDictionary {
    pub fn merge<S>(&mut self, lang: S, lines: &Lines) where S: AsRef<str> {
        if !self.lines.contains_key(lang.as_ref()) {
            self.lines.insert(lang.as_ref().to_string(), default());
        }
        let dict = self.lines.get_mut(lang.as_ref()).unwrap();
        dict.0.extend(lines.0.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
}