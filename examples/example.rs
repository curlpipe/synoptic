use synoptic::languages::rust;
use synoptic::tokens::Token;
use termion::color;

const DEMO: &str = r#"/*
  Comment
  covering
  six
  lines
*/

// Version of the program
pub const VERSION: &str = "0.4.6";

#[derive(Debug)]
pub struct Foo<T> {
    c: char,
}

impl<T> Foo<T> {
    pub fn new() { Foo { c: 'a' } }
}

fn main() {
    let f = Foo::new();
    println!("{:?}", VERSION);
    let mut awesome = true;
    let mut i = 2.4;
    i += 10;
    while awesome {
        // Exit with weird status code
        std::process::exit(3);
    }
}
"#;

fn main() {
    // Obtain provided Rust syntax highlighter
    let h = rust();

    // Run highlighter
    let result = h.run(DEMO);

    // For each row
    for (c, row) in result.iter().enumerate() {
        // Print line number (with padding)
        print!("{: >3} |", c + 1);
        // For each token within each row
        for tok in row {
            // Handle the tokens
            match tok {
                // Handle the start token (start foreground colour)
                Token::Start(kind) => match kind.as_str() {
                    "comment" => print!("{}", color::Fg(color::LightBlack)),
                    "string" => print!("{}", color::Fg(color::Green)),
                    "number" => print!("{}", color::Fg(color::Red)),
                    "keyword" => print!("{}", color::Fg(color::Blue)),
                    "boolean" => print!("{}", color::Fg(color::LightGreen)),
                    "function" => print!("{}", color::Fg(color::Yellow)),
                    "struct" => print!("{}", color::Fg(color::Magenta)),
                    "macro" => print!("{}", color::Fg(color::Magenta)),
                    "operator" => print!("{}", color::Fg(color::LightWhite)),
                    "namespace" => print!("{}", color::Fg(color::Blue)),
                    "character" => print!("{}", color::Fg(color::Cyan)),
                    "attribute" => print!("{}", color::Fg(color::Blue)),
                    "reference" => print!("{}", color::Fg(color::Magenta)),
                    _ => (),
                },
                // Handle a text token (print out the contents)
                Token::Text(txt) => print!("{}", txt),
                // Handle an end token (reset foreground colour)
                Token::End(_) => print!("{}", color::Fg(color::Reset)),
            }
        }
        // Prevent text being cut off without a newline
        println!();
    }
}
