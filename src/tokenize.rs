use core::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Number(f64),
    String(String),
    True,
    False,
    Null,
}

#[derive(Debug, PartialEq)]
pub enum TokenizeError {
    UnexpectedEof,
    CharNotRecognized(char),
    UnclosedQuotes,
    InvalidNumberFormat(String),
    InvalidEscapeSequence(char),
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizeError::UnexpectedEof => {
                write!(f, "unexpected end of file")
            }
            TokenizeError::CharNotRecognized(c) => {
                write!(f, "character not recognized: '{}'", c)
            }
            TokenizeError::UnclosedQuotes => {
                write!(f, "unclosed quotes")
            }
            TokenizeError::InvalidNumberFormat(s) => {
                write!(f, "invalid number format: {}", s)
            }
            TokenizeError::InvalidEscapeSequence(c) => {
                write!(f, "invalid escape sequence: '\\{}'", c)
            }
        }
    }
}

impl std::error::Error for TokenizeError {}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }

            _ if ch.is_whitespace() => {
                chars.next();
            }

            _ if ch.is_ascii_alphabetic() => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        word.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match word.as_str() {
                    "true" => tokens.push(Token::True),
                    "false" => tokens.push(Token::False),
                    "null" => tokens.push(Token::Null),
                    _ => return Err(TokenizeError::CharNotRecognized(ch)),
                }
            }

            '"' => {
                chars.next(); // consume opening quote
                let mut string_contents = String::new();
                let mut closed = false;

                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        closed = true;
                        chars.next();
                        break;
                    } else if c == '\\' {
                        chars.next(); // consume backslash
                        match chars.next() {
                            Some('"') => string_contents.push('"'),
                            Some('\\') => string_contents.push('\\'),
                            Some('/') => string_contents.push('/'),
                            Some('b') => string_contents.push('\x08'),
                            Some('f') => string_contents.push('\x0c'),
                            Some('n') => string_contents.push('\n'),
                            Some('r') => string_contents.push('\r'),
                            Some('t') => string_contents.push('\t'),
                            Some('u') => {
                                // \uXXXX unicode escape — surrogate pairs (U+D800–U+DFFF) are not supported
                                let mut hex_str = String::new();
                                for _ in 0..4 {
                                    match chars.next() {
                                        Some(hex_char) if hex_char.is_ascii_hexdigit() => {
                                            hex_str.push(hex_char);
                                        }
                                        Some(_) => {
                                            return Err(TokenizeError::InvalidEscapeSequence('u'));
                                        }
                                        None => return Err(TokenizeError::UnexpectedEof),
                                    }
                                }
                                let code_point = u16::from_str_radix(&hex_str, 16)
                                    .map_err(|_| TokenizeError::InvalidEscapeSequence('u'))?;
                                let decoded_char = char::from_u32(code_point as u32)
                                    .ok_or(TokenizeError::InvalidEscapeSequence('u'))?;
                                string_contents.push(decoded_char);
                            }
                            Some(unknown) => {
                                return Err(TokenizeError::InvalidEscapeSequence(unknown));
                            }
                            None => return Err(TokenizeError::UnexpectedEof),
                        }
                    } else {
                        string_contents.push(c);
                        chars.next();
                    }
                }

                if !closed {
                    return Err(TokenizeError::UnclosedQuotes);
                }
                tokens.push(Token::String(string_contents));
            }

            _ if ch.is_ascii_digit() || ch == '-' => {
                let mut num_str = String::new();
                let mut has_decimal = false;
                let mut has_exponent = false;

                // Consume an optional leading minus sign
                if ch == '-' {
                    num_str.push('-');
                    chars.next();
                }

                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        num_str.push(c);
                        chars.next();
                    } else if c == '.' && !has_decimal && !has_exponent {
                        num_str.push(c);
                        has_decimal = true;
                        chars.next();
                    } else if (c == 'e' || c == 'E') && !has_exponent {
                        num_str.push(c);
                        has_exponent = true;
                        chars.next();
                        // consume optional exponent sign
                        if let Some(&next_c) = chars.peek() {
                            if next_c == '+' || next_c == '-' {
                                num_str.push(next_c);
                                chars.next();
                            }
                        }
                    } else {
                        break;
                    }
                }

                let token = num_str
                    .parse::<f64>()
                    .map(Token::Number)
                    .map_err(|_| TokenizeError::InvalidNumberFormat(num_str))?;

                tokens.push(token);
            }

            _ => return Err(TokenizeError::CharNotRecognized(ch)),
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_tokenizer_pipeline() {
        let input = r#"{
            "id": 101,
            "balance": -45.5,
            "verified": true,
            "tags": [null, "user"]
        }"#;

        let expected = vec![
            Token::LeftBrace,
            Token::String("id".to_string()),
            Token::Colon,
            Token::Number(101.0),
            Token::Comma,
            Token::String("balance".to_string()),
            Token::Colon,
            Token::Number(-45.5),
            Token::Comma,
            Token::String("verified".to_string()),
            Token::Colon,
            Token::True,
            Token::Comma,
            Token::String("tags".to_string()),
            Token::Colon,
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::String("user".to_string()),
            Token::RightBracket,
            Token::RightBrace,
        ];

        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_escapes_and_scientific_notations() {
        let input = r#"[ "line\nbreak", 6.022e+23, -0e-5 ]"#;
        let tokens = tokenize(input).unwrap();

        let expected = vec![
            Token::LeftBracket,
            Token::String("line\nbreak".to_string()),
            Token::Comma,
            Token::Number(6.022e23),
            Token::Comma,
            Token::Number(-0.0),
            Token::RightBracket,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_mid_number_dash_is_error() {
        // A dash mid-number should NOT be consumed as part of the token
        let tokens = tokenize("1-2").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Number(1.0));
    }

    #[test]
    fn test_unclosed_string() {
        let input = r#""missing quote"#;
        assert_eq!(tokenize(input), Err(TokenizeError::UnclosedQuotes));
    }

    #[test]
    fn test_bad_escape_sequence() {
        let input = r#""bad \x escape""#;
        assert_eq!(
            tokenize(input),
            Err(TokenizeError::InvalidEscapeSequence('x'))
        );
    }

    #[test]
    fn test_valid_unicode_escapes() {
        let input = r#"[ "Copyright \u00A9", "\u231B" ]"#;
        let tokens = tokenize(input).unwrap();

        let expected = vec![
            Token::LeftBracket,
            Token::String("Copyright ©".to_string()),
            Token::Comma,
            Token::String("⌛".to_string()),
            Token::RightBracket,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_invalid_unicode_hex() {
        let input = r#""\u12z4""#;
        assert_eq!(
            tokenize(input),
            Err(TokenizeError::InvalidEscapeSequence('u'))
        );
    }

    #[test]
    fn test_short_unicode_escape() {
        let input = r#""\u00A""#;
        assert_eq!(
            tokenize(input),
            Err(TokenizeError::InvalidEscapeSequence('u'))
        );
    }

    #[test]
    fn test_error_display() {
        assert_eq!(TokenizeError::UnclosedQuotes.to_string(), "unclosed quotes");
        assert_eq!(
            TokenizeError::CharNotRecognized('!').to_string(),
            "character not recognized: '!'"
        );
        assert_eq!(
            TokenizeError::InvalidEscapeSequence('x').to_string(),
            "invalid escape sequence: '\\x'"
        );
    }
}
