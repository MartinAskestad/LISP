mod environment;
mod lexer;
mod parser;
mod program;
mod value;

use std::{cell::RefCell, rc::Rc};

use linefeed::{Interface, ReadResult};
use value::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = Interface::new("λ ")?;
    let mut env = Rc::new(RefCell::new(environment::Environment::new()));
    reader.set_prompt(format!("{}", "λ ").as_ref())?;
    while let ReadResult::Input(input) = reader.read_line()? {
        if input.eq("quit") {
            break;
        }
        let val = program::evaluate(input.as_ref(), &mut env)?;
        match val {
            Value::Nil => println!("nil"),
            Value::Number(n) => println!("{n}"),
            Value::Symbol(s) => println!("{s}"),
            Value::Lambda(args, body) => {
                println!("fn(");
                for arg in args {
                    println!("{} ", arg);
                }
                println!(")");
                for expr in body {
                    println!(" {}", expr);
                }
            }
            _ => println!("{val}"),
        }
    }
    Ok(())
}
