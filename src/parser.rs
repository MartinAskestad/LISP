use std::error::Error;
use std::fmt;

use crate::{
    lexer::{tokenize, Token},
    value::Value,
};

#[derive(Debug)]
pub struct ParseError {
    err: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error: {}", self.err)
    }
}

impl Error for ParseError {}

pub fn parse(program: &str) -> Result<Value, ParseError> {
    let tok_res = tokenize(program).unwrap();
    let mut tokens = tok_res.into_iter().rev().collect::<Vec<_>>();
    let mut expressions: Vec<Value> = Vec::new();
    while !tokens.is_empty() {
        let parsed_expression = parse_expression(&mut tokens)?;
        expressions.push(parsed_expression);
    }

    if expressions.len() == 1 {
        Ok(expressions.pop().unwrap())
    } else {
        Ok(Value::List(expressions))
    }
}

fn parse_expression(tokens: &mut Vec<Token>) -> Result<Value, ParseError> {
    let token = tokens.pop();
    match token {
        Some(Token::Number(n)) => Ok(Value::Number(n)),
        Some(Token::Symbol(s)) => Ok(Value::Symbol(s)),
        Some(Token::LParen) => {
            let mut list: Vec<Value> = Vec::new();
            while !tokens.is_empty() {
                if tokens.last() == Some(&Token::RParen) {
                    tokens.pop();
                    return Ok(Value::List(list));
                } else {
                    let parsed_expression = parse_expression(tokens)?;
                    list.push(parsed_expression);
                }
            }
            Err(ParseError {
                err: "Unbalanced parentheses".to_string(),
            })
        }
        Some(Token::RParen) => Err(ParseError {
            err: "Unexpected closing parenthesis".to_string(),
        }),
        None => Err(ParseError {
            err: "Unexpected end of input".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_simple_sexpr() {
        let nodes = parse("(print 5)").unwrap();
        assert_eq!(
            nodes,
            Value::List(vec![Value::Symbol("print".to_string()), Value::Number(5.),])
        );
    }

    #[test]
    fn test_one_small_program() {
        let program = "(let b 1.0)
                       (let h 14)
                       (print (/ (* b h) 2))";
        let nodes = parse(program).unwrap();
        assert_eq!(
            nodes,
            Value::List(vec![
                Value::List(vec![
                    Value::Symbol("let".to_string()),
                    Value::Symbol("b".to_string()),
                    Value::Number(1.0),
                ]),
                Value::List(vec![
                    Value::Symbol("let".to_string()),
                    Value::Symbol("h".to_string()),
                    Value::Number(14.),
                ]),
                Value::List(vec![
                    Value::Symbol("print".to_string()),
                    Value::List(vec![
                        Value::Symbol("/".to_string()),
                        Value::List(vec![
                            Value::Symbol("*".to_string()),
                            Value::Symbol("b".to_string()),
                            Value::Symbol("h".to_string()),
                        ]),
                        Value::Number(2.),
                    ]),
                ]),
            ])
        );
    }

    #[test]
    fn test_conditional() {
        let nodes = parse(
            "(cond
                             ((gt x 0) Positive)
                             ((eq x 0) Zero)
                             ((lt x 0) Negative))",
        )
        .unwrap();
        assert_eq!(
            nodes,
            Value::List(vec![
                Value::Symbol("cond".to_string()),
                Value::List(vec![
                    Value::List(vec![
                        Value::Symbol("gt".to_string()),
                        Value::Symbol("x".to_string()),
                        Value::Number(0.0)
                    ]),
                    Value::Symbol("Positive".to_string())
                ]),
                Value::List(vec![
                    Value::List(vec![
                        Value::Symbol("eq".to_string()),
                        Value::Symbol("x".to_string()),
                        Value::Number(0.0)
                    ]),
                    Value::Symbol("Zero".to_string())
                ]),
                Value::List(vec![
                Value::List(vec![
                    Value::Symbol("lt".to_string()),
                    Value::Symbol("x".to_string()),
                    Value::Number(0.0)
                ]),
                Value::Symbol("Negative".to_string())
                ]),
            ])
        );
    }
}

