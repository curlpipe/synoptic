//! These highlighters will return the following tokens names:
//!
//! keyword - a keyword for that language
//! boolean - a boolean
//! comment - a comment (both multiline and single line)
//! string - a string data type
//! number - a number
//! function - a function identifier
//! macro - a macro identifier
//! struct - a class / struct / enum / trait name
//! operator - operators within that language e.g. == or != or >= or +
//! namespace - a namespace for modules
//! character - a character data type
//! attribute - an attribute within the language
//! reference - for references within the language e.g. &self or &mut
//! symbol - a symbol data type (mainly for the Ruby language)
//! global - for global variable identifiers
//! regex - for regex datatypes in languages
//! header - for headers (mainly for the C language)
//!
//! These syntax highlighters are quite advanced and tend to do a decent job of syntax highlighting
//! with detail of which wouldn't be out of place in a popular text editor.
//! there may be an edge case where something goes a bit wrong, in that case, please open an issue

use crate::Highlighter;

/// Obtain the rust syntax highlighter
#[allow(clippy::missing_panics_doc, clippy::too_many_lines)]
#[must_use]
pub fn rust() -> Highlighter {
    let mut h = Highlighter::new();
    let keywords: Vec<&str> = vec![
        r"\b(as)\b",
        r"\b(break)\b",
        r"\b(char)\b",
        r"\b(const)\b",
        r"\b(continue)\b",
        r"\b(crate)\b",
        r"\b(else)\b",
        r"\b(enum)\b",
        r"\b(extern)\b",
        r"\b(fn)\b",
        r"\b(for)\b",
        r"\b(if)\b",
        r"\b(impl)\b",
        r"\b(in)\b",
        r"\b(let)\b",
        r"\b(loop)\b",
        r"\b(match)\b",
        r"\b(mod)\b",
        r"\b(move)\b",
        r"\b(mut)\b",
        r"\b(pub)\b",
        r"\b(ref)\b",
        r"\b(return)\b",
        r"\b(self)\b",
        r"\b(static)\b",
        r"\b(struct)\b",
        r"\b(super)\b",
        r"\b(trait)\b",
        r"\b(type)\b",
        r"\b(unsafe)\b",
        r"\b(use)\b",
        r"\b(where)\b",
        r"\b(while)\b",
        r"\b(async)\b",
        r"\b(await)\b",
        r"\b(dyn)\b",
        r"\b(abstract)\b",
        r"\b(become)\b",
        r"\b(box)\b",
        r"\b(do)\b",
        r"\b(final)\b",
        r"\b(macro)\b",
        r"\b(override)\b",
        r"\b(priv)\b",
        r"\b(typeof)\b",
        r"\b(unsized)\b",
        r"\b(virtual)\b",
        r"\b(yield)\b",
        r"\b(try)\b",
        r"\b('static)\b",
        r"\b(u8)\b",
        r"\b(u16)\b",
        r"\b(u32)\b",
        r"\b(u64)\b",
        r"\b(u128)\b",
        r"\b(usize)\b",
        r"\b(i8)\b",
        r"\b(i16)\b",
        r"\b(i32)\b",
        r"\b(i64)\b",
        r"\b(i128)\b",
        r"\b(isize)\b",
        r"\b(f32)\b",
        r"\b(f64)\b",
        r"\b(String)\b",
        r"\b(Vec)\b",
        r"\b(str)\b",
        r"\b(Some)\b",
        r"\b(bool)\b",
        r"\b(None)\b",
        r"\b(Box)\b",
        r"\b(Result)\b",
        r"\b(Option)\b",
        r"\b(Ok)\b",
        r"\b(Err)\b",
        r"\b(Self)\b",
        r"\b(std)\b",
    ];
    // Keywords
    h.join(keywords.as_slice(), "keyword").unwrap();
    h.join(&[r"\b(true)\b", r"\b(false)\b"], "boolean").unwrap();
    // Add comment definitions
    h.add(r"(?m)(//.*)$", "comment").unwrap();
    h.add_bounded("/*", "*/", false, "comment");
    // Add numbers definition
    h.join(&[r"\b(\d+.\d+|\d+)", r"\b(\d+.\d+(?:f32|f64))"], "number")
        .unwrap();
    // Add string definition
    h.add_bounded("\"", "\"", true, "string");
    // Add identifier definition
    h.add(r"([a-z_][A-Za-z0-9_]*)\s*\(", "function").unwrap();
    // Add macro definition
    h.add(r"([a-z_][A-Za-z0-9_]*!)\s*", "macro").unwrap();
    // Structs
    h.join(
        &[
            "(?:trait|enum|struct|impl)\\s+([A-Z][A-Za-z0-9_]*)\\s*",
            "impl(?:<.*?>|)\\s+([A-Z][A-Za-z0-9_]*)",
            "([A-Z][A-Za-z0-9_]*)::",
            r"([A-Z][A-Za-z0-9_]*)\s*\(",
            "impl.*for\\s+([A-Z][A-Za-z0-9_]*)",
            r"::\s*([A-Z_][A-Za-z0-9_]*)\s*\(",
        ],
        "struct",
    )
    .unwrap();
    // Operators
    h.join(
        &[
            r"(=)",
            r"(\+)",
            r"(\-)",
            r"(\*)",
            r"[^/](/)[^/]",
            r"(\+=)",
            r"(\-=)",
            r"(\*=)",
            r"(\\=)",
            r"(\?)",
            r"(==)",
            r"(!=)",
            r"(>=)",
            r"(<=)",
            r"(<)",
            r"(>)",
        ],
        "operator",
    )
    .unwrap();
    // Namespaces
    h.add(r"([a-z_][A-Za-z0-9_]*)::", "namespace").unwrap();
    // Characters
    h.join(&["('.')", r"('\\.')"], "character").unwrap();
    // Attributes
    h.add("(?ms)^\\s*(#(?:!|)\\[.*?\\])", "attribute").unwrap();
    // References
    h.join(
        &[
            "(&)", "&str", "&mut", "&self", "&i8", "&i16", "&i32", "&i64", "&i128", "&isize",
            "&u8", "&u16", "&u32", "&u64", "&u128", "&usize", "&f32", "&f64",
        ],
        "reference",
    )
    .unwrap();
    h
}

/// Obtain the python syntax highlighter
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn python() -> Highlighter {
    let mut h = Highlighter::new();
    let keywords: Vec<&str> = vec![
        r"\b(and)\b",
        r"\b(as)\b",
        r"\b(assert)\b",
        r"\b(break)\b",
        r"\b(class)\b",
        r"\b(continue)\b",
        r"\b(def)\b",
        r"\b(del)\b",
        r"\b(elif)\b",
        r"\b(else)\b",
        r"\b(except)\b",
        r"\b(exec)\b",
        r"\b(finally)\b",
        r"\b(for)\b",
        r"\b(from)\b",
        r"\b(global)\b",
        r"\b(if)\b",
        r"\b(import)\b",
        r"\b(in)\b",
        r"\b(is)\b",
        r"\b(lambda)\b",
        r"\b(not)\b",
        r"\b(or)\b",
        r"\b(pass)\b",
        r"\b(raise)\b",
        r"\b(return)\b",
        r"\b(try)\b",
        r"\b(while)\b",
        r"\b(with)\b",
        r"\b(yield)\b",
        r"\b(str)\b",
        r"\b(bool)\b",
        r"\b(int)\b",
        r"\b(tuple)\b",
        r"\b(list)\b",
        r"\b(dict)\b",
        r"\b(tuple)\b",
        r"\b(len)\b",
        r"\b(None)\b",
        r"\b(input)\b",
        r"\b(type)\b",
        r"\b(set)\b",
        r"\b(range)\b",
        r"\b(enumerate)\b",
        r"\b(open)\b",
        r"\b(iter)\b",
        r"\b(min)\b",
        r"\b(max)\b",
        r"\b(dir)\b",
        r"\b(self)\b",
        r"\b(isinstance)\b",
        r"\b(help)\b",
        r"\b(next)\b",
        r"\b(super)\b",
    ];
    // Keywords
    h.join(keywords.as_slice(), "keyword").unwrap();
    h.join(&[r"\b(True)\b", r"\b(False)\b"], "boolean").unwrap();
    // Add comment definitions
    h.add(r"(?m)(#.*)$", "comment").unwrap();
    // Add numbers definition
    h.add(r"\b(\d+.\d+|\d+)", "number").unwrap();
    // Add string definition
    h.add_bounded("\"\"\"", "\"\"\"", false, "string");
    h.add_bounded("\"", "\"", true, "string");
    // Add identifier definition
    h.add(r"([a-z_][A-Za-z0-9_]*)\s*\(", "function").unwrap();
    // Struct definition
    h.add(r"class\s+([A-Za-z0-9_]*)", "struct").unwrap();
    // Operators
    h.join(
        &[
            r"(=)",
            r"(\+)",
            r"(\-)",
            r"(\*)",
            r"(\s/\s)",
            r"(\s//\s)",
            r"(%)",
            r"(\+=)",
            r"(\-=)",
            r"(\*=)",
            r"(\\=)",
            r"(==)",
            r"(!=)",
            r"(>=)",
            r"(<=)",
            r"(<)",
            r"(>)",
        ],
        "operator",
    )
    .unwrap();
    // Attributes
    h.add("@.*$", "attribute").unwrap();
    h
}
