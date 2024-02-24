# Synoptic

> Syntax highlighting for Rust applications

This is a pretty lightweight (only 3 main depedencies) and simple regex-based syntax highlighter for Rust. 

I originally wrote this for my text editor, Ox. It needed a fast, configurable and optimised syntax highlighter that could easily integrate with existing projects. However, you can (and are encouraged) to use it for any project you have in mind.

---
**Advantages:**
- **Customisable** - You can highlight almost any language by adding in custom syntax highlighting rules
- **Fast** - Is reasonably fast, enough so that it won't slow your projects down, even with large files and many different rules
- **Simple** - You can get highlighting code pretty quickly (see example below)
- **Incremental** - As this was designed for use with a text editor, it can really quickly re-highlight code upon edit commands
- **Built in language rules** - Get highlighting even faster by choosing from existing syntax rules
- **File Buffering** - Synoptic doesn't need the whole file to perform a correct highlighting job, thus allowing file buffering
- **Escaping** - Will handle escaping if you need it (`"here is a quote: \" tada!"`)
- **Interpolation**  - Will handle interpolation if you need it (`"My name is {name}, nice to meet you!"`)

**Disadvantages:**
- **Not very well established** - There may be inconsistencies in the included pre-built language highlighting rules
- **Lacks understanding** - This will not be able to provide very detailed syntax highlighting, as no parsing is performed
- **Interpolation is limited** - You can't nest interpolated tokens like `"this is { "f{ "u" }n" }"` 

Despite its disadvantages, if you just want a simple syntax highlighter with no frills or excess baggage, synoptic might just be your crate.

## Installation
Just add it to your `Cargo.toml`:
```toml
[dependencies]
synoptic = "2"
```

- Construct a `Highlighter` instance
- Add regular expressions and keywords to the highlighter and assign each a name
- Use the `run` method to generate tokens
- Use the `line` method to obtain the tokens for each line

## Built-in languages

You can also use some provided syntax highlighters for various popular languages using the `from_extension` function.
There is highly likely to be inconsistencies in the existing rules, please do open an issue if you spot any.

Currently, synoptic includes

- [x] Various Higher Level Languages: Python, Ruby, Lua, Perl, Java, Visual Basic, Scala
- [x] The C Family: C, C++, C#
- [x] Various Lower Level Languages: Rust, Go, Assembly
- [x] Web Technologies: HTML, CSS, PHP, Javascript, JSON, TypeScript
- [x] Mathematical Languages: MATLAB, R, Haskell, Prolog
- [x] Moblie Development: Kotlin, Swift, Dart
- [x] Markup Languages: Markdown, YAML, TOML, XML, CSV
- [x] Other: SQL, Bash

Open an issue if there is a language not yet supported, or if you notice any issues in the built-in syntax highlighting rules.

## Example

Here's an example of a Rust syntax highlighter, using the lliw crate.

```rust
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

```

That will render a result similar to this (depending on your terminal's colour scheme):

![](https://i.postimg.cc/0QJTsMbf/image.png)

## License
`MIT` license to ensure that you can use it in your project

you can check the `LICENSE` file for more info


