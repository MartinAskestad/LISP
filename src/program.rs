use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::{Env, Environment},
    parser::parse,
    value::Value,
};

pub fn evaluate(source: &str, env: &mut Env) -> Result<Value, String> {
    if let Ok(parsed_program) = parse(source) {
        value(&parsed_program, env)
    } else {
        Err("Something hapened?".to_string())
    }
}

fn value(node: &Value, env: &mut Env) -> Result<Value, String> {
    match node {
        Value::Symbol(s) => symbol(s, env),
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::List(l) => list(l, env),
        _ => Ok(Value::Nil),
    }
}

fn symbol(s: &str, env: &mut Env) -> Result<Value, String> {
    if let Some(val) = env.borrow_mut().get(s) {
        Ok(val.clone())
    } else {
        Err(format!("Unbound symbol {}", s))
    }
}

fn list(list: &Vec<Value>, env: &mut Env) -> Result<Value, String> {
    let head = &list.first().unwrap();
    match head {
        Value::Symbol(s) => match s.as_str() {
            "not" => not(&list, env),
            "+" | "-" | "*" | "/" | "gt" | "gte" | "lt" | "lte" | "eq" => bin_op(&list, env),
            "let" => _let(&list, env),
            "fn" => _fn(&list),
            "cond" => cond(&list, env),
            _ => call(&s, &list, env),
        },
        _ => {
            let mut new_list = vec![];
            for node in list {
                let res = value(node, env)?;
                match res {
                    Value::Nil => {}
                    _ => new_list.push(res),
                }
            }
            Ok(Value::List(new_list))
        }
    }
}

fn bin_op(list: &Vec<Value>, env: &mut Env) -> Result<Value, String> {
    if list.len() < 3 {
        return Err("Insufficient number of arguments".to_string());
    }
    let op = &list.first().unwrap();
    let operands = list[1..]
        .iter()
        .map(|node| value(node, env))
        .collect::<Result<Vec<Value>, String>>()?;
    let start = match &operands.first() {
        Some(Value::Number(n)) => *n,
        _ => return Err("Operands must be numbers".to_string()),
    };
    let final_result = operands[1..].iter().try_fold(start, |acc, node| {
        if let Value::Number(n) = node {
            match op {
                Value::Symbol(s) => match s.as_str() {
                    "+" => Ok(acc + n),
                    "-" => Ok(acc - n),
                    "*" => Ok(acc * n),
                    "/" => Ok(acc / n),
                    "gt" => Ok(if acc > *n { 1.0 } else { 0.0 }),
                    "gte" => Ok(if acc >= *n { 1.0 } else { 0.0 }),
                    "lt" => Ok(if acc < *n { 1.0 } else { 0.0 }),
                    "lte" => Ok(if acc <= *n { 1.0 } else { 0.0 }),
                    "eq" => Ok(if acc == *n { 1.0 } else { 0.0 }),
                    _ => return Err(format!("Invalid operator {}", s)),
                },
                _ => return Err(format!("Operator must be a symbol")),
            }
        } else {
            return Err("Operands must be numbers".to_string());
        }
    })?;
    Ok(Value::Number(final_result))
}

fn not(list: &Vec<Value>, env: &mut Env) -> Result<Value, String> {
    if list.len() != 2 {
        return Err("Incorrect number of arguments".to_string());
    }
    if let Ok(Value::Number(n)) = value(&list[1], env) {
        Ok(Value::Number(if n == 0.0 { 1.0 } else { 0.0 }))
    } else {
        Err("Invalid argument".to_string())
    }
}

fn _let(list: &Vec<Value>, env: &mut Env) -> Result<Value, String> {
    if list.len() != 3 {
        return Err("Invalid number of arguments for let".to_string());
    }
    let symbol = match &list[1] {
        Value::Symbol(s) => s.clone(),
        _ => return Err("Invalid let".to_string()),
    };
    let val = value(&list[2], env)?;
    env.borrow_mut().set(&symbol, val);
    Ok(Value::Nil)
}

fn _fn(list: &Vec<Value>) -> Result<Value, String> {
    let args = match &list[1] {
        Value::List(l) => {
            let mut args = vec![];
            for arg in l {
                match arg {
                    Value::Symbol(s) => args.push(s.clone()),
                    _ => return Err("Invalid function argument".to_string()),
                }
            }
            args
        }
        _ => return Err("Invalid function".to_string()),
    };
    let body = match &list[2] {
        Value::List(l) => l.clone(),
        _ => return Err("Invalid function".to_string()),
    };
    Ok(Value::Lambda(args, body))
}

fn call(s: &str, list: &Vec<Value>, env: &mut Env) -> Result<Value, String> {
    let lamdba = env.borrow_mut().get(s);
    if let Some(func) = lamdba {
        match func {
            Value::Lambda(args, body) => {
                let mut new_env = Rc::new(RefCell::new(Environment::extend(env.clone())));
                for (i, arg) in args.iter().enumerate() {
                    let val = value(&list[i + 1], env)?;
                    new_env.borrow_mut().set(arg, val);
                }
                return value(&Value::List(body), &mut new_env);
            }
            _ => return Err(format!("Not a lambda: {}", s)),
        }
    } else {
        return Err(format!("Unbound symbol {}", s));
    }
}

fn cond(conds: &[Value], env: &mut Env) -> Result<Value, String> {
    for cond in &conds[1 .. conds.len()-1] {
        if let Value::List(cs) = cond {
            let res = value(&cs[0], env)?;
            if let Value::Number(n) = res {
                if n != 0.0 {
                    return value(&cs[1], env);
                }
            }
        }
    }
    value(conds.last().unwrap(), env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(+ 1 1)", &mut env).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_sub() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(- 2 1)", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_mul() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(* 2.5 2)", &mut env).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_div() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(/ 5 2)", &mut env).unwrap();
        assert_eq!(result, Value::Number(2.5));
    }

    #[test]
    fn test_gt() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(gt 5 2)", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_gte() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(gte 5 5)", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_gte_neg() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(gte 4 5)", &mut env).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_lt() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(lt 4 5)", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_eq() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(eq 4 4)", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_eq_neg() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(eq 4 5)", &mut env).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_not() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(not 1)", &mut env).unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_not2() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(not 0)", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_not_eq() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(not (eq 1 0))", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_not_gt() {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        let result = evaluate("(not (gt 0 1))", &mut env).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_simple_program() {
        let mut env= Rc::new(RefCell::new(Environment::new()));
        let source = "(let b 10)
                      (let h 14)
                      (/ (* b h) 2)";
        let res = evaluate(source, &mut env).unwrap();
        assert_eq!(res, Value::List(vec![Value::Number(70.0)]));
    }

    #[test]
    fn test_conditional() {
        let mut env= Rc::new(RefCell::new(Environment::new()));
        let source = "(let factorial (fn (n) (cond ((lt n 1) 1) (* n (factorial (- n 1))))))(factorial 5)";
        let res = evaluate(source, &mut env).unwrap();
        assert_eq!(res, Value::List(vec![Value::Number(120.0)]));
    }
}
