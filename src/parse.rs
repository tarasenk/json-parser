use crate::Value;
use crate::tokenize::Token;
use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken(Token),
    ExpectedColon,
    ExpectedComma,
}

/// Takes a flat slice of tokens and constructs a hierarchical JSON value tree.
pub fn parse(tokens: &[Token]) -> Result<Value, ParseError> {
    let mut token_iter = tokens.iter().peekable();
    let result = parse_value(&mut token_iter)?;

    if token_iter.peek().is_some() {
        let next_token = token_iter.next().unwrap().clone();
        return Err(ParseError::UnexpectedToken(next_token));
    }

    Ok(result)
}

fn parse_value(tokens: &mut Peekable<Iter<Token>>) -> Result<Value, ParseError> {
    let token = tokens.next().ok_or(ParseError::UnexpectedEof)?;

    match token {
        Token::Null => Ok(Value::Null),
        Token::True => Ok(Value::Boolean(true)),
        Token::False => Ok(Value::Boolean(false)),
        Token::Number(num) => Ok(Value::Number(*num)),
        Token::String(str_val) => Ok(Value::String(str_val.clone())),
        Token::LeftBracket => parse_array(tokens),
        Token::LeftBrace => parse_object(tokens),
        _ => Err(ParseError::UnexpectedToken(token.clone())),
    }
}

fn parse_array(tokens: &mut Peekable<Iter<Token>>) -> Result<Value, ParseError> {
    let mut elements = Vec::new();

    if matches!(tokens.peek(), Some(&&Token::RightBracket)) {
        tokens.next();
        return Ok(Value::Array(elements));
    }

    loop {
        elements.push(parse_value(tokens)?);

        match tokens.next().ok_or(ParseError::UnexpectedEof)? {
            Token::Comma => {
                // trailing commas are not valid JSON
                if matches!(tokens.peek(), Some(&&Token::RightBracket)) {
                    return Err(ParseError::UnexpectedToken(Token::RightBracket));
                }
            }
            Token::RightBracket => break,
            unknown_token => return Err(ParseError::UnexpectedToken(unknown_token.clone())),
        }
    }

    Ok(Value::Array(elements))
}

fn parse_object(tokens: &mut Peekable<Iter<Token>>) -> Result<Value, ParseError> {
    let mut map = HashMap::new();

    if matches!(tokens.peek(), Some(&&Token::RightBrace)) {
        tokens.next();
        return Ok(Value::Object(map));
    }

    loop {
        let key_token = tokens.next().ok_or(ParseError::UnexpectedEof)?;
        let key_str = match key_token {
            Token::String(s) => s.clone(),
            _ => return Err(ParseError::UnexpectedToken(key_token.clone())),
        };

        let colon_token = tokens.next().ok_or(ParseError::UnexpectedEof)?;
        if colon_token != &Token::Colon {
            return Err(ParseError::ExpectedColon);
        }

        let value = parse_value(tokens)?;
        map.insert(key_str, value);

        match tokens.next().ok_or(ParseError::UnexpectedEof)? {
            Token::Comma => {
                // trailing commas are not valid JSON
                if matches!(tokens.peek(), Some(&&Token::RightBrace)) {
                    return Err(ParseError::UnexpectedToken(Token::RightBrace));
                }
            }
            Token::RightBrace => break,
            unknown_token => return Err(ParseError::UnexpectedToken(unknown_token.clone())),
        }
    }

    Ok(Value::Object(map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenize::tokenize;

    #[test]
    fn test_parse_primitive_arrays() {
        let tokens = tokenize(r#"[null, true, [42.0]]"#).unwrap();
        let parsed = parse(&tokens).unwrap();

        let expected = Value::Array(vec![
            Value::Null,
            Value::Boolean(true),
            Value::Array(vec![Value::Number(42.0)]),
        ]);

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_illegal_trailing_comma() {
        let tokens = tokenize(r#"[1, 2, ]"#).unwrap();
        assert_eq!(
            parse(&tokens),
            Err(ParseError::UnexpectedToken(Token::RightBracket))
        );
    }

    #[test]
    fn test_parse_complex_object() {
        let input = r#"{
            "user": "Bob",
            "age": 30.0,
            "skills": ["Rust", "Python"]
        }"#;

        let tokens = tokenize(input).unwrap();
        let parsed = parse(&tokens).unwrap();

        let mut expected_map = HashMap::new();
        expected_map.insert("user".to_string(), Value::String("Bob".to_string()));
        expected_map.insert("age".to_string(), Value::Number(30.0));
        expected_map.insert(
            "skills".to_string(),
            Value::Array(vec![
                Value::String("Rust".to_string()),
                Value::String("Python".to_string()),
            ]),
        );

        assert_eq!(parsed, Value::Object(expected_map));
    }

    #[test]
    fn test_deeply_nested_arrays() {
        let tokens = tokenize(r#"[[[[1.0]]]]"#).unwrap();
        let parsed = parse(&tokens).unwrap();

        let expected = Value::Array(vec![Value::Array(vec![Value::Array(vec![Value::Array(
            vec![Value::Number(1.0)],
        )])])]);

        assert_eq!(parsed, expected);
    }
}
