use synoptic::{Highlighter, TokOpt, trim_fit};

pub static CODE: &str = "p你ub eg你g 你fn";

fn main() {
    let mut h = synoptic::from_extension("rs", 4).unwrap();
    let mut code: Vec<String> = CODE.split('\n').map(|x| x.to_string()).collect();
    h.run(&code);
    for start in 0..20 {
        // println!("{}|{}", &code[0].chars().take(start).collect::<String>(), &code[0].chars().skip(start).collect::<String>());
        for token in trim_fit(&h.line(0, &code[0]), start, 5, 4) {
            match token {
                TokOpt::Some(text, kind) => print!("[{text}]"),
                TokOpt::None(text) => print!("({text})"),
            }
        }
        println!();
    }
}
