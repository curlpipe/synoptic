#[cfg(test)]
use synoptic::highlighter::Highlighter;
use synoptic::tokens::Token::{Start, Text, End};

const DEMO: &str = r#"
/* hello
*/
pub fn main() -> bool {
    println!("Hello");
    return true;
}
"#;

#[test]
fn highlighter() {
    // Create new highlighter
    let mut rust = Highlighter::new();
    // Test adding keywords
    rust.add("fn", "keyword").unwrap();
    rust.add("let", "keyword").unwrap();
    rust.join(&["return", "pub"], "keyword").unwrap();
    rust.add("true", "keyword").unwrap();
    assert_eq!(
        rust.regex["keyword"][3].as_str(),
        "pub",
    );
    // Test highlighting
    assert_eq!(
        rust.run(DEMO), 
        [
            vec![], 
            vec![Text("/* hello".to_string())], 
            vec![Text("*/".to_string())], 
            vec![
                Start("keyword"), Text("pub".to_string()), End("keyword"), 
                Text(" ".to_string()), 
                Start("keyword"), Text("fn".to_string()), End("keyword"), 
                Text(" main() -> bool {".to_string())
            ], 
            vec![Text("    println!(\"Hello\");".to_string())], 
            vec![
                Text("    ".to_string()), 
                Start("keyword"), Text("return".to_string()), End("keyword"), 
                Text(" ".to_string()), 
                Start("keyword"), Text("true".to_string()), End("keyword"), 
                Text(";".to_string())
            ], 
            vec![Text("}".to_string())]
        ]
    );
    // Test regex
    rust.add("\".*?\"", "string").unwrap();
    rust.add(r"(?ms)/\*.*?\*/", "comment").unwrap();
    assert_eq!(
        rust.regex["string"][0].as_str(),
        "\".*?\"",
    );
    assert_eq!(
        rust.multiline_regex["comment"][0].as_str(),
        r"(?ms)/\*.*?\*/",
    );
    // Test highlighting
    assert_eq!(
        rust.run(DEMO), 
        [
            vec![], 
            vec![Start("comment"), Text("/* hello".to_string()), End("comment")], 
            vec![Start("comment"), Text("*/".to_string()), End("comment")], 
            vec![
                Start("keyword"), Text("pub".to_string()), End("keyword"), 
                Text(" ".to_string()), 
                Start("keyword"), Text("fn".to_string()), End("keyword"), 
                Text(" main() -> bool {".to_string())
            ], 
            vec![
                Text("    println!(".to_string()), 
                Start("string"), Text("\"Hello\"".to_string()), End("string"), 
                Text(");".to_string())
            ], 
            vec![
                Text("    ".to_string()), 
                Start("keyword"), Text("return".to_string()), End("keyword"), 
                Text(" ".to_string()), 
                Start("keyword"), Text("true".to_string()), End("keyword"), 
                Text(";".to_string())
            ], 
            vec![Text("}".to_string())]
        ]
    );
    // Test weird edge cases
    assert_eq!(
        rust.run("hello"), 
        [
            vec![Text("hello".to_string())], 
        ]
    );
    rust.add("print", "foo").unwrap();
    rust.add("pr", "foo").unwrap();
    assert_eq!(
        rust.run("print"), 
        [
            vec![Start("foo"), Text("print".to_string()), End("foo")], 
        ]
    );
}
