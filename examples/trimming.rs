use synoptic::{Highlighter, TokOpt, trim};
use lliw::Fg;

pub static CODE: &str = r#"
    arst的st了st在st为sts
  art的st了st在st为sts
hello world!
"#;

fn main() {
    let mut h = synoptic::from_extension("diff", 4).unwrap();
    let mut code: Vec<String> = CODE.split('\n').map(|x| x.to_string()).collect();
    h.run(&code);
    // Trim and render
    for length in 0..30 {
        for (line_no, line) in code.iter().enumerate() {
            let tokens = h.line(line_no, &line);
            let tokens = trim(&tokens, 0, length, 4);
            for token in &tokens {
                // Tokens can either require highlighting or not require highlighting
                match token {
                    // This is some text that needs to be highlighted
                    TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
                    // This is just normal text with no highlighting
                    TokOpt::None(text) => print!("{text}"),
                }
            }
            println!("|");
        }
    }
}

fn colour(name: &str) -> Fg {
    // This function will take in the function name
    // And it will output the correct foreground colour
    match name {
        "comment" => Fg::LightBlack,
        "digit" => Fg::Purple,
        "string" => Fg::Green,
        "macros" => Fg::LightPurple,
        "boolean" => Fg::Blue,
        "keyword" => Fg::Yellow,
        "function" => Fg::Red,
        "operator" => Fg::LightBlack,
        "link" => Fg::LightBlue,
        "list" => Fg::Green,
        "insertion" => Fg::Green,
        "deletion" => Fg::Red,
        "reference" => Fg::Purple,
        _ => panic!("unknown token {name}"),
    }
}
