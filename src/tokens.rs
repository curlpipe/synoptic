// Storing tokens to put into a string
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Start(&'static str),
    Text(String),
    End(&'static str),
}

// Storing all the data in a token to prevent overwriting
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FullToken {
    pub text: &'static str,
    pub kind: &'static str,
    pub start: usize,
    pub end: usize,
}

impl FullToken {
    pub fn len(&self) -> usize {
        self.text.len()
    }
}
