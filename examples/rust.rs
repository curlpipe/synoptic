use synoptic::{Highlighter, Token};
use termion::color;

const DEMO: &str = r#"/*
Multiline comments
Work great
*/

pub fn main() -> bool {
    // Demonstrate syntax highlighting in Rust!
    println!("Full Unicode Support: 你好！Escape: \" Pretty cool fn");
    return /* ignore 
    me */ true;
}
"#;

fn main() {
    // Build the rust syntax highlighter
    let mut rust = Highlighter::new();
    // Add keywords
    rust.join(&["fn", "return", "pub"], "keyword").unwrap();
    rust.join(&["bool"], "type").unwrap();
    rust.join(&["true", "false"], "boolean").unwrap();
    // Add comment definitions
    rust.add(r"(?m)(//.*)$", "comment").unwrap();
    rust.add_bounded("/*", "*/", false, "comment");
    // Add string definition
    rust.add_bounded("\"", "\"", true, "string");
    // Add identifier definition
    rust.add(r"([a-z_][A-Za-z0-9_]*)\s*\(", "identifier")
        .unwrap();
    // Add macro definition
    rust.add(r"([a-z_][A-Za-z0-9_]*!)\s*", "macro").unwrap();

    // Run highlighter
    let highlighting = rust.run(DEMO);
    //let highlighting = rust.run_line(DEMO, 7);

    // For each row
    for (c, row) in highlighting.iter().enumerate() {
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
                    "keyword" => print!("{}", color::Fg(color::Blue)),
                    "type" => print!("{}", color::Fg(color::LightMagenta)),
                    "boolean" => print!("{}", color::Fg(color::LightGreen)),
                    "identifier" => print!("{}", color::Fg(color::Yellow)),
                    "macro" => print!("{}", color::Fg(color::Magenta)),
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
