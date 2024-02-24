use synoptic::{Highlighter, TokOpt};

pub static CODE: &str = "\
\"

MULTILINE MADNESS

\"
";

fn main() {
    //let mut h = synoptic::from_extension("rs", 4).unwrap();
    let mut h = Highlighter::new(4);
    //h.bounded("string", "\"\"\"", "\"\"\"", true);
    h.bounded("string", "\"", "\"", true);
    let mut code = CODE.split('\n').map(|x| x.to_string()).collect();
    h.run(&code);
    println!("{:#?}", h.atoms);
    println!("{:#?}", h.tokens);
    println!("{:#?}", h.line_ref);
    for (y, line) in code.iter().enumerate() {
        print!("{: <3} |", y);
        for token in h.line(y, &line) {
            match token {
                TokOpt::Some(text, kind) => print!("({text})"),
                TokOpt::None(text) => print!("{text}"),
            }
        }
        println!();
    }
}
