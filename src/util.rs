use crate::highlighter::Highlighter;
use crate::tokens::{TokOpt, Token};

pub fn trim(input: Vec<Token>, start: usize) -> Vec<Token> {
    let mut opt = Highlighter::from_stream(&input);
    let mut total_width = 0;
    for i in &opt {
        match i {
            TokOpt::Some(txt, _) => total_width += txt.len(),
            TokOpt::None(txt) => total_width += txt.len(),
        }
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
