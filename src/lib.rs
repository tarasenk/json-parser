pub mod parse;
pub mod tokenize;

use parse::parse;
use tokenize::tokenize;

pub use parse::ParseError;
pub use tokenize::TokenizeError;

use core::fmt;
use indexmap::IndexMap;

/// Represents any valid JSON data type in a structured hierarchical tree.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(IndexMap<String, Value>),
}

fn escape_string(s: &str) -> String {
    let mut output = String::new();
    for c in s.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\x08' => output.push_str("\\b"),
            '\x0c' => output.push_str("\\f"),
            _ => output.push(c),
        }
    }
    output
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(map) => {
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in map {
                    if !first {
                        write!(f, ",")?;
                    }
                    first = false;
                    write!(f, "\"{}\":{}", escape_string(k), v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// A master error enum that combines both Tokenizer and Parser errors.
#[derive(Debug, PartialEq)]
pub enum JsonError {
    Tokenizer(TokenizeError),
    Parser(ParseError),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::Tokenizer(e) => write!(f, "tokenizer error: {}", e),
            JsonError::Parser(e) => write!(f, "parser error: {}", e),
        }
    }
}

impl std::error::Error for JsonError {}

// These implementations allow us to use the `?` operator to smoothly
// convert inner errors into our master JsonError type.
impl From<TokenizeError> for JsonError {
    fn from(err: TokenizeError) -> Self {
        JsonError::Tokenizer(err)
    }
}

impl From<ParseError> for JsonError {
    fn from(err: ParseError) -> Self {
        JsonError::Parser(err)
    }
}

/// The main entry point for your entire library.
/// Takes a raw JSON string slice, tokenizes it, parses it, and returns a unified tree.
///
/// # Example
/// ```
/// use json_parser::{from_str, Value};
///
/// let json = r#"{"success": true}"#;
/// let parsed = from_str(json).unwrap();
/// ```
pub fn from_str(input: &str) -> Result<Value, JsonError> {
    // 1. Run the Tokenizer pipeline to break text into flat symbols
    let tokens = tokenize(input)?;

    // 2. Run the Parser pipeline to construct the recursive object tree
    let value = parse(&tokens)?;

    Ok(value)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_end_to_end_pipeline() {
        let json_input = r#"{
            "project": "Custom JSON Parser",
            "complete": true,
            "version": 1.0,
            "contributors": [null, "You"]
        }"#;

        let result = from_str(json_input).unwrap();

        let mut expected_map = IndexMap::new();
        expected_map.insert(
            "project".to_string(),
            Value::String("Custom JSON Parser".to_string()),
        );
        expected_map.insert("complete".to_string(), Value::Boolean(true));
        expected_map.insert("version".to_string(), Value::Number(1.0));
        expected_map.insert(
            "contributors".to_string(),
            Value::Array(vec![Value::Null, Value::String("You".to_string())]),
        );

        assert_eq!(result, Value::Object(expected_map));
    }

    #[test]
    fn test_pipeline_error_forwarding() {
        // Missing a closing quote should trigger a tokenizer error wrapped in JsonError
        let broken_json = r#"{"name": "Alice}"#;
        assert_eq!(
            from_str(broken_json),
            Err(JsonError::Tokenizer(TokenizeError::UnclosedQuotes))
        );
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            JsonError::Tokenizer(TokenizeError::UnclosedQuotes).to_string(),
            "tokenizer error: unclosed quotes"
        );
        assert_eq!(
            JsonError::Parser(ParseError::UnexpectedEof).to_string(),
            "parser error: unexpected end of file"
        )
    }

    #[test]
    fn test_display_round_trip() {
        let json = r#"{"name":"alice","scores":[1,2],"active":true}"#;
        let parsed = from_str(json).unwrap();
        let serialized = parsed.to_string();
        let reparsed = from_str(&serialized).unwrap();
        println!("{parsed:?}");
        assert_eq!(parsed, reparsed);
    }
}
