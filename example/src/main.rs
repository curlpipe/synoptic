use lliw::Fg;
use synoptic::{Highlighter, TokOpt};
use std::time::Instant;

pub static CODE: &str = "\
/*
Multiline comments
Work great
*/

pub fn main() -> bool {
	// Demonstrate syntax highlighting in Rust!
	println!(\"Full Unicode Support: 你好\");
    // Interpolation
    let name = \"peter\";
    println!(\"My name is {name}, nice to meet you!\");
    // Bye!
	return true;
}
";

fn main() {
    //benchmark();
    let mut code: Vec<String> = CODE.split('\n').map(|x| x.to_string()).collect();
    let mut h = synoptic::from_extension("rs", 4).unwrap();
    h.run(&code);
    for (y, line) in code.iter().enumerate() {
        print!("{: <3} |", y);
        for token in h.line(y, &line) {
            match token {
                TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
                TokOpt::None(text) => print!("{text}"),
            }
        }
        println!();
    }
}

fn colour(kind: &str) -> Fg {
    match kind {
        "string" => Fg::Rgb(54, 161, 102),
        "boolean" => Fg::Rgb(54, 161, 102),
        "comment" => Fg::Rgb(108, 107, 90),
        "digit" => Fg::Rgb(157, 108, 124),
        "keyword" => Fg::Rgb(91, 157, 72),
        "attribute" => Fg::Rgb(95, 145, 130),
        "character" => Fg::Rgb(125, 151, 38),
        "namespace" => Fg::Rgb(125, 151, 38),
        "struct" => Fg::Rgb(125, 151, 38),
        "operator" => Fg::Rgb(125, 151, 38),
        "header" => Fg::Rgb(54, 161, 102),
        "reference" => Fg::Rgb(125, 151, 38),
        "type" => Fg::Rgb(165, 152, 13),
        "function" => Fg::Rgb(174, 115, 19),
        "macro" => Fg::Rgb(157, 108, 124),
        "heading" => Fg::Rgb(174, 115, 19),
        "tag" => Fg::Rgb(174, 115, 19),
        "bold" => Fg::Rgb(157, 108, 124),
        "strikethrough" => Fg::Rgb(54, 161, 102),
        "italic" => Fg::Rgb(125, 151, 38),
        "block" => Fg::Rgb(125, 151, 38),
        "table" => Fg::Rgb(125, 151, 38),
        "type" => Fg::Rgb(165, 152, 13),
        "linebreak" => Fg::Rgb(54, 161, 102),
        "math" => Fg::Rgb(54, 161, 102),
        "footnote" => Fg::Rgb(108, 107, 90),
        "quote" => Fg::Rgb(157, 108, 124),
        "list" => Fg::Rgb(91, 157, 72),
        "image" => Fg::Rgb(125, 151, 38),
        "link" => Fg::Rgb(165, 152, 13),
        "key" => Fg::Rgb(165, 152, 13),
        _ => panic!("Unknown token name {kind}"),
    }
}

fn benchmark() {
    let start = Instant::now();
    let mut h = synoptic::from_extension("rs", 4).unwrap();
    let end = Instant::now();
    println!("Initialisation time: {:?}", end - start);

    let mut file  = std::fs::read_to_string("/home/luke/dev/rust/kaolinite/demos/8.rs").unwrap().split('\n').map(|x| x.to_string()).collect::<Vec<String>>();
    let viewport_file1 = file.iter().take(10).cloned().collect();
    let viewport_file2 = file.iter().take(100).cloned().collect();
    let viewport_file3 = file.iter().take(1000).cloned().collect();

    let start = Instant::now();
    h.run(&viewport_file1);
    let end = Instant::now();
    println!("Run time ({}): {:?}", 10, end - start);
    let start = Instant::now();
    h.run(&viewport_file2);
    let end = Instant::now();
    println!("Run time ({}): {:?}", 100, end - start);
    let start = Instant::now();
    h.run(&viewport_file3);
    let end = Instant::now();
    println!("Run time ({}): {:?}", 1000, end - start);
    let start = Instant::now();
    h.run(&file);
    let end = Instant::now();
    println!("Run time ({}): {:?}", file.len(), end - start);

    let mut h = synoptic::from_extension("rs", 4).unwrap();
    //file[9996] = "/*".to_string();
    h.run(&file);

    for (mut y, line) in file.iter().skip(9996).take(7).enumerate() {
        y += 9996;
        print!("{: <3} |", y);
        for token in h.line(y, &line) {
            match token {
                TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
                TokOpt::None(text) => print!("{text}"),
            }
        }
        println!();
    }

    let start = Instant::now();

    h.edit(10000, &"/* this is a test pub  */ pub fn egg() return 3 + 4".to_string());
    file[10000] = "/* this is a test pub  */ pub fn egg() return 3 + 4".to_string();
    h.edit(10004, &"We are all living in a simulation".to_string());
    file[10004] = "We are all living in a simulation".to_string();
    for i in 1..10000 {
        h.edit(i, &file[i+1]);
        file[i] = file[i+1].clone()
    }

    for (mut y, line) in file.iter().skip(9996).take(7).enumerate() {
        y += 9996;
        print!("{: <3} |", y);
        for token in h.line(y, &line) {
            match token {
                TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
                TokOpt::None(text) => print!("{text}"),
            }
        }
        println!();
    }

    let end = Instant::now();
    println!("Edit time: {:?}", end - start);
}
