# Synoptic
### Syntax highlighting for Rust applications

I originally wrote this for my text editor, Ox. It needed a fast, robust and reliable syntax highlighter that was configurable and was able to easily plug-in to a front end.

## Low-level
No pre-built language rules so everything is entirely up to you. Outputs a simple stream of tokens.

## Fast
Takes advantage of Rust's speed, prioritises speed. Works concurrently and using collections for performance gain.

## Simple
You can highlight code for most languages in just a few steps:

- Construct a `Highlighter` instance
- Add regular expressions and keywords to the highlighter and assign each a name
- Use the `run` method to recieve a stream of start, end and text tokens to use in your program.

You can also very quickly add, remove and modify build syntax highlighting rules to adjust to your liking.

## Example

Here's an example of a Rust syntax highlighter, using the termion crate.

```rust
use synoptic::{Token, Highlighter};
use termion::color;

const DEMO: &str = r#"/*
Multiline comments
Work great
*/

pub fn main() -> bool {
	// Demonstrate syntax highlighting in Rust!
	println!("Full Unicode Support: 你好！Pretty cool");
	return true;
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
    rust.add(r"(?ms)/\*.*?\*/", "comment").unwrap();
    // Add string definition
    rust.add("\".*?\"", "string").unwrap();
    // Add identifier definition
    rust.add(r"([a-z_][A-Za-z0-9_]*)\s*\(", "identifier").unwrap();
    // Add macro definition
    rust.add(r"([a-z_][A-Za-z0-9_]*!)\s*", "macro").unwrap();

    // Run highlighter
    let highlighting = rust.run(DEMO);
    
    // For each row
    for (c, row) in highlighting.iter().enumerate() {
    	// Print line number (with padding)
        print!("{: >3} |", c);
        // For each token within each row
        for tok in row {
        	// Handle the tokens
            match tok {
            	// Handle the start token (start foreground colour)
                Token::Start(kind) => match *kind {
                    "comment" => print!("{}", color::Fg(color::Black)),
                    "string" => print!("{}", color::Fg(color::Green)),
                    "keyword" => print!("{}", color::Fg(color::Blue)),
                    "type" => print!("{}", color::Fg(color::LightMagenta)),
                    "boolean" => print!("{}", color::Fg(color::LightGreen)),
                    "identifier" => print!("{}", color::Fg(color::Yellow)),
                    "macro" => print!("{}", color::Fg(color::Magenta)),
                    _ => (),
                }
                // Handle a text token (print out the contents)
                Token::Text(txt) => print!("{}", txt),
                // Handle an end token (reset foreground colour)
                Token::End(_) => print!("{}", color::Fg(color::Reset)),
            }
        }
        // Prevent text being cut off without a newline
        println!("");
    }
}
```

That will render this result:

![](https://i.postimg.cc/1t32c35k/image.png)

## Installation
Just add it to your `Cargo.toml`:

```toml
[dependencies]
synoptic = "0"
```

## License
`MIT` license to ensure that you can use it in your project

you can check the `LICENSE` file for more info
