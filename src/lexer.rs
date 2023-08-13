use regex::Regex;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Number(f64),
    Symbol(String),
    LParen,
    RParen,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
        }
    }
}

#[derive(Debug)]
pub struct TokenError {
    ch: char,
}

impl Error for TokenError {}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unexpected character {}", self.ch)
    }
}

pub fn tokenize(program: &str) -> Result<Vec<Token>, TokenError> {
    let re = Regex::new(r"(\(|\)|\d+(\.\d+)?|[^\s()]+)").unwrap();
    let tokens: Vec<Token> = re
        .captures_iter(program)
        .map(|captures| {
            let token = captures.get(0).unwrap().as_str();
            match token {
                "(" => Token::LParen,
                ")" => Token::RParen,
                num_str if num_str.chars().all(|c| c.is_digit(10) || c == '.') => {
                    let num = f64::from_str(num_str).unwrap();
                    Token::Number(num)
                }
                _ => Token::Symbol(token.to_string()),
            }
        })
        .collect();
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_simple_sexpr() {
        let tokens = tokenize("(print 5)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Symbol("print".to_string()),
                Token::Number(5.),
                Token::RParen
            ]
        );
    }

    #[test]
    fn test_one_small_program() {
        let program = "(let b 1.0)
                       (let h 14)
                       (print (/ (* b h) 2))";
        let tokens = tokenize(program).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Symbol("let".to_string()),
                Token::Symbol("b".to_string()),
                Token::Number(1.0),
                Token::RParen,
                Token::LParen,
                Token::Symbol("let".to_string()),
                Token::Symbol("h".to_string()),
                Token::Number(14.),
                Token::RParen,
                Token::LParen,
                Token::Symbol("print".to_string()),
                Token::LParen,
                Token::Symbol("/".to_string()),
                Token::LParen,
                Token::Symbol("*".to_string()),
                Token::Symbol("b".to_string()),
                Token::Symbol("h".to_string()),
                Token::RParen,
                Token::Number(2.),
                Token::RParen,
                Token::RParen
            ]
        );
    }
}
