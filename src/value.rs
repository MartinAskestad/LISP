use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Symbol(String),
    List(Vec<Value>),
    Nil,
    Lambda(Vec<String>, Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::List(l) => {
                write!(f, "(")?;
                for (i, node) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", node)?;
                }
                write!(f, ")")
            }
            Value::Nil => write!(f, "nil"),
            Value::Lambda(args, body) => {
                write!(f, "fn(")?;
                for arg in args {
                    write!(f, "{} ", arg)?;
                }
                write!(f, ")")?;
                for expr in body {
                write!(f, " {}", expr)?;
                }
                Ok(())
            }
        }
    }
}
