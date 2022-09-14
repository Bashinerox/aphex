use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use std::{collections::HashMap, env, fmt, fs, str::FromStr};


pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

pub struct Error {
    pub span: Span,
    pub msg: String,
}



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Op(String),
    Ctrl(char),
    Ident(String),
    
    Class,
    Fn,
    Var,
    Ret,
    As,
    If,
    Else,
} 

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Void,
    Bool,
    F32,
    Str,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f32),
    Str(String),
    List(Vec<Value>),
    Func(String),
}

impl Value {
    pub fn num(self, span: Span) -> Result<f32, Error> {
        if let Value::Num(x) = self {
            Ok(x)
        } else {
            Err(Error {
                span,
                msg: format!("'{}' is not a number", self),
            })
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Num(x) => write!(f, "{}", x),
            Self::Str(x) => write!(f, "{}", x),
            Self::List(xs) => write!(
                f,
                "[{}]",
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Func(name) => write!(f, "<function: {}>", name),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Error,
    Value(Value),
    List(Vec<Spanned<Self>>),
    Local(String),
    Var(String, String, Box<Spanned<Self>>),
    Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Ret(Box<Spanned<Self>>),
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub return_type: String,
    pub params: Vec<(String, String)>,
    pub generic_params: Vec<String>,
    //body: Expr,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub body: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct NamedFunction {
    pub name: String,
    pub definition: FunctionDefinition
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ProgramUnit {
    Class(Class),
    Function(NamedFunction),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Null => write!(f, "null"),
            Token::Bool(x) => write!(f, "{}", x),
            Token::Num(n) => write!(f, "{}", n),
            Token::Str(s) => write!(f, "{}", s),
            Token::Op(s) => write!(f, "{}", s),
            Token::Ctrl(c) => write!(f, "{}", c),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Class => write!(f, "class"),
            Token::Fn => write!(f, "fun"),
            Token::As => write!(f, "as"),
            Token::Var => write!(f, "let"),
            Token::Ret => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
        }
    }
}