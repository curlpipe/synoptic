use synoptic::{Highlighter, TokOpt, trim_fit};
use lliw::Fg;

pub static CODE: &str = r#"f"""#;

fn main() {
    let mut h = synoptic::from_extension("py", 4).unwrap();
    let mut code: Vec<String> = CODE.split('\n').map(|x| x.to_string()).collect();
    h.run(&code);
    // Initial state
    for token in &h.line(0, &code[0]) {
        match token {
            TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
            TokOpt::None(text) => print!("{text}"),
        }
    }
    println!();
    // Try changing it
    code[0] = r#"f"{}""#.to_string();
    h.edit(0, &code[0]);
    // Observe incorrect new state
    for token in &h.line(0, &code[0]) {
        match token {
            TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
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
