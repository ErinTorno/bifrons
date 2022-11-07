use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const ENGLISH: &'static str = "english";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Lines(pub HashMap<String, HashMap<String, String>>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineDictionary {
    pub lines: Lines,
    pub current: String,
}
impl Default for LineDictionary {
    fn default() -> Self {
        LineDictionary { lines: Lines::default(), current: ENGLISH.to_string() }
    }
}
impl LineDictionary {
    pub fn merge(&mut self, lines: &Lines) {
        for (lang, lines) in lines.0.iter() {
            if !self.lines.0.contains_key(lang) {
                self.lines.0.insert(lang.clone(), HashMap::new());
            }
            let dict = self.lines.0.get_mut(lang).unwrap();
            dict.extend(lines.into_iter().map(|(k, v)| (k.clone(), v.clone())));
        }
    }
}