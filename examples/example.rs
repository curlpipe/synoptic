use synoptic::{Highlighter, TokOpt};
use lliw::Fg;

// Let's use some demonstration code
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
    // Setting up the highlighter
    // The `4` here just means tabs are shown as 4 spaces
    let mut h = Highlighter::new(4);

    // Bounded tokens are multiline tokens
    // Let's define multiline comments
    // In rust, these start with /* and end with */
    // Remember to escape any regex characters (like *)
    // The false here is whether or not to allow escaping
    // When true, we ignore any end markers with a backslash in front of them
    // So, if it were true: `/* this is a comment \*/ this is still a comment */ this isn't`
    h.bounded("comment", r"/\*", r"\*/", false);

    // Now let's define a string
    // In rust, format strings can be interpolated into between {}
    // We first define the name of the token, the starting and ending pattern
    // Then the starting and ending pattern of the interpolation section
    // We also want strings to be escapable e.g. "here's a quote: \" this is still a string"
    // Hence the true
    h.bounded_interp("string", "\"", "\"", "\\{", "\\}", true);

    // Now let's define some keywords
    // These are single line snippets of text
    h.keyword("keyword", r"\b(pub|fn|bool|let|return)\b");

    // Let's get numbers being highlighted
    h.keyword("digits", r"\b\d+\.(?:\.\d+)\b");

    // ... and some remaining syntax rules
    h.keyword("comment", "(//.*)$");
    h.keyword("boolean", r"\b(true|false)\b");
    h.keyword("macros", "[a-zA-Z_]+\\!");
    h.keyword("function", r"([a-z][a-zA-Z_]*)\s*\(");

    // Now let's run the highlighter on the example code
    // The run method takes a vector of strings (for each line)
    let code = CODE
        .split('\n')
        .map(|line| line.to_string())
        .collect();
    // Now we're ready to go
    h.run(&code);

    // Let's render the output
    for (line_number, line) in code.iter().enumerate() {
        // Line returns tokens for the corresponding line
        for token in h.line(line_number, &line) {
            // Tokens can either require highlighting or not require highlighting
            match token {
                // This is some text that needs to be highlighted
                TokOpt::Some(text, kind) => print!("{}{text}{}", colour(&kind), Fg::Reset),
                // This is just normal text with no highlighting
                TokOpt::None(text) => print!("{text}"),
            }
        }
        // Insert a newline at the end of every line
        println!();
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
        _ => panic!("unknown token name"),
    }
}
