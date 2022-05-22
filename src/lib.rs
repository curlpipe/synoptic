#![warn(clippy::all, clippy::pedantic)]

//! # Synoptic
//! ## A simple rust syntax highlighting crate
//!
//! Here's an example of it in action (using the `termion` crate)
//!
//! ```rust
//! use synoptic::{Token, Highlighter};
//! use termion::color;
//!
//! const DEMO: &str = r#"/*
//! Multiline comments
//! Work great
//! */
//!
//! pub fn main() -> bool {
//!     // Demonstrate syntax highlighting in Rust!
//!     println!("Full Unicode Support: 你好！Pretty cool");
//!     return true;
//! }
//! "#;
//!
//! fn main() {
//!     // Build the rust syntax highlighter
//!     let mut rust = Highlighter::new();
//!     // Add keywords
//!     rust.join(&["fn", "return", "pub"], "keyword").unwrap();
//!     rust.join(&["bool"], "type").unwrap();
//!     rust.join(&["true", "false"], "boolean").unwrap();
//!     // Add comment definitions
//!     rust.add(r"(?m)(//.*)$", "comment").unwrap();
//!     rust.add(r"(?ms)/\*.*?\*/", "comment").unwrap();
//!     // Add string definition
//!     rust.add("\".*?\"", "string").unwrap();
//!     // Add identifier definition
//!     rust.add(r"([a-z_][A-Za-z0-9_]*)\s*\(", "identifier").unwrap();
//!     // Add macro definition
//!     rust.add(r"([a-z_][A-Za-z0-9_]*!)\s*", "macro").unwrap();
//!
//!     // Run highlighter
//!     let highlighting = rust.run(DEMO);
//!     
//!     // For each row
//!     for (c, row) in highlighting.iter().enumerate() {
//!         // Print line number (with padding)
//!         print!("{: >3} |", c);
//!         // For each token within each row
//!         for tok in row {
//!         // Handle the tokens
//!             match tok {
//!                 // Handle the start token (start foreground colour)
//!                 Token::Start(kind) => match kind.as_str() {
//!                     "comment" => print!("{}", color::Fg(color::Black)),
//!                     "string" => print!("{}", color::Fg(color::Green)),
//!                     "keyword" => print!("{}", color::Fg(color::Blue)),
//!                     "type" => print!("{}", color::Fg(color::LightMagenta)),
//!                     "boolean" => print!("{}", color::Fg(color::LightGreen)),
//!                     "identifier" => print!("{}", color::Fg(color::Yellow)),
//!                     "macro" => print!("{}", color::Fg(color::Magenta)),
//!                     _ => (),
//!                 }
//!                 // Handle a text token (print out the contents)
//!                 Token::Text(txt) => print!("{}", txt),
//!                 // Handle an end token (reset foreground colour)
//!                 Token::End(_) => print!("{}", color::Fg(color::Reset)),
//!             }
//!         }
//!         // Prevent text being cut off without a newline
//!         println!("");
//!     }
//! }
//! ```

/// This provides the main Highlighter class you will need to make your own
/// syntax rules, or if you wish to modify the existing rules from the set of provided highlighters
pub mod highlighter;
/// This provides a set of prebuilt highlighters for various languages
/// You can always build on top of them, as they just return highlighter classes
pub mod languages;
/// This provides the types of tokens which you can use to apply your syntax highlighting into
/// whichever format you please
pub mod tokens;
/// This provides utilities to help with formatting tokens on the screen
pub mod util;

/// Highlighter is the highlighter struct that does the highlighting
/// This is what you'll want to use
pub use highlighter::Highlighter;

/// This contains enums and structs that represent tokens
pub use tokens::{TokOpt, Token};

/// This contains utilitiues for trimming lines
pub use util::trim;
