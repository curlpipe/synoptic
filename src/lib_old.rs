use std::collections::HashMap;
use std::ops::Range;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct RangeLoc {
    pub y: usize,
    pub x: Range<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub y: usize,
    pub x: usize,
}

#[derive(Debug, Clone)]
pub struct Keyword {
    pub kind: String,
    pub loc: RangeLoc,
}

#[derive(Debug)]
pub struct BoundedDef {
    pub name: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternKind {
    Start,
    End,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub of: String,
    pub kind: PatternKind,
    pub loc: RangeLoc,
    pub token: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TokenSpan {
    kind: String,
    // References to patterns
    start: usize,
    end: Option<usize>,
}

#[derive(Debug)]
pub enum Token {
    Start(String),
    Text(String),
    End(String),
}

pub struct Highlighter {
    pub patterns: Vec<Pattern>,
    pub tokens: Vec<TokenSpan>,
    pub keywords: Vec<Vec<Keyword>>,
    pub line_ref: Vec<Vec<usize>>,
    pub bounded_rules: HashMap<String, BoundedDef>,
    pub keyword_rules: HashMap<String, Vec<Regex>>,
    pub modified: Vec<bool>,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            patterns: vec![],
            tokens: vec![],
            keywords: vec![],
            modified: vec![],
            line_ref: vec![],
            bounded_rules: HashMap::default(),
            keyword_rules: HashMap::default(),
        }
    }

    pub fn bounded<S: Into<String>>(&mut self, name: S, start: S, end: S) {
        let (name, start, end) = (name.into(), start.into(), end.into());
        self.bounded_rules.insert(name.clone(), BoundedDef { name, start, end });
    }

    pub fn keyword<S: Into<String>>(&mut self, name: S, pattern: S) {
        let (name, pattern) = (name.into(), pattern.into());
        let regex = Regex::new(&pattern).expect("Invalid regex pattern");
        if let Some(v) = self.keyword_rules.get_mut(&name) {
            v.push(regex);
        } else {
            self.keyword_rules.insert(name, vec![regex]);
        }
    }

    // This will clone each line, potentially optimise using pointers?
    pub fn line(&mut self, idx: usize, contents: &String) -> Vec<Token> {
        // Get the tokens that appear on this line
        let mut tokens: Vec<(RangeLoc, Option<RangeLoc>, TokenSpan)> = self.line_ref[idx].iter()
            // Clone the token
            .map(|i| self.tokens[*i].clone())
            // Attach starting and ending information
            .map(|t| {
                // Obtain the start index from the pattern from the token
                let start_pattern = &self.patterns[t.start];
                let start = start_pattern.loc.clone();
                // Obtain the end pattern from the token
                let end_pattern = t.end.and_then(|t| Some(&self.patterns[t]));
                let end = end_pattern.and_then(|t| Some(t.loc.clone()));
                // Compose together into a tuple
                (start, end, t)
            })
            .collect();
        // Trim to fit
        if let Some((start, _, _)) = tokens.first_mut() {
            // Token starts on a different line?
            if start.y != idx {
                start.x = 0..0;
                start.y = idx;
            }
        }
        if let Some((_, end, _)) = tokens.last_mut() {
            // Token ends on a different line?
            if end.is_none() || end.as_ref().unwrap().y != idx {
                let len = contents.len();
                *end = Some(RangeLoc { x: len..len, y: idx });
            }
        }
        // Obtain keywords if necessary
        if self.modified[idx] {
            *self.keywords.get_mut(idx).unwrap() = self.find_keywords(contents, idx);
            self.modified[idx] = false;
        }
        // Create hashmap for easier detection (keywords)
        let kws: HashMap<usize, (RangeLoc, &String)> = self.keywords[idx].iter()
            .map(|k| (k.loc.x.start, (k.loc.clone(), &k.kind)))
            .collect();
        // Create hashmap for easier detection (bounded)
        let tokens: HashMap<usize, (RangeLoc, RangeLoc, &String)> = tokens.iter()
            .map(|(start, end, tok)| (start.x.start, (start.clone(), end.clone().unwrap(), &tok.kind)))
            .collect();
        // Run through the whole line, making sure everything is accounted for
        let mut result = vec![];
        let mut x = 0;
        while x < contents.len() {
            if tokens.contains_key(&x) {
                // There is a bounded token here
                let (start, end, name) = &tokens[&x];
                result.push(Token::Start(name.to_string()));
                result.push(Token::Text(contents[start.x.start..end.x.end].to_string()));
                result.push(Token::End(name.to_string()));
                x = end.x.end;
            } else if kws.contains_key(&x) {
                // There is a keyword token here
                let (range, name) = &kws[&x];
                result.push(Token::Start(name.to_string()));
                result.push(Token::Text(contents[range.x.start..range.x.end].to_string()));
                result.push(Token::End(name.to_string()));
                x = range.x.end;
            } else {
                // There is no bounded token here, append to text
                let ch = contents.chars().nth(x).unwrap();
                if let Some(Token::Text(ref mut text)) = result.last_mut() {
                    text.push(ch);
                } else {
                    result.push(Token::Text(ch.to_string()));
                }
                x += 1;
            }
        }
        result
    }

    /// Initially highlight lines, additional lines can be added through append
    pub fn run(&mut self, lines: &Vec<String>) {
        // Locate patterns (starting from line 0)
        let mut patterns = self.find_patterns(0, lines);
        // Form tokens from patterns
        let tokens = Self::form_tokens(&mut patterns);
        // Add to highlighter
        self.patterns = patterns;
        self.tokens = tokens;
        // Build line references
        self.build_line_ref(lines.len());
        // Build keyword information
        self.modified = (0..lines.len()).map(|_| true).collect();
        self.keywords = (0..lines.len()).map(|_| vec![]).collect();
    }

    /// Add an additional line to this highlighter
    pub fn append(&mut self, line: &String) {
        let line_number = self.line_ref.len();
        let lines = vec![line.clone()];
        // Locate patterns
        let mut patterns = self.find_patterns(line_number, &lines);
        // Append to highlighter
        self.patterns.append(&mut patterns);
        self.line_ref.push(vec![]);
        self.modified.push(true);
        self.keywords.push(vec![]);
        // Perform update
        self.retokenize();
    }

    pub fn insert(&mut self, loc: Loc, line: &String) {
        self.modified[loc.y] = true;
        let ch = line.chars().nth(loc.x).unwrap();
        // Shift up patterns past a certain x
        let mut idx = self.patterns.iter().enumerate()
            .find(|(_, p)| loc.y < p.loc.y || (loc.y == p.loc.y && loc.x <= p.loc.x.start))
            .and_then(|(n, _)| Some(n))
            .unwrap_or(self.patterns.len());
        self.patterns.iter_mut()
            .skip(idx)
            .filter(|p| loc.y == p.loc.y)
            .for_each(|p| {
                p.loc.x.end += 1;
                p.loc.x.start += 1;
            });
        // Check for any pattern being destroyed
        let mut delete = false;
        if let Some(previous_pattern) = &self.patterns.get(idx.saturating_sub(1)) {
            if previous_pattern.loc.y == loc.y {
                if previous_pattern.loc.x.contains(&loc.x) {
                    self.patterns.remove(idx.saturating_sub(1));
                    idx -= 1;
                    delete = true;
                }
            }
        }
        // Check for new start or end pattern
        for kind in vec![PatternKind::Start, PatternKind::End, PatternKind::Hybrid] {
            let is = match kind {
                PatternKind::Start => self.is_new_start(loc, ch, line),
                PatternKind::End => self.is_new_end(loc, ch, line),
                PatternKind::Hybrid => self.is_new_hybrid(loc, ch, line),
            };
            if let Some((s, def)) = is {
                // Get the length of the pattern
                let len = match kind {
                    PatternKind::Start | PatternKind::Hybrid => def.start.len(),
                    PatternKind::End => def.end.len(),
                };
                // Register the pattern
                let pattern = Pattern {
                    token: None,
                    loc: RangeLoc { y: loc.y, x: s..(s + len) },
                    kind,
                    of: def.name.to_string(),
                };
                self.patterns.insert(idx, pattern);
                // Retokenize to correct any dodgy tokens
                self.retokenize();
                return;
            }
        }
        // If this insertion only deleted a token, then manually retokenize
        if delete {
            self.retokenize();
        }
    }

    pub fn remove(&mut self, loc: Loc, line: &String) {
        self.modified[loc.y] = true;
        // Find idx of next pattern
        let mut idx = self.patterns.iter().enumerate()
            .find(|(_, p)| loc.y < p.loc.y || (loc.y == p.loc.y && loc.x <= p.loc.x.end))
            .and_then(|(n, _)| Some(n))
            .unwrap_or(self.patterns.len());
        let mut modified = false;
        // Check to see if any patterns have been destroyed
        let in_pattern = self.patterns.iter().enumerate()
            .find(|(_, p)| loc.y == p.loc.y && p.loc.x.contains(&loc.x))
            .and_then(|(n, _)| Some(n));
        if let Some(pattern_idx) = in_pattern {
            self.patterns.remove(pattern_idx);
            modified = true;
        }
        // Check to see if any patterns have been created as a result
        if let Some(joined_char) = line.chars().nth(loc.x + 1) {
            let mut line = line.clone();
            line.remove(loc.x);
            let mut result: Option<(usize, usize, &String, PatternKind)> = None;
            // Find out if any new patterns have been created
            if let Some((s, def)) = self.is_new_start(loc, joined_char, &line) {
                // A new start pattern has been created
                result = Some((s, def.start.len(), &def.name, PatternKind::Start));
            } else if let Some((s, def)) = self.is_new_end(loc, joined_char, &line) {
                // A new end pattern has been created
                result = Some((s, def.end.len(), &def.name, PatternKind::End));
            } else if let Some((s, def)) = self.is_new_hybrid(loc, joined_char, &line) {
                // A new hybrid pattern has been created
                result = Some((s, def.start.len(), &def.name, PatternKind::Hybrid));
            }
            // If so, register
            if let Some((s, len, name, kind)) = result {
                let double_start = self.bounded_rules[name].start.len() > 1;
                let double_end = self.bounded_rules[name].end.len() > 1;
                let double = (kind == PatternKind::Start && double_start) || 
                             (kind == PatternKind::End && double_end) || 
                             (kind == PatternKind::Hybrid && double_start);
                if double {
                    let pattern = Pattern {
                        token: None,
                        loc: RangeLoc { y: loc.y, x: s..(s + len) },
                        kind,
                        of: name.to_string(),
                    };
                    self.patterns.insert(idx, pattern);
                    modified = true;
                    idx += 1;
                }
            }
        }
        // Shift back patterns before a certain x
        self.patterns.iter_mut()
            .skip(idx)
            .filter(|p| loc.y == p.loc.y)
            .for_each(|p| {
                p.loc.x.end -= 1;
                p.loc.x.start -= 1;
            });
        // Retokenize if necessary
        if modified {
            self.retokenize();
        }
    }

    pub fn insert_line(&mut self, y: usize) {
        self.patterns.iter_mut()
            .filter(|p| p.loc.y > y)
            .for_each(|p| p.loc.y += 1);
        self.line_ref.insert(y, vec![]);
        self.keywords.insert(y, vec![]);
        self.modified.insert(y, true);
    }

    pub fn remove_line(&mut self, y: usize) {
        self.patterns.iter_mut()
            .filter(|p| p.loc.y > y)
            .for_each(|p| p.loc.y -= 1);
        self.line_ref.remove(y);
        self.keywords.remove(y);
        self.modified.remove(y);
    }

    pub fn split_down(&mut self, loc: Loc) {
        // Inside a pattern: kill off the pattern
        let pattern_chop = self.patterns.iter().enumerate()
            .filter(|(_, p)| p.loc.y == loc.y)
            .find(|(_, p)| ((p.loc.x.start + 1)..p.loc.x.end).contains(&loc.x))
            .and_then(|(n, _)| Some(n));
        if let Some(idx) = pattern_chop {
            self.patterns.remove(idx);
            self.retokenize();
        }
        // Adjust keywords
        self.modified[loc.y] = true;
        // Adjust patterns
        self.insert_line(loc.y);
        self.patterns.iter_mut()
            .filter(|p| p.loc.y == loc.y && loc.x <= p.loc.x.start)
            .for_each(|p| {
                p.loc.y += 1;
                p.loc.x.start -= loc.x;
                p.loc.x.end -= loc.x;
            });
        self.build_line_ref(self.line_ref.len());
    }

    pub fn splice_up(&mut self, loc: Loc, line: &String) {
        let idx = self.patterns.iter().enumerate()
            .find(|(_, p)| p.loc.y >= loc.y + 1)
            .and_then(|(n, _)| Some(n))
            .unwrap_or(self.patterns.len());
        let mut modified = false;
        // Adjust keywords
        self.modified[loc.y] = true;
        // Adjust patterns
        self.patterns.iter_mut()
            .filter(|p| p.loc.y == loc.y + 1)
            .for_each(|p| {
                p.loc.y -= 1;
                p.loc.x.start += loc.x;
                p.loc.x.end += loc.x;
            });
        self.remove_line(loc.y + 1);
        // Check to see if any patterns have been created as a result
        if let Some(joined_char) = line.chars().nth(loc.x) {
            let line = line.clone();
            let mut result: Option<(usize, usize, &String, PatternKind)> = None;
            // Find out if any new patterns have been created
            //println!("{loc:?} {joined_char:?} {line:?}");
            if let Some((s, def)) = self.is_new_start(loc, joined_char, &line) {
                // A new start pattern has been created
                result = Some((s, def.start.len(), &def.name, PatternKind::Start));
            } else if let Some((s, def)) = self.is_new_end(loc, joined_char, &line) {
                // A new end pattern has been created
                result = Some((s, def.end.len(), &def.name, PatternKind::End));
            } else if let Some((s, def)) = self.is_new_hybrid(loc, joined_char, &line) {
                // A new hybrid pattern has been created
                result = Some((s, def.start.len(), &def.name, PatternKind::Hybrid));
            }
            // If so, register
            if let Some((s, len, name, kind)) = result {
                let double_start = self.bounded_rules[name].start.len() > 1;
                let double_end = self.bounded_rules[name].end.len() > 1;
                let double = (kind == PatternKind::Start && double_start) || 
                             (kind == PatternKind::End && double_end) || 
                             (kind == PatternKind::Hybrid && double_start);
                if double {
                    let pattern = Pattern {
                        token: None,
                        loc: RangeLoc { y: loc.y, x: s..(s + len) },
                        kind,
                        of: name.to_string(),
                    };
                    self.patterns.insert(idx, pattern);
                    modified = true;
                }
            }
        }
        if modified {
            self.retokenize();
        } else {
            self.build_line_ref(self.line_ref.len());
        }
    }

    fn retokenize(&mut self) {
        let patterns = &mut self.patterns;
        self.tokens = Self::form_tokens(patterns);
        self.build_line_ref(self.line_ref.len());
    }

    fn is_new_start(&self, loc: Loc, ch: char, line: &String) -> Option<(usize, &BoundedDef)> {
        self.is_new_pattern(loc, ch, line, PatternKind::Start)
    }

    fn is_new_end(&self, loc: Loc, ch: char, line: &String) -> Option<(usize, &BoundedDef)> {
        self.is_new_pattern(loc, ch, line, PatternKind::End)
    }

    fn is_new_hybrid(&self, loc: Loc, ch: char, line: &String) -> Option<(usize, &BoundedDef)> {
        self.is_new_pattern(loc, ch, line, PatternKind::Hybrid)
    }

    fn is_new_pattern(&self, loc: Loc, ch: char, line: &String, kind: PatternKind) -> Option<(usize, &BoundedDef)> {
        // Get all non-hybrid rules
        let rules = self.bounded_rules.values();
        let mut result = None;
        // Return a match if there is one
        for def in rules {
            let pattern = match kind {
                PatternKind::Start => &def.start,
                PatternKind::End => &def.end,
                PatternKind::Hybrid => &def.start,
            };
            let hybrid = def.start == def.end;
            // Determine if a start or end token has actually been created
            result = pattern.chars().enumerate()
                // Find locations within the start or end pattern where this character could be
                .filter(|(_, i)| *i == ch)
                // For each one, work out where the pattern would theoretically start
                .map(|(n, _)| loc.x.saturating_sub(n))
                // Attach a corresponding end location
                .map(|pattern_start| (pattern_start, pattern_start + pattern.len()))
                // Find out if any of these candidates are actually start or end patterns
                .map(|(start, end)| (start, &line[start..end] == pattern))
                .find(|(_, is_match)| *is_match && (!hybrid || kind == PatternKind::Hybrid))
                // Link in definition
                .and_then(|(pattern_start, _)| Some((pattern_start, def)));
            if result.is_some() {
                break;
            }
        }
        result
    }

    /// Finds patterns in the provided lines
    /// offset will add to the y axis (useful for when you're appending lines)
    fn find_patterns(&mut self, offset: usize, lines: &Vec<String>) -> Vec<Pattern> {
        let mut result = vec![];
        // For each line
        for (mut y, line) in lines.iter().enumerate() {
            // Offset y tokens
            y += offset;
            // For each character
            let mut x = 0;
            while x < line.len() {
                // Set up line and position info
                let line = &line[x..];
                let loc = Loc { y, x };
                // Work out if there is a pattern here
                let pattern = self.bounded_rules.values()
                    // Find whether this pattern is a start pattern or end pattern
                    .map(|def| (&def.name, line.starts_with(&def.start), line.starts_with(&def.end)))
                    // Find one that is either a start or end pattern
                    .find(|(_, starts, ends)| *starts || *ends);
                // If there is, register the pattern
                if let Some((name, starts, ends)) = pattern {
                    // Form the pattern
                    let def = &self.bounded_rules[name];
                    let kind = match (starts, ends) {
                        // Start pattern
                        (true, false) => PatternKind::Start,
                        // End pattern
                        (false, true) => PatternKind::End,
                        // Hybrid pattern
                        (true, true) => PatternKind::Hybrid,
                        // No pattern here
                        (false, false) => unreachable!(),
                    };
                    let of = def.name.clone();
                    let x_range = loc.x..(loc.x + def.end.len());
                    let range = RangeLoc { y: loc.y, x: x_range };
                    let pattern = Pattern { token: None, kind, loc: range, of };
                    result.push(pattern);
                    // Keep searching forward
                    x += if starts { def.start.len() } else { def.end.len() };
                } else {
                    x += 1;
                }
            }
        }
        result
    }

    /// Forms tokens based on patterns
    /// Ensure patterns are correctly registered before running this
    fn form_tokens(patterns: &mut Vec<Pattern>) -> Vec<TokenSpan> {
        let mut result = vec![];
        let mut registering = false;
        let mut registering_kind = "".to_string();
        // Run through patterns
        for (n, pattern) in patterns.iter_mut().enumerate() {
            let Pattern { of, kind, ref mut token, .. } = pattern;
            let len = result.len();
            match (kind, registering) {
                // New start token
                (PatternKind::Start, false) => {
                    registering = true;
                    registering_kind = of.clone();
                    // Make pattern active
                    *token = Some(len);
                    // Put on token
                    result.push(TokenSpan {
                        kind: of.clone(),
                        start: n,
                        end: None,
                    });
                }
                // Corresponding end token
                (PatternKind::End, true) => {
                    if *of == registering_kind {
                        if let Some(this) = result.last_mut() {
                            registering = false;
                            registering_kind = "".to_string();
                            // Make pattern active
                            *token = Some(len - 1);
                            // Update end pattern in token
                            this.end = Some(n);
                        }
                    }
                }
                // Opportunity to end a hybrid token
                (PatternKind::Hybrid, true) => {
                    if let Some(this) = result.last_mut() {
                        // Tokens are of the same type?
                        if *of == this.kind {
                            // They are, terminate this hybrid token
                            registering = false;
                            registering_kind = "".to_string();
                            // Make pattern active
                            *token = Some(len - 1);
                            // Update end pattern in token
                            this.end = Some(n);
                        }
                    }
                }
                // Opportunity to start a new hybrid token
                (PatternKind::Hybrid, false) => {
                    registering = true;
                    registering_kind = of.clone();
                    // Make pattern active
                    *token = Some(len);
                    // Push on token
                    result.push(TokenSpan {
                        kind: of.clone(),
                        start: n,
                        end: None,
                    });
                }
                _ => (),
            }
        }
        result
    }

    fn find_keywords(&self, line: &String, y: usize) -> Vec<Keyword> {
        let mut result = vec![];
        for (name, group) in &self.keyword_rules {
            for exp in group {
                result.append(&mut exp.find_iter(line)
                    .map(|s| Keyword { 
                        loc: RangeLoc { x: s.start()..s.end(), y }, 
                        kind: name.to_string(),
                    })
                    .collect());
            }
        }
        result
    }

    fn build_line_ref(&mut self, max: usize) {
        // Refresh line reference
        self.line_ref = vec![];
        (0..max).for_each(|_| self.line_ref.push(vec![]));
        // Register tokens according to the lines they span
        for (n, token) in self.tokens.iter().enumerate() {
            // Obtain start and end positions
            let start = self.patterns[token.start].loc.y;
            let end = match token.end {
                // Find y position of end pattern
                Some(end) => self.patterns[end].loc.y,
                // This token is a hanging token, max it out
                None => max - 1,
            };
            for y in start..=end {
                self.line_ref[y].push(n);
            }
        }
    }
}
