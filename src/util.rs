use crate::highlighter::Highlighter;
use crate::tokens::{TokOpt, Token};

#[macro_export]
macro_rules! glen {
    ($e:expr) => {
        $e.chars().collect::<Vec<_>>().len()
    };
}

// Converts grapheme index into byte index
#[macro_export]
macro_rules! gidx {
    ($e:expr, $i:expr) => {
        $e.chars().nth($i).unwrap().len_utf8()
    };
}

/// This will trim tokens to adjust to an offset
/// This is really useful if you are building a text editor on the command line
/// The first argument is a stream of tokens, the second is the start point
/// ```rust
/// let mut rust = Highlighter::new();
/// rust.add("fn", "keyword");
/// let result = rust.run("fn");
/// trim(&result, 1); // <- This will return [Start("keyword"), Text("n"), End("keyword")]
/// ```
/// This will cut off the beginning of the token and keep the token's colour intact
#[must_use]
pub fn trim(input: &[Token], start: usize) -> Vec<Token> {
    let mut opt = Highlighter::from_stream(input);
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
    Highlighter::from_opt(&opt)
}
