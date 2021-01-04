use crate::tokens::FullToken;
use crate::tokens::Token;
use regex::{Error as ReError, Regex};
use std::collections::HashMap;

type Str = &'static str;

// For performing highlighting operations
pub struct Highlighter {
    pub regex: HashMap<&'static str, Vec<Regex>>,
    pub multiline_regex: HashMap<&'static str, Vec<Regex>>,
}

impl Highlighter {
    pub fn new() -> Self {
        // Create a new highlighter
        Self {
            regex: HashMap::new(),
            multiline_regex: HashMap::new(),
        }
    }

    pub fn join(&mut self, regex: &[Str], token: Str) -> Result<(), ReError> {
        // Add a regex that will match on a single line
        for i in regex {
            self.add(i, token)?;
        }
        Ok(())
    }

    pub fn add(&mut self, regex: Str, token: Str) -> Result<(), ReError> {
        // Add a regex that will match on a single line
        let re = Regex::new(regex)?;
        if regex.starts_with("(?ms)") || regex.starts_with("(?sm)") {
            insert_regex(&mut self.multiline_regex, re, token);
        } else {
            insert_regex(&mut self.regex, re, token);
        }
        Ok(())
    }

    pub fn run(&mut self, code: &'static str) -> Vec<Vec<Token>> {
        // Do the highlighting on the code
        let mut result: HashMap<usize, Vec<FullToken>> = HashMap::new();
        // Locate regular expressions
        for (name, expressions) in &self.regex {
            for expr in expressions {
                let mut captures = expr.captures_iter(code);
                while let Some(captures) = captures.next() {
                    if let Some(m) = captures.get(captures.len().saturating_sub(1)) {
                        insert_token(
                            &mut result,
                            m.start(),
                            FullToken {
                                text: m.as_str(),
                                kind: name,
                                start: m.start(),
                                end: m.end(),
                            },
                        );
                    }
                }
            }
        }
        // Locate multiline regular expressions
        for (name, expressions) in &self.multiline_regex {
            for expr in expressions {
                let mut captures = expr.captures_iter(code);
                while let Some(captures) = captures.next() {
                    if let Some(m) = captures.get(captures.len().saturating_sub(1)) {
                        insert_token(
                            &mut result,
                            m.start(),
                            FullToken {
                                text: m.as_str(),
                                kind: name,
                                start: m.start(),
                                end: m.end(),
                            },
                        );
                    }
                }
            }
        }
        // Use the hashmap into a vector
        let mut lines = vec![];
        let mut stream = vec![];
        let mut eat = String::new();
        let mut c = 0;
        let mut g = 0;
        let chars: Vec<char> = code.chars().collect();
        while c != code.len() {
            if let Some(v) = result.get(&c) {
                // There are tokens here
                if !eat.is_empty() {
                    stream.push(Token::Text(eat.to_string()));
                    eat = String::new();
                }
                // Get token
                let tok = self.find_longest_token(&v);
                stream.push(Token::Start(tok.kind));
                // Iterate over each character in the token text
                let mut token_eat = String::new();
                for ch in tok.text.chars() {
                    if ch == '\n' {
                        stream.push(Token::Text(token_eat));
                        token_eat = String::new();
                        stream.push(Token::End(tok.kind));
                        lines.push(stream);
                        stream = vec![Token::Start(tok.kind)];
                    } else {
                        token_eat.push(ch);
                    }
                }
                if !token_eat.is_empty() {
                    stream.push(Token::Text(token_eat))
                }
                stream.push(Token::End(tok.kind));
                c += tok.len();
                g += tok.text.chars().count();
            } else {
                // There are no tokens here
                if chars[g] == '\n' {
                    if !eat.is_empty() {
                        stream.push(Token::Text(eat.to_string()));
                    }
                    lines.push(stream);
                    stream = vec![];
                    eat = String::new();
                } else {
                    eat.push(chars[g]);
                }
                c += chars[g].to_string().len();
                g += 1;
            }
        }
        if !eat.is_empty() {
            stream.push(Token::Text(eat.to_string()));
        }
        if !stream.is_empty() {
            lines.push(stream);
        }
        lines
    }

    fn find_longest_token(&mut self, tokens: &Vec<FullToken>) -> FullToken {
        let mut longest = FullToken {
            text: "",
            kind: "",
            start: 0,
            end: 0,
        };
        for tok in tokens {
            if longest.len() < tok.len() {
                longest = *tok;
            }
        }
        longest
    }
}

fn insert_regex(hash: &mut HashMap<Str, Vec<Regex>>, regex: Regex, token: Str) {
    // Insert regex into hashmap of vectors
    if let Some(v) = hash.get_mut(token) {
        v.push(regex);
    } else {
        hash.insert(token, vec![regex]);
    }
}

fn insert_token(map: &mut HashMap<usize, Vec<FullToken>>, key: usize, token: FullToken) {
    // Insert token into hashmap of vectors
    if let Some(v) = map.get_mut(&key) {
        v.push(token);
    } else {
        map.insert(key, vec![token]);
    }
}
