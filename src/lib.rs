use unicode_width::UnicodeWidthChar;
pub use regex::Regex;
use std::collections::HashMap;
use std::ops::Range;

/// Represents a point in a 2d space
#[derive(Debug, Clone, PartialEq)]
pub struct Loc {
    y: usize,
    x: usize,
}

/// A definition of an Atom
/// See [Atom] for more information
#[derive(Debug, Clone)]
pub struct AtomDef {
    /// Name of the atom
    name: String,
    /// The kind of atom
    kind: AtomKind,
    /// The corresponding bounded token definition
    tok: Option<usize>,
    /// The regex expression that defines this atom
    exp: Regex,
}

/// The kind of atom being represented
#[derive(Debug, Clone, PartialEq)]
pub enum AtomKind {
    /// This is the start atom of a token, for example /* for a multiline comment
    Start,
    /// This is the end atom of a token, for example */ for a multiline comment
    End,
    /// Sometimes bounded tokens have the same start and end atom, e.g. a string having a " to
    /// start and an " to end, a hybrid token allows atoms to be used to start and end a token in
    /// cases where due to having the same start and end atom definitions, their kind is ambiguous
    Hybrid,
    /// This is just a normal keyword
    Keyword,
    /// This is a start marker for interpolation
    InterpolateStart,
    /// This is an end marker for interpolation
    InterpolateEnd,
}

/// An atom is a portion of text within a document that is significant. 
/// An atom only covers one line.
/// Atoms cover keywords as well as start and end indicators for bounded tokens
/// E.g., in a string, the atoms would be the starting " and the ending "
#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    /// Name of the atom
    name: String,
    /// The kind of atom
    kind: AtomKind,
    /// The corresponding token
    tok: Option<usize>,
    /// The range covered by the atom
    x: Range<usize>,
    /// Whether or not there is a preceding backslash
    backslashed: bool,
}

/// Definition for a bounded token, these are tokens that can cover multiple lines.
/// Things like multiline comments and strings are examples of this.
/// They work well for buffering files where you are unaware of where the end indicator may be as
/// it occurs further down in the file.
#[derive(Debug)]
pub struct BoundedDef {
    /// The name of the bounded token
    name: String,
    /// The start pattern of the bounded token
    start: String,
    /// The end pattern of the bounded token
    end: String,
    /// Whether or not this token can be escaped
    escapable: bool,
    /// The start pattern of interpolation region
    i_start: Option<String>,
    /// The end pattern of interpolation region
    i_end: Option<String>,
}

/// This is a TokenRef, which contains detailed information on what a token is
#[derive(Debug, Clone, PartialEq)]
pub enum TokenRef {
    /// Keyword tokens
    Keyword {
        /// The name of the bounded token
        name: String,
        /// A reference to the keyword atom
        atom: Loc,
    },
    /// Bounded tokens
    Bounded {
        /// The name of the bounded token
        name: String,
        /// A reference to the start atom
        start: Loc,
        /// A reference to the end atom
        end: Option<Loc>,
    },
}

/// This is an enum for representing tokens.
#[derive(Debug, Clone)]
pub enum TokOpt {
    /// The Some variant represents a token being present in the format Some(TEXT, NAME).
    ///
    /// So for a comment token, you can expect to see Some("/* comment */", "comment")
    /// provided that you defined the comment using either the keyword or bounded function on
    /// [Highlighter]
    Some(String, String),
    /// The None variant represents just plain text.
    None(String),
}

impl TokOpt {
    /// Works out if this token is empty, and thus redundant
    pub fn is_empty(&self) -> bool {
        let (TokOpt::Some(text, _) | TokOpt::None(text)) = self;
        text.len() == 0
    }

    /// This will remove the first character from the end of this token
    pub fn nibble(&mut self) -> Option<char> {
        let (TokOpt::Some(ref mut text, _) | TokOpt::None(ref mut text)) = self;
        let ch = text.chars().nth(0)?;
        text.remove(0);
        if ch.width()? > 1 {
            text.insert(0, ' ');
        }
        Some(ch)
    }
}

/// This is the main struct that will highlight your document
#[derive(Debug)]
pub struct Highlighter {
    /// The list of atoms, encapsulated within an inner vector for atoms on the same line
    pub atoms: Vec<Vec<Atom>>,
    /// The list of atom definitions to be used at atomization
    pub atom_def: Vec<AtomDef>,
    /// The list of bounded definitions to be used at tokenization
    pub bounded_def: Vec<BoundedDef>,
    /// A reference to what tokens lie on which line numbers
    pub line_ref: Vec<Vec<usize>>,
    /// A list of the resulting tokens generated from run and append
    pub tokens: Vec<TokenRef>,
    /// How many spaces a tab character should be
    pub tab_width: usize,
    /// For purposes of tokenization
    tokenize_state: Option<usize>,
    tokenize_interp: bool,
}

impl Highlighter {
    /// Creates a new highlighter
    pub fn new(tab_width: usize) -> Self {
        Self {
            atoms: vec![],
            atom_def: vec![],
            bounded_def: vec![],
            line_ref: vec![],
            tokens: vec![],
            tab_width,
            tokenize_state: None,
            tokenize_interp: false,
        }
    }

    /// Register a new keyword token, provide its name and regex
    pub fn keyword<S: Into<String>>(&mut self, name: S, exp: &str) {
        let name = name.into();
        let exp = Regex::new(&exp).expect("Invalid regex!");
        self.atom_def.push(AtomDef { name, exp, kind: AtomKind::Keyword, tok: None });
    }
    
    /// Register a new bounded token, with a start and end, 
    /// e.g. a multiline comment having starting /* and an ending */ to delimit it
    /// The last argument is a boolean
    /// when true, tokens can be escaped with a backslash e.g. "\"" would be a string of a quote
    pub fn bounded<S: Into<String>>(&mut self, name: S, start: S, end: S, escapable: bool) {
        let (name, start, end) = (name.into(), start.into(), end.into());
        // Gather atom information
        let start_exp = Regex::new(&start).expect("Invalid start regex");
        let end_exp = Regex::new(&end).expect("Invalid end regex");
        let hybrid = start == end;
        // Register bounded definition
        let idx = self.bounded_def.len();
        self.bounded_def.push(BoundedDef { 
            name: name.clone(),
            start, end,
            i_start: None, i_end: None,
            escapable,
        });
        // Register atom definitions
        if hybrid {
            self.atom_def.push(AtomDef { 
                name,
                exp: start_exp,
                kind: AtomKind::Hybrid,
                tok: Some(idx),
            });
        } else {
            self.atom_def.push(AtomDef { 
                name: name.clone(),
                exp: start_exp,
                kind: AtomKind::Start,
                tok: Some(idx),
            });
            self.atom_def.push(AtomDef { 
                name,
                exp: end_exp,
                kind: AtomKind::End,
                tok: Some(idx),
            });
        }
    }

    /// Register a new interpolatable bounded token, with a start and end, 
    /// e.g. a string as a bounded token, but allowing substitution between {}
    /// The last argument is a boolean
    /// when true, tokens can be escaped with a backslash e.g. "\"" would be a string of a quote
    pub fn bounded_interp<S: Into<String>>(&mut self, name: S, start: S, end: S, i_start: S, i_end: S, escapable: bool) {
        let (name, start, end, i_start, i_end) = (name.into(), start.into(), end.into(), i_start.into(), i_end.into());
        if i_start == i_end { panic!("start and end markers for interpolation must not be equal!"); }
        // Gather atom information
        let start_exp = Regex::new(&start).expect("Invalid start regex");
        let end_exp = Regex::new(&end).expect("Invalid end regex");
        let hybrid = start == end;
        let i_start_exp = Regex::new(&i_start).expect("Invalid interpolation start regex");
        let i_end_exp = Regex::new(&i_end).expect("Invalid interpolation end regex");
        // Register bounded definition
        let idx = self.bounded_def.len();
        self.bounded_def.push(BoundedDef { 
            name: name.clone(), 
            start, end, 
            i_start: Some(i_start), 
            i_end: Some(i_end),
            escapable,
        });
        // Register atom definitions
        if hybrid {
            self.atom_def.push(AtomDef { 
                name: name.clone(),
                exp: start_exp,
                kind: AtomKind::Hybrid,
                tok: Some(idx),
            });
        } else {
            self.atom_def.push(AtomDef { 
                name: name.clone(),
                exp: start_exp,
                kind: AtomKind::Start,
                tok: Some(idx),
            });
            self.atom_def.push(AtomDef { 
                name: name.clone(),
                exp: end_exp,
                kind: AtomKind::End,
                tok: Some(idx),
            });
        }
        self.atom_def.push(AtomDef { 
            name: name.clone(),
            exp: i_start_exp,
            kind: AtomKind::InterpolateStart,
            tok: Some(idx),
        });
        self.atom_def.push(AtomDef { 
            name: name.clone(),
            exp: i_end_exp,
            kind: AtomKind::InterpolateEnd,
            tok: Some(idx),
        });
    }

    /// Do an initial pass on a vector of lines.
    ///
    /// Note that this will overwrite any existing information,
    /// use append to add extra lines to the document.
    pub fn run(&mut self, lines: &Vec<String>) {
        // Atomize every line
        self.atoms = lines.iter().map(|l| self.atomize(l)).collect();
        self.tokenize();
    }

    /// Appends a line to the highlighter.
    pub fn append(&mut self, line: &String) {
        // Atomize this line
        self.atoms.push(self.atomize(line));
        self.line_ref.push(vec![]);
        self.tokenize_line(self.atoms.len().saturating_sub(1));
    }

    /// Once you have called the run or append methods, you can use this function
    /// to retrieve individual lines by providing the original line text and the y index.
    ///
    /// # Example
    /// ```
    /// let highlighter = Highlighter::new(4); // Tab ('\t') has a display width of 4
    /// highlighter.keyword("kw", "keyword"); // All occurances of "keyword" will be classed as a token of "kw"
    /// highlighter.run(vec![
    ///     "this is a keyword".to_string(), 
    ///     "second line!".to_string()
    /// ]);
    /// // Get the TokOpt for the first line
    /// highlighter.line(0, &"this is a keyword".to_string())
    /// // Get the TokOpt for the second line
    /// highlighter.line(1, &"second line!".to_string())
    /// ```
    pub fn line(&self, y: usize, line: &String) -> Vec<TokOpt> {
        let line = line.replace("\t", &" ".repeat(self.tab_width));
        let len = line.chars().count();
        let mut result = vec![];
        let mut registry: HashMap<usize, (usize, &TokenRef)> = HashMap::default();
        // Create token registry for this line
        for token in self.line_ref[y].iter().map(|t| &self.tokens[*t]) {
            match token {
                // Register bounded token
                TokenRef::Bounded { start, end, .. } => {
                    let start = if start.y != y { 0 } else { self.atoms[start.y][start.x].x.start };
                    let end = end.clone()
                        .and_then(|end| Some(if end.y != y { len } else { self.atoms[end.y][end.x].x.end }))
                        .unwrap_or(len);
                    registry.insert(start, (end, token));
                }
                // Register keyword token
                TokenRef::Keyword { atom, .. } => {
                    //println!("{:?}", self.atoms);
                    let start = self.atoms[atom.y][atom.x].x.start;
                    let end = self.atoms[atom.y][atom.x].x.end;
                    registry.insert(start, (end, token));
                }
            }
        }
        // Process tokens into TokOpt format
        let mut x = 0;
        while x < len {
            if let Some((end, TokenRef::Bounded { name, .. } | TokenRef::Keyword { name, .. })) = registry.get(&x) {
                // Process token
                let text = line.chars().skip(x).take(end - x).collect::<String>();
                result.push(TokOpt::Some(text, name.clone()));
                x = *end;
            } else {
                // Process plain text
                if let Some(TokOpt::None(ref mut s)) = result.last_mut() {
                    s.push(line.chars().nth(x).unwrap());
                } else {
                    result.push(TokOpt::None(line.chars().nth(x).unwrap().to_string()));
                }
                x += 1;
            }
        }
        result
    }

    /// Whenever a character is deleted or inserted on a line,
    /// call this function to update any tokens.
    pub fn edit(&mut self, y: usize, line: &String) {
        let old_atoms = self.atoms[y].clone();
        // Update the atoms on this line
        self.atoms[y] = self.atomize(line);
        // Determine whether tokenisation is necessary by checking atomic changes
        if self.retokenization_needed(&old_atoms, &self.atoms[y]) {
            self.tokenize();
        }
    }

    /// Takes two lists of atoms and determines if retokenization is required in the first place
    /// This method will ignore index (as this is expected to change when editing)
    /// Has been shown to make editing events 500x faster to apply (where no atoms are modified)
    fn retokenization_needed(&self, old: &Vec<Atom>, new: &Vec<Atom>) -> bool {
        // List lengths differ => atoms have been added or deleted
        if old.len() != new.len() { return true; }
        for (o, n) in old.iter().zip(new) {
            // If there is ever ANY discrepancy between atoms, we must retokenize
            if !(o.name == n.name && o.kind == n.kind && o.tok == n.tok && o.backslashed == n.backslashed) {
                return true;
            }
        }
        return false;
    }

    /// Whenever a line is inserted into the document,
    /// call this function to update any tokens.
    pub fn insert_line(&mut self, y: usize, line: &String) {
        self.atoms.insert(y, self.atomize(line));
        self.tokenize();
    }

    /// Whenever a line is removed from a document,
    /// call this function to update any tokens.
    pub fn remove_line(&mut self, y: usize) {
        self.atoms.remove(y);
        self.tokenize();
    }

    /// This process will turn a line into a vector of atoms
    fn atomize(&self, line: &String) -> Vec<Atom> {
        let mut atoms = vec![];
        // For each atom definition
        for def in &self.atom_def {
            let occurances = find_all(&def.exp, line, self.tab_width);
            // Register all occurances of any atom
            for x in occurances {
                if !x.is_empty() {
                    // Check if there is a lone backslash behind atom (indicating an escape)
                    let backslashed = x.start != 0 && line.chars().nth(x.start - 1) == Some('\\');
                    let escapedescape = x.start != 0 && x.start != 1 && line.chars().nth(x.start - 2) == Some('\\');
                    atoms.push(Atom {
                        kind: def.kind.clone(),
                        name: def.name.clone(),
                        tok: def.tok,
                        backslashed: backslashed && !escapedescape,
                        x,
                    });
                }
            }
        }
        // Order them based on start index
        atoms.sort_by(|a, b| a.x.start.cmp(&b.x.start));
        atoms
    }

    fn tokenize(&mut self) {
        self.tokenize_state = None;
        self.line_ref = vec![];
        self.atoms.iter().enumerate().for_each(|_| self.line_ref.push(vec![]));
        self.tokens = vec![];
        for y in 0..self.atoms.len() {
            self.tokenize_line(y);
        }
    }

    fn tokenize_line(&mut self, y: usize) {
        let line_ref = self.line_ref.get_mut(y).unwrap();
        let mut at_x = 0;
        let atoms = &self.atoms[y];
        for (x, atom) in atoms.iter().enumerate() {
            if atom.x.start < at_x { continue; }
            // Work out if this atom is to be ignored (due to escaping)
            if let Atom { tok: Some(t), backslashed, .. } = atom {
                if self.bounded_def[*t].escapable && *backslashed {
                    continue;
                }
            }
            // Continue tokenising...
            match atom {
                Atom { name, kind: AtomKind::Keyword, .. } => {
                    if self.tokenize_state.is_none() || self.tokenize_interp {
                        self.tokens.push(TokenRef::Keyword {
                            name: name.clone(),
                            atom: Loc { y, x },
                        });
                        line_ref.push(self.tokens.len().saturating_sub(1));
                        at_x = atom.x.end;
                    }
                }
                Atom { name, kind: AtomKind::Start, tok, .. } => {
                    if self.tokenize_interp { continue; }
                    if self.tokenize_state.is_none() {
                        self.tokenize_state = *tok;
                        self.tokens.push(TokenRef::Bounded {
                            name: name.clone(),
                            start: Loc { y, x },
                            end: None,
                        });
                        at_x = atom.x.end;
                    }
                }
                Atom { kind: AtomKind::End, tok, .. } => {
                    if self.tokenize_interp { continue; }
                    if self.tokenize_state == *tok {
                        self.tokenize_state = None;
                        if let TokenRef::Bounded { ref mut end, .. } = self.tokens.last_mut().unwrap() {
                            *end = Some(Loc { y, x });
                            at_x = atom.x.end;
                        }
                        line_ref.push(self.tokens.len().saturating_sub(1));
                    }
                }
                Atom { name, kind: AtomKind::Hybrid, tok, .. } => {
                    if self.tokenize_interp { continue; }
                    if self.tokenize_state.is_none() {
                        // Start registering token
                        self.tokenize_state = *tok;
                        self.tokens.push(TokenRef::Bounded {
                            name: name.clone(),
                            start: Loc { y, x },
                            end: None,
                        });
                        at_x = atom.x.end;
                    } else if self.tokenize_state == *tok {
                        // Stop registering token
                        self.tokenize_state = None;
                        if let TokenRef::Bounded { ref mut end, .. } = self.tokens.last_mut().unwrap() {
                            *end = Some(Loc { y, x });
                            at_x = atom.x.end;
                        }
                        line_ref.push(self.tokens.len().saturating_sub(1));
                    }
                }
                Atom { name, kind: AtomKind::InterpolateStart, tok, .. } => {
                    if self.tokenize_state == *tok {
                        // End the current token
                        if let TokenRef::Bounded { ref mut end, .. } = self.tokens.last_mut().unwrap() {
                            *end = Some(Loc { y, x });
                            at_x = atom.x.end;
                        }
                        line_ref.push(self.tokens.len().saturating_sub(1));
                        // Register interpolation
                        self.tokenize_interp = true;
                    }
                }
                Atom { name, kind: AtomKind::InterpolateEnd, tok, .. } => {
                    if self.tokenize_state == *tok {
                        // Stop interpolating
                        self.tokenize_interp = false;
                        // Resume capturing the outer token
                        self.tokens.push(TokenRef::Bounded {
                            name: name.clone(),
                            start: Loc { y, x },
                            end: None,
                        });
                        at_x = atom.x.end;
                    }
                }
            }
            if self.tokenize_state.is_some() {
                line_ref.push(self.tokens.len().saturating_sub(1));
            }
        }
        if self.tokenize_state.is_some() {
            line_ref.push(self.tokens.len().saturating_sub(1));
        }
        line_ref.dedup();
    }
}

/// This will find all occurances of a string in a document (and return character indices)
pub fn find_all(exp: &Regex, target: &String, tab_width: usize) -> Vec<Range<usize>> {
    exp.captures_iter(&target)
        // Get last capture
        .map(|c| c.iter().filter_map(|c| c).collect::<Vec<_>>())
        .map(|mut c| c.pop().unwrap())
        // Extract end and start values
        .map(|m| (m.start(), m.end()))
        // Gather character counts
        .map(|(s, e)| (char_len(&target[0..s], tab_width), char_len(&target[s..e], tab_width)))
        // Create character range
        .map(|(s, l)| s..(s + l))
        .collect()
}

/// Finds out the length of a string in characters, being mindful of tab width
fn char_len(string: &str, tab_width: usize) -> usize {
    string.chars().count() + string.matches('\t').count() * tab_width.saturating_sub(1)
}

/// Trim utility function to trim down a line of tokens to offset text
pub fn trim(input: &[TokOpt], start: usize) -> Vec<TokOpt> {
    let mut opt: Vec<TokOpt> = input.iter().cloned().collect();
    let mut total_width = 0;
    for i in &opt {
        let (TokOpt::Some(txt, _) | TokOpt::None(txt)) = i;
        total_width += txt.len();
    }
    let width = total_width.saturating_sub(start);
    while total_width != width {
        if let Some(token) = opt.get_mut(0) {
            token.nibble();
            total_width -= 1;
            if token.is_empty() {
                opt.remove(0);
            }
        } else {
            break;
        }
    }
    opt
}

/// Function to obtain a syntax highlighter based on a file extension
pub fn from_extension(ext: &str, tab_width: usize) -> Option<Highlighter> {
    let mut result = Highlighter::new(tab_width);
    match ext.to_lowercase().as_str() {
        "rs" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded("string", "r#\"", "\"#", true);
            result.bounded("string", "r\"", "\"", true);
            result.bounded("string", "#\"", "\"#", true);
            result.bounded("string", "\"", "\"", true);
            result.bounded("attribute", r"\#\[", r"\]", false);
            result.bounded("attribute", r"\#!\[", r"\]", false);
            result.keyword("namespace", "([a-z_][A-Za-z0-9_]*)::");
            add_keywords(&mut result, &[
                "as", "break", "const", "continue", "crate", "else", "enum", "extern",
                "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
                "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait",
                "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
                "abstract", "become", "box", "do", "final", "macro", "override", "priv",
                "typeof",  "unsized", "virtual", "yield", "try", "'static", "u8", "u16", "u32",
                "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
                "f64", "String", "Vec", "str", "Some", "bool", "None", "Box", "Result",
                "Option", "Ok", "Err", "Self", "std"
            ]);
            bulk_add(&mut result, "operator", &[
                "=", "\\+", "\\-", "\\*", "[^/](/)[^/]", "\\+=", "\\-=", 
                "\\*=", "\\\\=", "==", "!=", "\\?", ">=", "<=", "<", ">",
            ]);
            bulk_add(&mut result, "character", &[
                r"'[^\\]'", "'\\\\.'"
            ]);
            bulk_add(&mut result, "digit", &[
                "\\b(\\d+.\\d+|\\d+)", "\\b(\\d+.\\d+(?:f32|f64))"
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b"
            ]);
            bulk_add(&mut result, "function", &[
                "fn\\s+([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "\\.([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "([a-z_][A-Za-z0-9_]*)\\s*\\(",
            ]);
            bulk_add(&mut result, "struct", &[
                "(?:trait|enum|struct|impl)\\s+([A-Z][A-Za-z0-9_]*)\\s*",
                "impl(?:<.*?>|)\\s+([A-Z][A-Za-z0-9_]*)",
                "([A-Z][A-Za-z0-9_]*)::",
                "([A-Z][A-Za-z0-9_]*)\\s*\\(",
                "impl.*for\\s+([A-Z][A-Za-z0-9_]*)",
                "::\\s*([a-z_][A-Za-z0-9_]*)\\s*\\(",
            ]);
            bulk_add(&mut result, "macro", &[
                "\\b([a-z_][a-zA-Z0-9_]*!)",
                "(\\$[a-z_][A-Za-z0-9_]*)",
            ]);
            bulk_add(&mut result, "reference", &[
                "&", "&str", "&mut", "&self",
                "&i8", "&i16", "&i32", "&i64", "&i128", "&isize",
                "&u8", "&u16", "&u32", "&u64", "&u128", "&usize",
                "&f32", "&f64",
            ]);
        }
        "asm" | "s" => {
            result.keyword("function", "([a-zA-Z_]+)\\:$");
            result.keyword("comment", "(;.*)$");
            result.keyword("digit", "\\b((?:0x)?\\d+.\\d+|\\d+)");
            result.bounded("string", "\"", "\"", true);
            add_keywords_case_indep(&mut result, &[
                "mov", "add", "sub", "jmp", "call", "ret", "bss",
                "data", "text", "section", "globl", "extern", "db",
                "eax", "ebx", "ecx", "edx", "esp", "ebp", "int", "xor",
                "imul", "inc", "jle", "cmp", "global", "section", "resb"
            ]);
        }
        "py" | "pyw" => {
            result.keyword("comment", "(#.*)$");
            result.bounded("string", "\"\"\"", "\"\"\"", true);
            result.bounded("string", "\'\'\'", "\'\'\'", true);
            result.bounded("string", "b\"", "\"", true);
            result.bounded("string", "r\"", "\"", true);
            result.bounded_interp("string", "f\"", "\"", "\\{", "\\}", true);
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "b\'", "\'", true);
            result.bounded("string", "r\'", "\'", true);
            result.bounded_interp("string", "f\'", "\'", "\\{", "\\}", true);
            result.bounded("string", "\'", "\'", true);
            add_keywords(&mut result, &[
                "and", "as", "assert", "break", "class", "continue",
                "def", "del", "elif", "else", "except", "exec",
                "finally", "for", "from", "global", "if", "import",
                "in", "is", "lambda", "not", "or", "pass", "print",
                "raise", "return", "try", "while", "with", "yield",
                "str", "bool", "int", "tuple", "list", "dict", "tuple",
                "len", "None", "input", "type", "set", "range", "enumerate",
                "open", "iter", "min", "max", "dir", "self", "isinstance",
                "help", "next", "super",
            ]);
            result.keyword("attribute", "@.*$");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "class\\s+([A-Za-z0-9_]+)");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"(\s//\s)", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(True)\\b", "\\b(False)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "def\\s+([a-z_][A-Za-z0-9_]*)",
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
        }
        "rb" | "ruby" => {
            result.keyword("comment", "(#.*)$");
            result.bounded("comment", "=begin", "=end", false);
            result.bounded_interp("string", "\"", "\"", "#\\{", "\\}", true);
            result.bounded("string", "\'", "\'", true);
            result.keyword("string", r"(\:[a-zA-Z_]+)");
            add_keywords(&mut result, &[
                "__ENCODING__", "__LINE__", "__FILE__", "BEGIN", "END", "alias", "and",
                "begin", "break", "case", "class", "def", "defined?", "do", "else",
                "elsif", "end", "ensure", "for", "if", "in", "module", "next",
                "nil", "not", "or", "redo", "rescue", "retry", "return", "self", "super",
                "then", "undef", "unless", "until", "when", "while", "yield",
                "extend", "include", "attr_reader", "attr_writer", "attr_accessor",
            ]);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "class\\s+([A-Za-z0-9_]+)");
            bulk_add(&mut result, "operator", &[
                "!!", "=", "\\+", "\\-", "\\*", "[^/](/)[^/]", "\\+=", "\\-=", 
                "\\*=", "\\\\=", "==", "!=", "\\?", ">=", "<=", "<", ">",
                "&&", "\\|\\|", "!", "&", "\\|", "\\^", "%",
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "def\\s+([a-z_][A-Za-z0-9_]*)",
                "^\\s*([a-z_][A-Za-z0-9_]*)\\s+[^=]",
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
        }
        "cgi" | "pm" => {
            result.keyword("comment", "(#.*)$");
            result.bounded_interp("string", "\"", "\"", "#\\{", "\\}", true);
            result.bounded("string", "(?:m|s)/", "/", true);
            result.bounded("string", "\'", "\'", true);
            result.keyword("string", r"(\:[a-zA-Z_]+)");
            add_keywords(&mut result, &[
                "if", "else", "elsif", "unless", "while", "for", "foreach", "until",
                "do", "next", "last", "goto", "return", "sub", "my", "local", "our",
                "package", "use", "require", "import", "undef", "and", "or", "not",
                "eq", "ne", "lt", "le", "gt", "ge", "cmp", "qw",
                "scalar", "array", "hash", "undef", "undef", "ref", "bless",
                "glob", "filehandle", "code", "regexp", "integer", "float", "string",
                "boolean", "reference", "die"
            ]);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)");
            bulk_add(&mut result, "operator", &[
                "!!", "=", "\\+", "\\-", "\\*", "[^/](/)[^/]", "\\+=", "\\-=", 
                "\\*=", "\\\\=", "==", "!=", "\\?", ">=", "<=", "<", ">", "\\$",
                "&&", "\\|\\|", "!", "&", "\\|", "\\^", "(?:\\\\)?%", "\\\\@",
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "sub\\s+([a-z_][A-Za-z0-9_]*)",
                "^\\s*([a-z_][A-Za-z0-9_]*)\\s+[^=]",
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
        }
        "lua" => {
            result.bounded("comment", r"--\[\[", r"--\]\]", false);
            result.keyword("comment", "(--.*)$");
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "\'", "\'", true);
            result.bounded("string", "\\[\\[", "\\]\\]", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
            bulk_add(&mut result, "function", &[
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(and)", r"(or)", r"(not)",
            ]);
            add_keywords(&mut result, &[
                "break", "do", "else", "elseif", "end", "false", "for", "function",
                "if", "in", "local", "nil", "repeat", "return", "then", "true",
                "until", "while", "self",
            ]);
        }
        "r" | "rproj" => {
            result.keyword("comment", "(#.*)$");
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "\'", "\'", true);
            bulk_add(&mut result, "boolean", &[
                "\\b(FALSE)\\b", "\\b(TRUE)\\b",
            ]);
            add_keywords(&mut result, &[
                "if", "else", "repeat", "while", "function", "for", "in",
                "next", "break", "TRUE", "FALSE", "NULL", "Inf", "NaN",
                "NA", "NA_integer_", "NA_real_", "NA_complex_", "NA_character_",
                r"\.\.\.",
            ]);
            result.keyword("attribute", "@.*$");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "class\\s+([A-Za-z0-9_]+)");
            bulk_add(&mut result, "operator", &[
                r"<-", r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"(\s//\s)", r"(&)",
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", r"(\$)", r"(|)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)", r"(\?)", 
            ]);
            bulk_add(&mut result, "function", &[
                "def\\s+([a-z_][A-Za-z0-9_]*)",
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
        }
        "go" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "`", "`", true);
            bulk_add(&mut result, "character", &[
                r"'[^\\]'", "'\\\\.'"
            ]);
            add_keywords(&mut result, &[
                "break", "case", "chan", "const", "continue", "default", "defer",
                "else", "fallthrough", "for", "func", "go", "goto", "if", "import",
                "interface", "map", "package", "range", "return", "select", "struct",
                "switch", "type", "var",
                "bool", "byte", "complex64", "complex128", "error", "float32", "float64",
                "int", "int8", "int16", "int32", "int64", "rune", "string",
            ]);
            bulk_add(&mut result, "operator", &[
                ":=", "=", "\\+", "\\-", "\\*", "[^/](/)[^/]", "\\+=", "\\-=", 
                "\\*=", "\\\\=", "==", "!=", "\\?", ">=", "<=", "<", ">",
            ]);
            bulk_add(&mut result, "digit", &[
                "\\b(\\d+.\\d+|\\d+)", "\\b(\\d+.\\d+(?:f32|f64))"
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b"
            ]);
            bulk_add(&mut result, "function", &[
                "func\\s+([A-Za-z0-9_]+)\\s*\\(",
                "\\.([A-Za-z0-9_]+)\\s*\\(",
                "([A-Za-z0-9_]+)\\s*\\(",
            ]);
            bulk_add(&mut result, "reference", &[
                "&"
            ]);
        }
        "js" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "//.*$");
            result.bounded("string", "r\"", "\"", true);
            result.bounded("string", "f\"", "\"", true);
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "r\'", "\'", true);
            result.bounded("string", "f\'", "\'", true);
            result.bounded("string", "\'", "\'", true);
            result.bounded_interp("string", "r`", "`", "\\$\\{", "\\}", true);
            result.bounded_interp("string", "f`", "`", "\\$\\{", "\\}", true);
            result.bounded_interp("string", "`", "`", "\\$\\{", "\\}", true);
            result.bounded("string", "/", "/", true);
            add_keywords(&mut result, &[
                "abstract", "arguments", "await", "boolean", "break", "byte",
                "case", "catch", "char", "class", "const", "continue", "debugger",
                "default", "delete", "do", "double", "else", "enum", "eval",
                "export", "extends", "final", "finally", "float", "for", "of",
                "function", "goto", "if", "implements", "import", "in", "instanceof",
                "int", "interface", "let", "long", "native", "new", "null", "package",
                "private", "protected", "public", "return", "short", "static",
                "super", "switch", "synchronized", "this", "throw", "throws",
                "transient", "try", "typeof", "var", "void", "volatile", "console",
                "while", "with", "yield", "undefined", "NaN", "-Infinity", "Infinity",
            ]);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "class\\s+([A-Za-z0-9_]+)");
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "function\\s+([a-z_][A-Za-z0-9_]*)",
                "\\b([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "\\.([a-z_][A-Za-z0-9_]*)\\s*",
            ]);
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
        }
        "ts" | "tsx" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "//.*$");
            result.bounded("string", "r\"", "\"", true);
            result.bounded("string", "f\"", "\"", true);
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "r\'", "\'", true);
            result.bounded("string", "f\'", "\'", true);
            result.bounded("string", "\'", "\'", true);
            result.bounded_interp("string", "r`", "`", "\\$\\{", "\\}", true);
            result.bounded_interp("string", "f`", "`", "\\$\\{", "\\}", true);
            result.bounded_interp("string", "`", "`", "\\$\\{", "\\}", true);
            result.bounded("string", "/", "/", true);
            add_keywords(&mut result, &[
               "abstract", "any", "as", "asserts", "boolean", "break", "case", "catch",
               "class", "const", "constructor", "continue", "debugger", "declare",
                "default", "delete", "do", "else", "enum", "export", "extends", "false",
                "finally", "for", "from", "function", "get", "if", "implements", "import",
                "in", "infer", "instanceof", "interface", "is", "keyof", "let", "module",
                "namespace", "never", "new", "null", "number", "object", "package", "private",
                "protected", "public", "readonly", "require", "global", "return", "set",
                "static", "string", "super", "switch", "symbol", "this", "throw", "true",
                "try", "type", "typeof", "undefined", "unique", "unknown", "var", "void",
                "while", "with", "yield"
            ]);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "class\\s+([A-Za-z0-9_]+)");
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "function\\s+([a-z_][A-Za-z0-9_]*)",
                "\\b([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "\\.([a-z_][A-Za-z0-9_]*)\\s*",
            ]);
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
        }
        "dart" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "//.*$");
            result.bounded("string", "\"\"\"", "\"\"\"", true);
            result.bounded("string", "\'\'\'", "\'\'\'", true);
            result.bounded_interp("string", "\"", "\"", "\\$\\{", "\\}", true);
            result.bounded("string", "\'", "\'", true);
            add_keywords(&mut result, &[
                "abstract", "as", "assert", "async", "await", "break", "case", "catch",
                "class", "const", "continue", "covariant", "default", "deferred", "do",
                "dynamic", "else", "enum", "export", "extends", "extension", "external",
                "factory", "false", "final", "finally", "for", "Function", "get", "hide",
                "if", "implements", "import", "in", "inout", "interface", "is", "late",
                "library", "mixin", "new", "null", "on", "operator", "out", "part",
                "required", "rethrow", "return", "set", "show", "static", "super", "switch",
                "sync", "this", "throw", "true", "try", "typedef", "var", "void", "while",
                "with", "yield", "int", "double", "num", "string",
            ]);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]+)");
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "\\b([a-z_][A-Za-z0-9_]*)(?:<[A-Za-z_]*>)?\\s*\\(",
                "\\.([a-z_][A-Za-z0-9_]*)\\s*",
            ]);
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", "~/",
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)", "\\?",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S", "\\?\\?",
            ]);
        }
        "c" | "h" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded("string", "\"", "\"", true);
            add_keywords(&mut result, &[
                "auto", "break", "case", "char", "const", "continue", "default",
                "do", "double", "else", "enum", "extern", "float", "for", "goto",
                "if", "int", "long", "register", "return", "short", "signed",
                "sizeof", "static", "struct", "switch", "typedef", "union",
                "unsigned", "void", "volatile", "while", "printf", "fscanf",
                "scanf", "fputsf", "exit", "stderr", "malloc", "calloc", "bool",
                "realloc", "free", "strlen", "size_t",
            ]);
            result.keyword("struct", "\\}\\s+([A-Za-z0-9_]+)\\s*");
            result.keyword("attribute", "^\\s*(#.*?)\\s");
            result.keyword("header", "(<.*?>)");
            bulk_add(&mut result, "digit", &[
                "\\b(\\d+.\\d+|\\d+)",
                "\\b(\\d+.\\d+(?:f|))",
            ]);
            bulk_add(&mut result, "character", &[
                r"'[^\\]'", "'\\\\.'"
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "(int|bool|void|char|double|long|short|size_t)\\s+([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "\\b([a-z_][A-Za-z0-9_]*)\\s*\\(",
            ]);
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
        }
        "cpp" | "hpp" | "c++" | "cxx" | "cc" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded("string", "\"", "\"", true);
            add_keywords(&mut result, &[
                "alignas", "alignof", "and", "and_eq", "asm", "auto",
                "bitand", "bitor", "bool", "break", "case", "catch", "char",
                "char8_t", "char16_t", "char32_t", "class", "compl", "concept",
                "const", "consteval", "constexpr", "constinit", "const_cast",
                "continue", "co_await", "co_return", "co_yield", "decltype",
                "default", "delete", "do", "double", "dynamic_cast", "else",
                "enum", "explicit", "export", "extern", "false", "float", "for",
                "friend", "goto", "if", "inline", "int", "long", "mutable",
                "namespace", "new", "noexcept", "not", "not_eq", "nullptr",
                "operator", "or", "or_eq", "private", "protected", "public",
                "register", "reinterpret_cast", "requires", "return", "short",
                "signed", "sizeof", "static", "static_assert", "static_cast",
                "struct", "switch", "template", "this", "thread_local",
                "throw", "true", "try", "typedef", "typeid", "typename",
                "union", "unsigned", "using", "virtual", "void", "volatile",
                "wchar_t", "while", "xor", "xor_eq", "std", "string",
            ]);
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)");
            result.keyword("attribute", "^\\s*(#[a-zA-Z_]+)\\s*");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
                r"(|)", r"(&)", r"(^)", r"(~)",
            ]);
            result.keyword("header", "(<.*?>)");
            bulk_add(&mut result, "digit", &[
                "\\b(\\d+.\\d+|\\d+)",
                "\\b(\\d+.\\d+(?:f|))",
            ]);
            bulk_add(&mut result, "character", &[
                r"'[^\\]'", "'\\\\.'"
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "(int|bool|void|char|double|long|short|size_t)\\s+([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "\\b([a-z_][A-Za-z0-9_]*)\\s*\\(",
            ]);
        }
        "cs" | "csproj" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded("string", "\"", "\"", true);
            add_keywords(&mut result, &[
                "abstract", "as", "base", "bool", "break", "byte", "case", "catch",
                "char", "checked", "class", "const", "continue", "decimal", "default",
                "delegate", "do", "double", "else", "enum", "event", "explicit",
                "extern", "false", "finally", "fixed", "float", "for", "foreach",
                "goto", "if", "implicit", "in", "int", "interface", "internal",
                "is", "lock", "long", "namespace", "new", "null", "object", "operator",
                "out", "override", "params", "private", "protected", "public", "readonly",
                "ref", "return", "sbyte", "sealed", "short", "sizeof", "stackalloc",
                "static", "string", "struct", "switch", "this", "throw", "true", "try",
                "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort", "using",
                "using", "static", "virtual", "void", "volatile", "while", "add",
                "alias", "ascending", "async", "await", "by", "descending", "dynamic",
                "equals", "from", "get", "global", "group", "into", "join", "let",
                "nameof", "on", "orderby", "partial", "remove", "select", "set",
                "unmanaged", "value", "var", "when", "where", "with", "yield"
            ]);
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
                r"(|)", r"(&)", r"(^)", r"(~)",
            ]);
            bulk_add(&mut result, "digit", &[
                "\\b(\\d+.\\d+|\\d+)",
                "\\b(\\d+.\\d+(?:f|m|))",
            ]);
            bulk_add(&mut result, "character", &[
                r"'[^\\]'", "'\\\\.'"
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b",
            ]);
            bulk_add(&mut result, "function", &[
                "(int|bool|void|char|double|long|short|size_t)\\s+([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "\\b([a-z_][A-Za-z0-9_]*)\\s*\\(",
            ]);
        }
        "swift" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded_interp("string", "#\"", "\"#", "\\\\#?\\(", "\\)", true);
            result.bounded("string", "\"\"\"", "\"\"\"", true);
            result.bounded_interp("string", "\"", "\"", "\\\\\\(", "\\)", true);
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
            add_keywords(&mut result, &[
                "associatedtype", "class", "deinit", "enum", "extension",
                "fileprivate", "func", "import", "init", "inout", "internal",
                "let", "open", "operator", "private", "protocol", "public",
                "static", "struct", "subscript", "typealias", "var", "break",
                "case", "continue", "default", "defer", "do", "else", "fallthrough",
                "for", "guard", "if", "in", "repeat", "return", "switch", "where",
                "while", "as", "catch", "throw", "try", "Any", "false", "is", "nil",
                "super", "self", "Self", "true", "associativity", "convenience",
                "dynamic", "didSet", "final", "get", "infix", "indirect", "lazy",
                "left", "mutating", "none", "nonmutating", "optional", "override",
                "postfix", "precedence", "prefix", "Protocol", "required", "right",
                "set", "Type", "unowned", "weak", "willSet", "Int", "String", 
                "Double", "Optional", "endif",
            ]);
            bulk_add(&mut result, "operator", &[
                "=", "\\+", "\\-", "\\*", "[^/](/)[^/]", "\\+=", "\\-=", 
                "\\*=", "\\\\=", "==", "!=", "\\?", ">=", "<=", "<", ">", "!"
            ]);
            bulk_add(&mut result, "digit", &[
                "\\b(\\d+.\\d+|\\d+)", "\\b(\\d+.\\d+(?:f32|f64))"
            ]);
            bulk_add(&mut result, "boolean", &[
                "\\b(true)\\b", "\\b(false)\\b"
            ]);
            bulk_add(&mut result, "function", &[
                "func\\s+([a-z_][A-Za-z0-9_]*)\\s*(?:\\(|<)",
                "\\.([a-z_][A-Za-z0-9_]*)\\s*\\(",
                "([a-z_][A-Za-z0-9_]*)\\s*\\(",
            ]);
        }
        "json" => {
            result.bounded("string", "\"", "\"", true);
            result.keyword("keyword", r"\b(null)\b");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("boolean", "\\b(true|false)\\b");
        }
        "kt" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded("string", "\"\"\"", "\"\"\"", true);
            result.bounded("string", "\"", "\"", true);
            result.keyword("attribute", r"@\w+");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
            add_keywords(&mut result, &[
                "abstract", "actual", "annotation", "companion", "constructor",
                "enum", "external", "expect", "final", "fun", "inline", "inner",
                "interface", "internal", "private", "protected", "public", "sealed",
                "suspend", "tailrec", "vararg", "as", "break", "class", "continue",
                "do", "else", "false", "for", "if", "in", "is", "null", "object", "infix",
                "package", "return", "super", "this", "throw", "true", "try", "data",
                "typealias", "typeof", "val", "when", "while", "var", "operator", "override",
            ]);
            bulk_add(&mut result, "function", &[
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\{",
            ]);
        }
        "class" | "java" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded("string", "\"", "\"", true);
            result.keyword("attribute", r"@\w+");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
            add_keywords(&mut result, &[
                "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
                "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
                "finally", "float", "for", "if", "goto", "implements", "import", "instanceof", "int",
                "interface", "long", "native", "new", "package", "private", "protected", "public",
                "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
                "throw", "throws", "transient", "try", "var", "void", "volatile", "while", "null",
            ]);
            bulk_add(&mut result, "function", &[
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
        }
        "vb" => {
            result.keyword("comment", "('.*)$");
            result.bounded("string", "\"", "\"", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            bulk_add(&mut result, "function", &[
                "\\b([A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
            add_keywords(&mut result, &[
                "AddHandler", "AddressOf", "Alias", "And", "AndAlso", "Ansi", "As", "Assembly",
                "Auto", "Boolean", "ByRef", "Byte", "ByVal", "Call", "Case", "Catch", "CBool",
                "CByte", "CChar", "CDate", "CDec", "CDbl", "Char", "CInt", "Class", "CLng",
                "CObj", "Const", "CShort", "CSng", "CStr", "CType", "Date", "Decimal", "Declare",
                "Default", "Delegate", "Dim", "DirectCast", "Do", "Double", "Each", "Else",
                "ElseIf", "End", "Enum", "Erase", "Error", "Event", "Exit", "False", "Finally",
                "For", "Friend", "Function", "Get", "GetType", "GoSub", "GoTo", "Handles",
                "If", "Implements", "Imports", "In", "Inherits", "Integer", "Interface", "Is",
                "IsNot", "Let", "Lib", "Like", "Long", "Loop", "Me", "Mod", "Module", "MustInherit",
                "MustOverride", "MyBase", "MyClass", "Namespace", "Narrowing", "New", "Next",
                "Not", "Nothing", "NotInheritable", "NotOverridable", "Object", "Of", "On",
                "Operator", "Option", "Optional", "Or", "OrElse", "Out", "Overloads", "Overridable",
                "Overrides", "ParamArray", "Partial", "Private", "Property", "Protected", "Public",
                "RaiseEvent", "ReadOnly", "ReDim", "REM", "RemoveHandler", "Resume", "Return",
                "SByte", "Select", "Set", "Shadows", "Shared", "Short", "Single", "Static", "Step",
                "Stop", "String", "Structure", "Sub", "SyncLock", "Then", "Throw", "To", "True",
                "Try", "TryCast", "TypeOf", "UInteger", "ULong", "UShort", "Using", "Variant",
                "Wend", "When", "While", "Widening", "With", "WithEvents", "WriteOnly", "Xor", "Console",
            ]);
        }
        "m" => {
            result.bounded("comment", "%\\{", "%\\}", true);
            result.keyword("comment", "(%.*)$");
            result.bounded("string", "\'", "\'", true);
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
            add_keywords(&mut result, &[
                "break", "case", "catch", "classdef", "continue", "else", "elseif",
                "end", "for", "function", "global", "if", "otherwise", "parfor", "persistent",
                "return", "spmd", "switch", "try", "while", "inf", "nan",
                "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64",
                "single", "double", "char", "string", "cell", "struct", "table", "datetime",
                "properties", "NaN", "max", "min", "length", "sort", "sum", "prod", "mode",
                "median", "mean", "std", "pi", "randi", "randn", "rand", "clf", "shg", "close",
                "path", "addpath", "rmpath", "cd", "grid", "on", "axis", "square", "equal", "off",
                "hold", "help", "doc", "lookfor", "profile", "viewer", "clc", "diary", "ctrl-c",
                "who", "whos", "clear", "load", "format", "short", "long", "bank",
            ]);
            bulk_add(&mut result, "function", &[
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
        }
        "php" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.keyword("comment", "(#.*)$");
            result.bounded_interp("string", "\"", "\"", "\\{", "\\}", true);
            result.bounded_interp("string", "\"", "\"", "\\$\\{", "\\}", true);
            result.bounded("string", "\'", "\'", true);
            result.keyword("boolean", "\\b(true|false|TRUE|FALSE)\\b");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
            bulk_add(&mut result, "function", &[
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
            add_keywords(&mut result, &[
                "__halt_compiler", "abstract", "and", "array", "as", "break", "callable", "case",
                "catch", "class", "clone", "const", "continue", "declare", "default", "die", "do",
                "echo", "else", "elseif", "empty", "enddeclare", "endfor", "endforeach", "endif",
                "endswitch", "endwhile", "eval", "exit", "extends", "final", "finally", "for",
                "foreach", "function", "global", "goto", "if", "implements", "include", "include_once",
                "instanceof", "insteadof", "interface", "isset", "list", "namespace", "new", "or",
                "print", "private", "protected", "public", "require", "require_once", "return",
                "static", "switch", "throw", "trait", "try", "unset", "use", "var", "while", "xor",
                "__CLASS__", "__DIR__", "__FILE__", "__FUNCTION__", "__LINE__", "__METHOD__",
                "__NAMESPACE__", "__TRAIT__", "null",
            ]);
            result.keyword("keyword", r"<\?php");
            result.keyword("keyword", r"\?>");
            bulk_add(&mut result, "operator", &[
                r"(->)", r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", r"(\?)",
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)", r"(\$)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S", r"(\.)",
            ]);
        }
        "scala" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.keyword("comment", "(//.*)$");
            result.bounded_interp("string", "f\"", "\"", "\\$\\{", "\\}", true);
            result.bounded_interp("string", "s\"", "\"", "\\$\\{", "\\}", true);
            result.bounded("string", "\"\"\"", "\"\"\"", true);
            result.bounded("string", "raw\"", "\"", true);
            result.bounded("string", "\"", "\"", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            bulk_add(&mut result, "character", &[
                r"'[^\\]'", "'\\\\.'"
            ]);
            result.keyword("boolean", "\\b(true|false)\\b");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", 
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S",
            ]);
            add_keywords(&mut result, &[
                "abstract", "case", "catch", "class", "def", "do", "else", "extends", "false",
                "final", "finally", "for", "forSome", "if", "implicit", "import", "lazy", "macro",
                "match", "new", "null", "object", "override", "package", "private", "protected",
                "return", "sealed", "super", "this", "throw", "trait", "try", "true", "type",
                "val", "var", "while", "with", "yield",
                "Boolean", "Byte", "Char", "Double", "Float", "Int", "Long", "Short", "String",
                "Unit", "Any", "AnyVal", "AnyRef", "Nothing", "Null",
                "foreach", "map", "println", "to", "by"
            ]);
            bulk_add(&mut result, "function", &[
                "\\.([a-z_][A-Za-z0-9_\\?!]*)\\s*",
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
        }
        "pl" | "prolog" => {
            result.keyword("comment", "(\\%.*)$");
            result.bounded("string", "\"", "\"", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("struct", "\\b([A-Z][A-Za-z0-9_]*)\\b");
            add_keywords_no_boundary(&mut result, &[
                ":-", "\\,", "\\.", ";", "\\->", "\\+", "=", "is", "not", "fail", "!",
                "repeat", "call", "cut", "assert", "asserta", "assertz", "retract", "abolish",
                "dynamic", "consult", "listing", "op", "assertions", "clauses", "predicate",
                "query", "rule", "fact", "variable", "atom", "number", "list", "compound",
                "ground", "callable",
                "atom", "number", "integer", "float", "variable", "list", "compound"
            ]);
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s",
                r"(<)", r"(>)",
            ]);
            bulk_add(&mut result, "function", &[
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
            
        }
        "hs" => {
            result.keyword("comment", "(\\-\\-.*)$");
            result.bounded("comment", "\\{-", "-\\}", true);
            result.bounded("string", "\"", "\"", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("boolean", "\\b(True|False)\\b");
            bulk_add(&mut result, "character", &[
                r"'[^\\]'", "'\\\\.'"
            ]);
            bulk_add(&mut result, "operator", &[
                "->", "\\$", "`.*`", "<-", "<", ">", "&&", "\\|\\|", "\\\\", "\\:",
                "=",
                r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s",
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(/=)", "!", "\\.", "\\|",
                r"(==)", r"(!=)", r"(>=)", r"(<=)", "_",
                r"(<<)", r"(>>)", r"(!)\S", "\\band\\b", "\\bor\\b", "\\bnot\\b",
            ]);
            add_keywords(&mut result, &[
                "module", "import", "as", "qualified", "hiding", "do", "case", "of", "let", "in",
                "if", "then", "else", "data", "type", "newtype", "deriving", "class", "instance",
                "where", "foreign", "export", "ccall", "stdcall", "capi", "prim", "safe", "unsafe",
                "otherwise", "head", "tail", "last", "init", "null", "length", "return",
                "map", "filter", "foldl", "foldr", "zip", "zipWith", "take", "drop", "reverse",
                "concat", "concatMap", "maximum", "minimum", "elem", "notElem", "sum", "array",
                "product", "scanl", "scanr", "replicate", "cycle", "repeat", "iterate", "fst", "snd",
                "id", "Maybe", "Either", "Bool", "Char", "String", "putStrLn", "getLine", "Just", "Nothing", "for",
                "Int", "Integer", "Float", "Double", "Ordering", "IO", "Functor", "Applicative", "Monad"
            ]);
            result.keyword("function", "^[a-z][a-zA-Z0-9]*");
        }
        "css" => {
            result.bounded("comment", r"/\*", r"\*/", false);
            result.bounded("string", "\"", "\"", true);
            add_keywords(&mut result, &[
                "from", "to", "rotate", "none"
            ]);
            result.keyword("digit", r"\#[0-9a-fA-F]+");
            result.keyword("digit", "((?:\\d+.\\d+|\\d+)(?:%|deg|px|em|rem)?)");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("attribute", r"\.[a-zA-Z0-9\-]*");
            result.keyword("attribute", r"\:[a-zA-Z0-9\-]*");
            result.keyword("attribute", r"\::[a-zA-Z0-9\-]*");
            result.keyword("attribute", r"@\w+");
            add_keywords(&mut result, &[
                "a", "abbr", "address", "area", "article", "aside", "audio", "b", 
                "base", "bdi", "bdo", "blockquote", "body", "br", "button", "canvas", 
                "caption", "cite", "code", "col", "colgroup", "data", "datalist", 
                "dd", "del", "details", "dfn", "dialog", "div", "dl", "dt", "em", 
                "embed", "fieldset", "figcaption", "figure", "footer", "form", 
                "h1", "h2", "h3", "h4", "h5", "h6", "head", "header", "hgroup", "hr", 
                "html", "i", "iframe", "img", "input", "ins", "kbd", "label", "legend", 
                "li", "link", "main", "map", "mark", "meta", "meter", "nav", "noscript", 
                "object", "ol", "optgroup", "option", "output", "p", "param", "picture", 
                "pre", "progress", "q", "rb", "rp", "rt", "rtc", "ruby", "s", "samp", 
                "script", "section", "select", "slot", "small", "source", "span", "strong", 
                "style", "sub", "summary", "sup", "table", "tbody", "td", "template", 
                "textarea", "tfoot", "th", "thead", "time", "title", "tr", "track", 
                "u", "ul", "var", "video", "wbr", "svg",
            ]);
            add_keywords(&mut result, &[
                "-webkit-touch-callout", "-webkit-user-select", "-moz-user-select", "-ms-user-select",
                "user-select", "transform", "border-radius", "border-right", "border-left",
                "border-top", "border-bottom", "border", "content", "display", "height", "width", "margin-top",
                "margin-bottom", "margin-left", "margin-right", "margin", "pointer-events", "position", "top", "transform-origin",
                "-moz-appearance", "-webkit-appearance", "cursor", "flex-grow", "flex-shrink", "font-size",
                "max-height", "max-width", "min-height", "min-width", "outline", "vertical-align", "background-color",
                "background-image", "background-position", "background-repeat", "background-size", "background",
                "animation", "border-(?:left|right|top|bottom)-color", "border-(?:left|right|top|bottom)-radius",
                "border-(?:left|right|top|bottom)-width", "border-(?:left|right|top|bottom)-style", "align-items", "box-shadow",
                "justify-content", "line-height", "padding", "padding-(?:left|bottom|right|top)", "font-weight",
                "list-style", "box-sizing", "text-align", "bottom", "overflow-x", "overflow-y", "text-rendering", 
                "-moz-osx-font-smoothing", "-webkit-font-smoothing", "text-size-adjust", "font-family", "color", 
                "text-decoration", "font-style", "word-wrap", "white-space", "-webkit-overflow-scrolling", "clear", 
                "float", "overflow", "!important", "text-transform", "clip", "visibility", "border-color", "opacity", 
                "flex-wrap", "border-(?:top|bottom)-(?:left|right)-radius", "z-index", "word-break", "letter-spacing", 
                "text-transform", "resize", "flex-direction", "order", "border-style", "border-width", "text-overflow", 
                "flex-basis", "-ms-overflow-y", "-ms-overflow-x", "transition-duration", "transition-property", 
                "transition-timing-function", "(flex)[^-]", "-webkit-text-decoration-style", "-apple-system", "sans-serif",
                "left", "right", "bottom", "top", "font", "tab-size", "text-shadow"
            ]);
        }
        "html" | "htm" | "xhtml" => {
            result.bounded("comment", "<!--", "-->", false);
            result.bounded("string", "\"", "\"", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("operator", "=");
            bulk_add(&mut result, "tag", &[
                "</", "/>", ">", "<!", "<"
            ]);
            add_html_keywords(&mut result, &[
                "a", "abbr", "address", "area", "article", "aside", "audio", "b", 
                "base", "bdi", "bdo", "blockquote", "body", "br", "button", "canvas", 
                "caption", "cite", "code", "col", "colgroup", "data", "datalist", 
                "dd", "del", "details", "dfn", "dialog", "div", "dl", "dt", "em", 
                "embed", "fieldset", "figcaption", "figure", "footer", "form", 
                "h1", "h2", "h3", "h4", "h5", "h6", "head", "header", "hgroup", "hr", 
                "html", "i", "iframe", "img", "input", "ins", "kbd", "label", "legend", 
                "li", "link", "main", "map", "mark", "meta", "meter", "nav", "noscript", 
                "object", "ol", "optgroup", "option", "output", "p", "param", "picture", 
                "pre", "progress", "q", "rb", "rp", "rt", "rtc", "ruby", "s", "samp", 
                "script", "section", "select", "slot", "small", "source", "span", "strong", 
                "style", "sub", "summary", "sup", "table", "tbody", "td", "template", 
                "textarea", "tfoot", "th", "thead", "time", "title", "tr", "track", 
                "u", "ul", "var", "video", "wbr", "svg",
            ]);
            bulk_add(&mut result, "attribute", &[
                r"([A-Za-z0-9-]+)=",
                r"(class)\s*=", r"(id)\s*=", r"(style)\s*=", r"(src)\s*=", r"(rel)\s*=", r"(type)\s*=", r"(charset)\s*=",
                r"(data-target)\s*=", r"(name)\s*=", r"(href)\s*=", r"(content)\s*=", r"(width)\s*=", r"(height)\s*=",
                r"(aria-label)\s*=", r"(role)\s*=", r"(aria-hidden)\s*=", r"(aria-expanded)\s*=", r"\s*defer\s*",
            ]);
        }
        "md" | "markdown" => {
            result.keyword("heading", "(#.*)$");
            result.keyword("quote", "(>.*)$");
            result.bounded("bold", "\\*\\*", "\\*\\*", true);
            result.bounded("italic", "\\*", "\\*", true);
            result.bounded("strikethrough", "~~", "~~", true);
            result.bounded("image", "!\\[", "\\]", true);
            result.bounded("link", "\\[", "\\]", true);
            result.bounded("math", "\\$\\$", "\\$\\$", false);
            result.bounded("math", "\\$", "\\$", false);
            result.bounded("block", "```", "```", false);
            result.bounded("block", "`", "`", true);
            result.keyword("link", r"\b(?:https?://|www\.)\S+\b");
            result.keyword("linebreak", "^\\s*-{3}");
            result.keyword("list", "[0-9]+\\.");
            result.keyword("list", "^\\s*-");
        }
        "toml" => {
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "\'", "\'", true);
            result.keyword("comment", "(#.*)$");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("table", r"^(\[.*\])");
            bulk_add(&mut result, "digit", &[
                r"(?:=|\[|,)\s*(0x[a-fA-F]+)",
                r"(?:=|\[|,)\s*(0o[0-7]+)",
                r"(?:=|\[|,)\s*(0b[0-1]+)",
                r"(?:=|\[|,)\s*((?:\+|-)?[0-9]+(?:\.[0-9]+)?(?:e|E)(?:\+|-)?[0-9]+)",
                r"(?:=|\[|,)\s*((?:\+|-)?[0-9_]+(?:\.[0-9]+)?)",
            ]);
            add_keywords(&mut result, &["inf", "nan"]);
        }
        "yaml" | "yml" => {
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "\'", "\'", true);
            result.keyword("comment", "(#.*)$");
            result.keyword("key", r"^\s*[ \.a-zA-Z_-]+:");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("tag", "!!(?:bool|int|float|str|timestamp|null|binary)");
            add_keywords(&mut result, &["No", "Yes", "no", "yes", "true", "false", "null"]);

        }
        "csv" => {
            result.keyword("keyword", ",");
        }
        "sh" | "bash" | "bash_profile" | "bashrc" => {
            result.bounded_interp("string", "\"", "\"", "\\$\\(", "\\)", true);
            result.bounded("string", "\'", "\'", true);
            result.bounded("string", "EOF", "EOF", true);
            result.keyword("comment", "(#.*)$");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            bulk_add(&mut result, "operator", &[
                r"(=)", r"(\+)", r"(\-)", r"(\*)", r"(\s/\s)", r"\s(//)\s", 
                r"(%)", r"(\+=)", r"(\-=)", r"(\*=)", r"(\\=)", r"(\{)", r"(\})",
                r"(==)", r"(!=)", r"(>=)", r"(<=)", r"(<)", r"(>)", r"(\$)", r"(\.\.)",
                r"(<<)", r"(>>)", r"(\&\&)", r"(\|\|)", r"(!)\S", r"(\.)", r"(&)",
            ]);
            add_keywords(&mut result, &[
                "if", "then", "else", "elif", "fi", "case", "esac", "for", "while", "until",
                "do", "done", "in", "function", "select", "continue", "break", "return",
                "exit", "source", "declare", "readonly", "local", "export", "ls", "cd", 
                "pwd", "cp", "mv", "rm", "mkdir", "rmdir", "touch", "chmod", "chown", "grep", 
                "awk", "sed", "cat", "head", "tail", "sort", "uniq", "wc", "cut", "paste",
                "find", "tar", "gzip", "gunzip", "zip", "unzip", "ssh", "scp", "rsync", "curl",
                "wget", "ping", "traceroute", "netstat", "ps", "kill", "top", "df", "du",
                "date", "cal", "history", "alias", "source", "source", "exec", "exit", "help",
                "man", "info", "echo", "fgrep", "apropos", "whoami", "python", "bg", "fg", "sleep",
                "jobs", "read", "trap", "clear", "sh", "bash"
            ]);
            bulk_add(&mut result, "function", &[
                "\\b([a-z_][A-Za-z0-9_\\?!]*)\\s*\\(",
            ]);
            
        }
        "sql" | "sqlproj" => {
            result.keyword("comment", "(--.*)$");
            result.bounded("string", "\"", "\"", true);
            result.bounded("string", "\'", "\'", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            bulk_add(&mut result, "operator", &[
                r"\+", "-", r"\*", "/", "%",
                "=", "<>", "!=", "<", ">", "<=", ">=",
                "&", "|", "^", "~", "||", "=",
            ]);
            add_keywords(&mut result, &[
                "ADD", "ALL", "ALTER", "AND", "AS", "ASC", "BETWEEN", "BY", "CASE", "CHECK",
                "COLUMN", "CONSTRAINT", "CREATE", "DATABASE", "DEFAULT", "DELETE", "DESC",
                "DISTINCT", "DROP", "ELSE", "END", "EXISTS", "FOREIGN", "FROM", "FULL",
                "GROUP", "HAVING", "IN", "INDEX", "INNER", "INSERT", "INTO", "IS", "JOIN",
                "LEFT", "LIKE", "LIMIT", "NOT", "NULL", "ON", "OR", "ORDER", "OUTER", "PRIMARY",
                "REFERENCES", "RIGHT", "SELECT", "SET", "TABLE", "TOP", "TRUNCATE", "UNION", 
                "UNIQUE", "UPDATE", "VALUES", "VIEW", "WHERE", "SHOW", "USE", "VARCHAR",
            ]);
        }
        "xml" => {
            result.bounded("comment", "<!--", "-->", false);
            result.bounded("string", "\"", "\"", true);
            result.keyword("digit", "\\b(\\d+.\\d+|\\d+)");
            result.keyword("boolean", "\\b(true|false)\\b");
            result.keyword("operator", "=");
            bulk_add(&mut result, "tag", &[
                "<[A-Za-z0-9_]+>?", "</[A-Za-z0-9_]+>",
                "</", "/>", ">", "<!", "<"
            ]);
            bulk_add(&mut result, "attribute", &[
                r"([A-Za-z0-9-]+)=",
            ]);
        }
        _ => return None,
    }
    Some(result)
}

fn add_html_keywords(h: &mut Highlighter, kw: &[&str]) {
    h.keyword("keyword", &format!(r"(?:<|</|<!)({})\b", kw.join("|")));
}

fn add_keywords_no_boundary(h: &mut Highlighter, kw: &[&str]) {
    h.keyword("keyword", &format!(r"({})", kw.join("|")));
}

fn add_keywords(h: &mut Highlighter, kw: &[&str]) {
    h.keyword("keyword", &format!(r"\b({})\b", kw.join("|")));
}

fn add_keywords_case_indep(h: &mut Highlighter, kw: &[&str]) {
    h.keyword("keyword", &format!(r"\b({})\b", kw.join("|")));
    h.keyword("keyword", &format!(r"\b({})\b", kw.iter().map(|x| x.to_uppercase()).collect::<Vec<_>>().join("|")));
}

fn bulk_add(h: &mut Highlighter, name: &str, kw: &[&str]) {
    h.keyword(name, &format!(r"({})", kw.join("|")));
}
