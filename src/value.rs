use crate::{Callable, Literal};
use core::fmt;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    String(Rc<String>),
    Number(f64),
    Function(Rc<RefCell<dyn Callable>>),
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Nil => Value::Nil,
            Literal::Boolean(value) => Value::Boolean(value),
            Literal::String(value) => Value::String(value),
            Literal::Number(value) => Value::Number(value),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => std::ptr::addr_eq(a.as_ptr(), b.as_ptr()),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(value) => {
                if *value {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Value::String(value) => write!(f, "{}", *value),
            Value::Number(value) => write!(f, "{value}"),
            Value::Function(value) => write!(f, "{}", value.borrow().name()),
        }
    }
}
