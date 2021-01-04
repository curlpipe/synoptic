use crate::tokens::FullToken;
use crate::tokens::Token;
use regex::{Error as ReError, Regex};
use std::collections::HashMap;

type Str = &'static str;

/// For performing highlighting operations
/// You can create a new Highlighter instance using the `new` method
/// ```rust
/// let mut h = Highlighter::new();
/// ```
pub struct Highlighter {
    pub regex: HashMap<&'static str, Vec<Regex>>,
    pub multiline_regex: HashMap<&'static str, Vec<Regex>>,
}

impl Highlighter {
    /// This will create a new, blank highlighter instance
    #[must_use] pub fn new() -> Self {
        // Create a new highlighter
        Self {
            regex: HashMap::new(),
            multiline_regex: HashMap::new(),
        }
    }

    /// This method allows you to add multiple definitions to the highlighter
    /// The first argument is for your list of definitions and the second is for the name
    /// This is useful for adding lists of keywords, for example:
    /// ```rust
    /// let mut python = Highlighter::new();
    /// python.join(&["def", "return", "import"], "keyword");
    /// ```
    /// For multiline tokens, you can add (?ms) or (?sm) to the beginning 
    ///
    /// # Errors
    /// This will return an error if one or more of your regex expressions are invalid
    pub fn join(&mut self, regex: &[Str], token: Str) -> Result<(), ReError> {
        // Add a regex that will match on a single line
        for i in regex {
            self.add(i, token)?;
        }
        Ok(())
    }

    /// This method allows you to add a single definition to the highlighter
    /// The first argument is for your definition and the second is for the name
    /// This is useful for adding things like regular expressions, for example:
    /// ```rust
    /// let mut python = Highlighter::new();
    /// python.add("[0-9]+", "number");
    /// ```
    /// For multiline tokens, you can add (?ms) or (?sm) to the beginning 
    ///
    /// # Errors
    /// This will return an error if your regex is invalid
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

    /// This is the method that you call to get the stream of tokens
    /// The argument is the string with the code that you wish to highlight
    /// This will return a vector of a vector of tokens, representing the lines and the tokens in
    /// them
    /// ```rust
    /// let mut python = Highlighter::new();
    /// python.add("[0-9]+", "number");
    /// python.run("some numbers: 123");
    /// ```
    /// This example will highlight the numbers `123` in the string
    pub fn run(&mut self, code: &'static str) -> Vec<Vec<Token>> {
        // Do the highlighting on the code
        let mut result: HashMap<usize, Vec<FullToken>> = HashMap::new();
        // Locate regular expressions
        for (name, expressions) in &self.regex {
            for expr in expressions {
                let captures = expr.captures_iter(code);
                for captures in captures {
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
                let captures = expr.captures_iter(code);
                for captures in captures {
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
                let tok = find_longest_token(&v);
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
            stream.push(Token::Text(eat));
        }
        if !stream.is_empty() {
            lines.push(stream);
        }
        lines
    }

}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// This is a method to find the token that occupies the most space
/// The argument is for the list of tokens to compare
fn find_longest_token(tokens: &[FullToken]) -> FullToken {
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

/// This is a method to insert regex into a hashmap
/// It takes the hashmap to add to, the regex to add and the name of the token
fn insert_regex(hash: &mut HashMap<Str, Vec<Regex>>, regex: Regex, token: Str) {
    // Insert regex into hashmap of vectors
    if let Some(v) = hash.get_mut(token) {
        v.push(regex);
    } else {
        hash.insert(token, vec![regex]);
    }
}

/// This is a method to insert a token into a hashmap
/// It takes the hashmap to add to, the token to add and the start position of the token
fn insert_token(map: &mut HashMap<usize, Vec<FullToken>>, key: usize, token: FullToken) {
    // Insert token into hashmap of vectors
    if let Some(v) = map.get_mut(&key) {
        v.push(token);
    } else {
        map.insert(key, vec![token]);
    }
}
