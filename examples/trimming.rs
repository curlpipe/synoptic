use synoptic::{Highlighter, TokOpt, trim};
use lliw::Fg;

pub static CODE: &str = r#"println("你你"); // turn your back on me"#;

fn main() {
    let mut h = synoptic::from_extension("rs", 4).unwrap();
    let code = CODE.to_string();
    h.run(&vec![code.clone()]);
    let tokens = h.line(0, &code);
    // Trim and render
    let tokens = trim(&tokens, 0, 40, 4);
    println!("{:?}", tokens);
    for token in &tokens {
        // Tokens can either require highlighting or not require highlighting
        match token {
            // This is some text that needs to be highlighted
            TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
            // This is just normal text with no highlighting
            TokOpt::None(text) => print!("{text}"),
        }
    }
    println!();
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
