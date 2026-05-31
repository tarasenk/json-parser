pub mod parse;
pub mod tokenize;

use parse::parse;
use tokenize::tokenize;

pub use parse::ParseError;
pub use tokenize::TokenizeError;

use std::collections::HashMap;

/// Represents any valid JSON data type in a structured hierarchical tree.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// A master error enum that combines both Tokenizer and Parser errors.
#[derive(Debug, PartialEq)]
pub enum JsonError {
    Tokenizer(TokenizeError),
    Parser(ParseError),
}

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

        let mut expected_map = HashMap::new();
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
}
