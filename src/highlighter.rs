use crate::tokens::{Bounded, FullToken, TokOpt, Token};
use crate::{gidx, glen};
use regex::{Error as ReError, Regex};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;

/// For performing highlighting operations
/// You can create a new Highlighter instance using the `new` method
/// ```rust
/// let mut h = Highlighter::new();
/// ```
#[derive(Debug, Clone)]
pub struct Highlighter {
    pub regex: HashMap<String, Vec<Regex>>,
    pub multiline_regex: HashMap<String, Vec<Regex>>,
    pub bounded: Vec<Bounded>,
}

impl Highlighter {
    /// This will create a new, blank highlighter instance
    #[must_use]
    pub fn new() -> Self {
        // Create a new highlighter
        Self {
            regex: HashMap::new(),
            multiline_regex: HashMap::new(),
            bounded: Vec::new(),
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
    pub fn join(&mut self, regex: &[&str], token: &str) -> Result<(), ReError> {
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
    /// For multiline tokens, you can add (?ms) or (?sm) to the beginning.
    /// (See the `add_bounded` method for a better way of doing multiline tokens
    /// if you plan on doing file buffering.)
    ///
    /// # Errors
    /// This will return an error if your regex is invalid
    pub fn add(&mut self, regex: &str, token: &str) -> Result<(), ReError> {
        // Add a regex that will match on a single line
        let re = Regex::new(regex)?;
        if regex.starts_with("(?ms)") || regex.starts_with("(?sm)") {
            insert_regex(&mut self.multiline_regex, re, token);
        } else {
            insert_regex(&mut self.regex, re, token);
        }
        Ok(())
    }

    /// This method allows you to add a special, non-regex definition to the highlighter
    /// This not only makes it clearer to use for multiline tokens, but it will also allow you
    /// to buffer files from memory, and still be able to highlight multiline tokens, without
    /// having to have the end part visible in order to create a token.
    /// The first argument is for the text that starts the token
    /// The second argument is for the text that ends the token
    /// The third argument is true if you want to allow for escaping of the end token, false if
    /// not (for example, you might want to allow string escaping in strings).
    /// The forth argument is for the token name.
    /// ```rust
    /// let mut rust = Highlighter::new();
    /// rust.add_bounded("/*", "*/", false, "comment");
    /// ```
    /// You can still use regex to create a multiline token, but doing that won't guarantee that
    /// your highlighting will survive file buffering.
    pub fn add_bounded(&mut self, start: &str, end: &str, escaping: bool, token: &str) {
        let bounded = Bounded {
            kind: token.to_string(),
            start: start.to_string(),
            end: end.to_string(),
            escaping,
        };
        // Insert it into the bounded hashmap
        self.bounded.push(bounded);
    }

    /// A utility function to scan for just single line tokens
    fn run_singleline(&self, context: &str, result: &mut HashMap<usize, Vec<FullToken>>) {
        for (name, expressions) in &self.regex {
            for expr in expressions {
                let captures = expr.captures_iter(context);
                for captures in captures {
                    if let Some(m) = captures.get(captures.len().saturating_sub(1)) {
                        insert_token(
                            result,
                            m.start(),
                            FullToken {
                                text: m.as_str().to_string(),
                                kind: name.clone(),
                                start: m.start(),
                                end: m.end(),
                                multi: false,
                            },
                        );
                    }
                }
            }
        }
    }

    /// A utility function to scan for just multi line tokens
    fn run_multiline(&self, context: &str, result: &mut HashMap<usize, Vec<FullToken>>) {
        for (name, expressions) in &self.multiline_regex {
            for expr in expressions {
                let captures = expr.captures_iter(context);
                for captures in captures {
                    if let Some(m) = captures.get(captures.len().saturating_sub(1)) {
                        insert_token(
                            result,
                            m.start(),
                            FullToken {
                                text: m.as_str().to_string(),
                                kind: name.to_string(),
                                start: m.start(),
                                end: m.end(),
                                multi: true,
                            },
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::missing_panics_doc)]
    /// A utility function to scan for just bounded tokens
    pub fn run_bounded(&self, context: &str, result: &mut HashMap<usize, Vec<FullToken>>) {
        for tok in &self.bounded {
            // Init
            let mut start_index = 0;
            let mut grapheme_index = 0;
            // Iterate over each character
            while start_index < context.len() {
                // Get and check for potential start token match
                let potential_token: String = context
                    .chars()
                    .skip(grapheme_index)
                    .take(glen!(tok.start))
                    .collect();

                // If there is a start token, keep incrementing until end token is found
                if potential_token == tok.start {
                    let tok_start_index = start_index;
                    let mut tok_grapheme_index = grapheme_index;

                    // Start creating token
                    let mut current_token = FullToken {
                        kind: tok.kind.to_string(),
                        text: tok.start.to_string(),
                        start: tok_start_index,
                        end: tok_start_index + tok.start.len(),
                        multi: false,
                    };
                    tok_grapheme_index += glen!(tok.start);
                    let mut potential_end: String = "".to_string();
                    while potential_end != tok.end && current_token.end != context.len() {
                        potential_end = context
                            .chars()
                            .skip(tok_grapheme_index)
                            .take(glen!(tok.end))
                            .collect();
                        // Check for potential escaped end character to skip over
                        if tok.escaping {
                            if let Some(lookahead) =
                                context.chars().nth(tok_grapheme_index + glen!(tok.end))
                            {
                                if format!("{}{}", potential_end, lookahead)
                                    == format!("\\{}", tok.end)
                                {
                                    current_token.end += 1 + tok.end.len();
                                    write!(current_token.text, "\\{}", tok.end).unwrap();
                                    tok_grapheme_index += 1 + glen!(tok.end);
                                    continue;
                                }
                            }
                        }
                        if potential_end == tok.end {
                            current_token.end += tok.end.len();
                            current_token.text.push_str(&tok.end);
                            break;
                        }
                        // Part of the token, append on
                        current_token
                            .text
                            .push(context.chars().nth(tok_grapheme_index).unwrap());
                        current_token.end += gidx!(context, tok_grapheme_index);
                        tok_grapheme_index += 1;
                    }
                    // Update and add the token to the end result
                    current_token.multi = current_token.text.contains('\n');
                    insert_token(result, current_token.start, current_token);
                }
                // Update the indices
                if start_index < context.len() {
                    start_index += gidx!(context, grapheme_index);
                    grapheme_index += 1;
                }
            }
        }
    }

    /// This is the method that you call to get the stream of tokens for a specific line.
    /// The first argument is the string with the code that you wish to highlight.  
    /// the second argument is the line number that you wish to highlight.
    /// It returns a vector of tokens which can be used to highlight the individual line
    /// ```rust
    /// let mut lua = Highlighter::new();
    /// lua.add("(?ms)[[.*?]]", "string");
    /// lua.add("print", "keyword");
    /// lua.run_line(r#"
    /// print ([[ Hello World!
    /// ]])
    /// "#, 2);
    /// ```
    /// This example will return the second line, with the `]]` marked as a string
    /// The advantage of using this over the `run` method is that it is a lot faster
    /// This is because it only has to render one line rather than all of them, saving time
    ///
    /// This won't work with bounded tokens due to problems with determining what is a start
    /// token and what isn't. Bounded tokens require all lines above to be loaded, which
    /// run line doesn't assume.
    #[must_use]
    pub fn run_line(&self, context: &str, line: usize) -> Option<Vec<Token>> {
        // Locate multiline stuff
        let mut result: HashMap<usize, Vec<FullToken>> = HashMap::new();
        // Locate multiline regular expressions
        self.run_multiline(context, &mut result);
        // Calculate start and end indices (raw) of the line
        let (mut start, mut end) = (0, 0);
        let mut current_line = 0;
        let mut raw: usize = 0;
        for i in context.chars() {
            raw += i.to_string().len();
            if i == '\n' {
                current_line += 1;
                match current_line.cmp(&line) {
                    Ordering::Equal => start = raw,
                    Ordering::Greater => {
                        end = raw.saturating_sub(1);
                        break;
                    }
                    #[cfg(not(tarpaulin_include))]
                    Ordering::Less => (),
                }
            }
        }
        // Prune multiline tokens
        for (s, tok) in result.clone() {
            let tok = find_longest_token(&tok);
            if tok.start > end || tok.end < start {
                // This token is before or after this line
                result.remove(&s);
            } else {
                // This token is outside this line
                result.insert(s, vec![tok]);
            }
        }
        // Get then line contents
        let line_text = &context.get(start..end)?;
        // Locate single line tokens within the line (not the context - hence saving time)
        self.run_singleline(line_text, &mut result);
        // Split multiline tokens to ensure all data in result is relevant
        for (s, tok) in result.clone() {
            let tok = tok[0].clone();
            if tok.multi {
                // Check if line starts in token
                let tok_start = if start > tok.start && start < tok.end {
                    start - tok.start
                } else {
                    0
                };
                let tok_end = if end > tok.start && end < tok.end {
                    end - tok.start
                } else {
                    tok.len()
                };
                let tok_text = &tok.text[tok_start..tok_end];
                let true_start = if start > tok.start {
                    0
                } else {
                    tok.start - start
                };
                let true_end = true_start + tok_text.len();
                result.remove(&s);
                let tok = FullToken {
                    text: tok_text.to_string(),
                    kind: tok.kind,
                    start: true_start,
                    end: true_end,
                    multi: true,
                };
                result.insert(true_start, vec![tok]);
            }
        }
        // Assemble the line
        let mut stream = vec![];
        let mut eat = String::new();
        let mut c = 0;
        let mut g = 0;
        let chars: Vec<char> = line_text.chars().collect();
        while c != line_text.len() {
            if let Some(v) = result.get(&c) {
                // There are tokens here
                if !eat.is_empty() {
                    stream.push(Token::Text(eat.to_string()));
                    eat = String::new();
                }
                // Get token
                let tok = find_longest_token(v);
                stream.push(Token::Start(tok.kind.clone()));
                // Iterate over each character in the token text
                let mut token_eat = String::new();
                for ch in tok.text.chars() {
                    token_eat.push(ch);
                }
                if !token_eat.is_empty() {
                    stream.push(Token::Text(token_eat));
                }
                stream.push(Token::End(tok.kind.clone()));
                c += tok.len();
                g += tok.text.chars().count();
            } else {
                // There are no tokens here
                eat.push(chars[g]);
                c += chars[g].to_string().len();
                g += 1;
            }
        }
        if !eat.is_empty() {
            stream.push(Token::Text(eat));
        }
        Some(stream)
    }

    /// This is the method that you call to get the stream of tokens
    /// The argument is the string with the code that you wish to highlight
    /// Return a vector of a vector of tokens, representing the lines and the tokens in them
    /// ```rust
    /// let mut python = Highlighter::new();
    /// python.add("[0-9]+", "number");
    /// python.run("some numbers: 123");
    /// ```
    /// This example will highlight the numbers `123` in the string
    #[must_use]
    pub fn run(&self, code: &str) -> Vec<Vec<Token>> {
        // Do the highlighting on the code
        let mut result: HashMap<usize, Vec<FullToken>> = HashMap::new();
        // Locate regular expressions
        self.run_singleline(code, &mut result);
        // Locate multiline regular expressions
        self.run_multiline(code, &mut result);
        // Locate bounded tokens
        self.run_bounded(code, &mut result);
        // Use the hashmap into a vector
        let mut lines = vec![];
        let mut stream = vec![];
        let mut eat = String::new();
        let mut c = 0;
        let mut g = 0;
        let chars: Vec<char> = code.chars().collect();
        while c < code.len() {
            if let Some(v) = result.get(&c) {
                // There are tokens here
                if !eat.is_empty() {
                    stream.push(Token::Text(eat.to_string()));
                    eat = String::new();
                }
                // Get token
                let tok = find_longest_token(v);
                stream.push(Token::Start(tok.kind.clone()));
                // Iterate over each character in the token text
                let mut token_eat = String::new();
                for ch in tok.text.chars() {
                    if ch == '\n' {
                        stream.push(Token::Text(token_eat));
                        token_eat = String::new();
                        stream.push(Token::End(tok.kind.clone()));
                        lines.push(stream);
                        stream = vec![Token::Start(tok.kind.clone())];
                    } else {
                        token_eat.push(ch);
                    }
                }
                if !token_eat.is_empty() {
                    stream.push(Token::Text(token_eat));
                }
                stream.push(Token::End(tok.kind.clone()));
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
        lines.push(stream);
        lines
    }

    /// This is a function that will convert from a stream of tokens into a token option type
    /// A token option type is nicer to work with for certain formats such as HTML
    #[must_use]
    pub fn from_stream(input: &[Token]) -> Vec<TokOpt> {
        let mut result = vec![];
        let mut current = String::new();
        let mut toggle = false;
        for i in input {
            match i {
                Token::Start(_) => {
                    toggle = true;
                }
                Token::Text(t) => {
                    if toggle {
                        current.push_str(t);
                    } else {
                        result.push(TokOpt::None(t.clone()));
                    }
                }
                Token::End(k) => {
                    toggle = false;
                    result.push(TokOpt::Some(current, k.clone()));
                    current = String::new();
                }
            }
        }
        result
    }

    /// This is a function that will convert from a tokopt slice to a token stream
    /// A token stream is easier to render for certain formats such as the command line
    #[must_use]
    pub fn from_opt(input: &[TokOpt]) -> Vec<Token> {
        let mut result = vec![];
        for i in input {
            match i {
                TokOpt::Some(text, kind) => {
                    result.push(Token::Start(kind.to_string()));
                    result.push(Token::Text(text.clone()));
                    result.push(Token::End(kind.to_string()));
                }
                TokOpt::None(text) => result.push(Token::Text(text.clone())),
            }
        }
        result
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
        text: "".to_string(),
        kind: "".to_string(),
        start: 0,
        end: 0,
        multi: false,
    };
    for tok in tokens {
        if longest.len() < tok.len() {
            longest = tok.clone();
        }
    }
    longest
}

/// This is a method to insert regex into a hashmap
/// It takes the hashmap to add to, the regex to add and the name of the token
fn insert_regex(hash: &mut HashMap<String, Vec<Regex>>, regex: Regex, token: &str) {
    // Insert regex into hashmap of vectors
    if let Some(v) = hash.get_mut(token) {
        v.push(regex);
    } else {
        hash.insert(token.to_string(), vec![regex]);
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
