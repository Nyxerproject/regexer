use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct RegexError(String);

impl Display for RegexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RegexError: {}", self.0)
    }
}

impl Error for RegexError {}

pub struct CustomRegex {
    pattern: String,
}

impl CustomRegex {
    pub fn new(pattern: &str) -> Result<CustomRegex, RegexError> {
        if pattern.is_empty() {
            Err(RegexError("Empty pattern".to_string()))
        } else {
            Ok(CustomRegex {
                pattern: pattern.to_string(),
            })
        }
    }

    // find_iter simulates finding matches by checking for pattern substrings.
    // Real regex logic would be far more complex.
    pub fn find_iter<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut results = Vec::new();
        let mut start = 0;
        while let Some(pos) = text[start..].find(&self.pattern) {
            let abs_pos = start + pos;
            let matched = &text[abs_pos..abs_pos + self.pattern.len()];
            results.push(matched);
            start = abs_pos + self.pattern.len();
        }
        results
    }
}

pub struct RegexEngine {
    pattern: String,
}

impl RegexEngine {
    pub fn new(pattern: &str) -> Result<RegexEngine, RegexError> {
        if pattern.is_empty() {
            return Err(RegexError("Empty pattern".to_string()));
        }
        Ok(RegexEngine {
            pattern: pattern.to_string(),
        })
    }

    pub fn find_iter<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut results = Vec::new();
        let mut start = 0;
        let pattern = &self.pattern;

        while let Some(pos) = text[start..].find(pattern) {
            let abs_pos = start + pos;
            let matched = &text[abs_pos..abs_pos + pattern.len()];
            results.push(matched);
            start = abs_pos + pattern.len();
        }

        results
    }
}
