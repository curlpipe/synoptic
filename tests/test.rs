#[cfg(test)]
use synoptic::highlighter::Highlighter;
use synoptic::tokens::FullToken;
use synoptic::tokens::Token::{End, Start, Text};
use synoptic::util::trim;

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
    assert_eq!(rust.regex["keyword"][3].as_str(), "pub",);
    // Test highlighting
    assert_eq!(
        rust.run(DEMO),
        [
            vec![],
            vec![Text("/* hello".to_string())],
            vec![Text("*/".to_string())],
            vec![
                Start("keyword".to_string()),
                Text("pub".to_string()),
                End("keyword".to_string()),
                Text(" ".to_string()),
                Start("keyword".to_string()),
                Text("fn".to_string()),
                End("keyword".to_string()),
                Text(" main() -> bool {".to_string())
            ],
            vec![Text("    println!(\"Hello\");".to_string())],
            vec![
                Text("    ".to_string()),
                Start("keyword".to_string()),
                Text("return".to_string()),
                End("keyword".to_string()),
                Text(" ".to_string()),
                Start("keyword".to_string()),
                Text("true".to_string()),
                End("keyword".to_string()),
                Text(";".to_string())
            ],
            vec![Text("}".to_string())],
            vec![]
        ]
    );
    // Test regex
    rust.add("\".*?\"", "string").unwrap();
    rust.add(r"(?ms)/\*.*?\*/", "comment").unwrap();
    assert_eq!(rust.regex["string"][0].as_str(), "\".*?\"",);
    assert_eq!(
        rust.multiline_regex["comment"][0].as_str(),
        r"(?ms)/\*.*?\*/",
    );
    // Test highlighting
    assert_eq!(
        rust.run(DEMO),
        [
            vec![],
            vec![
                Start("comment".to_string()),
                Text("/* hello".to_string()),
                End("comment".to_string())
            ],
            vec![
                Start("comment".to_string()),
                Text("*/".to_string()),
                End("comment".to_string())
            ],
            vec![
                Start("keyword".to_string()),
                Text("pub".to_string()),
                End("keyword".to_string()),
                Text(" ".to_string()),
                Start("keyword".to_string()),
                Text("fn".to_string()),
                End("keyword".to_string()),
                Text(" main() -> bool {".to_string())
            ],
            vec![
                Text("    println!(".to_string()),
                Start("string".to_string()),
                Text("\"Hello\"".to_string()),
                End("string".to_string()),
                Text(");".to_string())
            ],
            vec![
                Text("    ".to_string()),
                Start("keyword".to_string()),
                Text("return".to_string()),
                End("keyword".to_string()),
                Text(" ".to_string()),
                Start("keyword".to_string()),
                Text("true".to_string()),
                End("keyword".to_string()),
                Text(";".to_string())
            ],
            vec![Text("}".to_string())],
            vec![],
        ]
    );
    assert_eq!(
        rust.run_line(DEMO, 2).unwrap(),
        vec![
            Start("comment".to_string()),
            Text("*/".to_string()),
            End("comment".to_string())
        ],
    );
    assert_eq!(
        rust.run_line(DEMO, 1).unwrap(),
        vec![
            Start("comment".to_string()),
            Text("/* hello".to_string()),
            End("comment".to_string())
        ],
    );
    assert_eq!(
        rust.run_line(DEMO, 3).unwrap(),
        vec![
            Start("keyword".to_string()),
            Text("pub".to_string()),
            End("keyword".to_string()),
            Text(" ".to_string()),
            Start("keyword".to_string()),
            Text("fn".to_string()),
            End("keyword".to_string()),
            Text(" main() -> bool {".to_string())
        ],
    );
    // Test weird edge cases
    assert_eq!(rust.run("hello"), [vec![Text("hello".to_string())],]);
    rust.add("print", "foo").unwrap();
    rust.add("pr", "foo").unwrap();
    assert_eq!(
        rust.run("print"),
        [vec![
            Start("foo".to_string()),
            Text("print".to_string()),
            End("foo".to_string())
        ],]
    );
    assert_eq!(
        rust.run("print\n"),
        [
            vec![
                Start("foo".to_string()),
                Text("print".to_string()),
                End("foo".to_string())
            ],
            vec![]
        ]
    );
    assert_eq!(
        rust.run("print\n\n"),
        [
            vec![
                Start("foo".to_string()),
                Text("print".to_string()),
                End("foo".to_string())
            ],
            vec![],
            vec![]
        ]
    );
    assert!(FullToken {
        text: "".to_string(),
        kind: "".to_string(),
        start: 0,
        end: 0,
        multi: false
    }
    .is_empty());
    assert_eq!(
        format!("{:?}", Highlighter::new()),
        format!("{:?}", Highlighter::default()),
    );
    let mut rust = Highlighter::new();
    rust.add("fn", "keyword").unwrap();
}

#[test]
fn bounded() {
    let mut h = Highlighter::new();
    h.add("pub", "keyword").unwrap();
    h.add_bounded("/*", "*/", false, "comment");
    h.add("(?ms)egg.*?gge", "egg").unwrap();
    h.add_bounded("\"", "\"", true, "string");
    assert_eq!(
        h.run("pub egg pub pub gge/* egg */\"hello \\\" \" pub \"safe!\" gge"),
        vec![vec![
            Start("keyword".to_string()),
            Text("pub".to_string()),
            End("keyword".to_string()),
            Text(" ".to_string()),
            Start("egg".to_string()),
            Text("egg pub pub gge".to_string()),
            End("egg".to_string()),
            Start("comment".to_string()),
            Text("/* egg */".to_string()),
            End("comment".to_string()),
            Start("string".to_string()),
            Text("\"hello \\\" \"".to_string()),
            End("string".to_string()),
            Text(" ".to_string()),
            Start("keyword".to_string()),
            Text("pub".to_string()),
            End("keyword".to_string()),
            Text(" ".to_string()),
            Start("string".to_string()),
            Text("\"safe!\"".to_string()),
            End("string".to_string()),
            Text(" gge".to_string()),
        ],],
    );
    let mut h = Highlighter::new();
    h.add("pub", "keyword").unwrap();
    h.add_bounded("/*", "*/", true, "comment");
    h.add("(?ms)egg.*?gge", "egg").unwrap();
    h.add_bounded("\"", "\"", true, "string");
    assert_eq!(
        h.run("pub egg pub pub gge/* egg \\*/\"hello \\\" \" pub \"safe!\" gge"),
        vec![vec![
            Start("keyword".to_string()),
            Text("pub".to_string()),
            End("keyword".to_string()),
            Text(" ".to_string()),
            Start("egg".to_string()),
            Text("egg pub pub gge".to_string()),
            End("egg".to_string()),
            Start("comment".to_string()),
            Text("/* egg \\*/\"hello \\\" \" pub \"safe!\" gge".to_string()),
            End("comment".to_string()),
        ],],
    );
}

#[test]
fn trimming() {
    assert_eq!(
        trim(
            &[
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string()),
                Text("lol".to_string())
            ],
            3
        ),
        [
            Start("foo".to_string()),
            Text("lo".to_string()),
            End("foo".to_string()),
            Text("lol".to_string())
        ],
    );
    assert_eq!(
        trim(
            &[
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string())
            ],
            4
        ),
        [
            Start("foo".to_string()),
            Text("o".to_string()),
            End("foo".to_string())
        ],
    );
    assert_eq!(
        trim(
            &[
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string())
            ],
            0
        ),
        [
            Start("foo".to_string()),
            Text("hello".to_string()),
            End("foo".to_string())
        ],
    );
    assert_eq!(
        trim(
            &[
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string())
            ],
            10
        ),
        [],
    );
    assert_eq!(
        trim(
            &[
                Text("hi".to_string()),
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string())
            ],
            1
        ),
        [
            Text("i".to_string()),
            Start("foo".to_string()),
            Text("hello".to_string()),
            End("foo".to_string())
        ],
    );
    assert_eq!(
        trim(
            &[
                Text("hi".to_string()),
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string())
            ],
            3
        ),
        [
            Start("foo".to_string()),
            Text("ello".to_string()),
            End("foo".to_string())
        ],
    );
    assert_eq!(
        trim(
            &[
                Text("hi".to_string()),
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string())
            ],
            2
        ),
        [
            Start("foo".to_string()),
            Text("hello".to_string()),
            End("foo".to_string())
        ],
    );
    assert_eq!(
        trim(
            &[
                Text("hi".to_string()),
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string()),
                Text("test".to_string())
            ],
            7
        ),
        [Text("test".to_string())],
    );
    assert_eq!(
        trim(
            &[
                Text("hi".to_string()),
                Start("foo".to_string()),
                Text("hello".to_string()),
                End("foo".to_string()),
                Text("te你st".to_string())
            ],
            10
        ),
        [Text(" st".to_string())],
    );
    assert_eq!(
        trim(
            &[
                Text("hi".to_string()),
                Start("foo".to_string()),
                Text("he你llo".to_string()),
                End("foo".to_string())
            ],
            5
        ),
        [
            Start("foo".to_string()),
            Text(" llo".to_string()),
            End("foo".to_string())
        ],
    );
    assert_eq!(trim(&[], 9), [],);
}
