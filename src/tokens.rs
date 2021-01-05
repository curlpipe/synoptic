use unicode_width::UnicodeWidthChar;

/// For storing tokens to put into a string
/// It has a start token, to mark the start of a token
/// It has a text token, for the text inbetween and inside tokens
/// It also has an end token, to mark the end of a token
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Start(&'static str),
    Text(String),
    End(&'static str),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokOpt {
    Some(String, &'static str),
    None(String),
}

impl TokOpt {
    pub fn is_empty(&self) -> bool {
        match self {
            TokOpt::Some(text, _) => text.len() == 0,
            TokOpt::None(text) => text.len() == 0,
        }
    }

    pub fn nibble(&mut self) -> Option<char> {
        match self {
            TokOpt::Some(ref mut text, _) => {
                let ch = *text.chars().collect::<Vec<_>>().get(0)?;
                text.remove(0);
                if UnicodeWidthChar::width(ch)? > 1 {
                    text.insert(0, ' ');
                }
                Some(ch)
            }
            TokOpt::None(ref mut text) => {
                let ch = *text.chars().collect::<Vec<_>>().get(0)?;
                text.remove(0);
                if UnicodeWidthChar::width(ch)? > 1 {
                    text.insert(0, ' ');
                }
                Some(ch)
            }
        }
    }
}

/// For storing all the data in a token to prevent overwriting
/// This contains the contents, type, start and end of the token
/// This is used to compare tokens to each other to prevent tokens inside tokens
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FullToken {
    pub text: &'static str,
    pub kind: &'static str,
    pub start: usize,
    pub end: usize,
    pub multi: bool,
}

impl FullToken {
    /// Returns the length of the token
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Determines if the token is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
