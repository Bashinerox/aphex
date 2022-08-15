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
    Let,
    Td,

    Print,
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

// impl FromStr for Type {

//     type Err = ();

//     fn from_str(input: &str) -> Result<Type, Self::Err> {
//         match input {
//             "void"  => Ok(Foo::Bar),
//             "i32"  => Ok(Foo::Baz),
//             "i64"  => Ok(Foo::Bat),
//             "f32" => Ok(Foo::Quux),
//             _      => Err(()),
//         }
//     }
// }



#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Value>),
    Func(String),
}

impl Value {
    pub fn num(self, span: Span) -> Result<f64, Error> {
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
    Let(String, Box<Spanned<Self>>),
    Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Print(Box<Spanned<Self>>),
    
}

#[derive(Debug, Clone)]
pub struct Function {
    pub return_type: String,
    pub params: Vec<(String, String)>,
    pub body: Spanned<Expr>,
    //body: Expr,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub funcs: Vec<(String, Function)>,
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
            Token::Td => write!(f, ":"),
            Token::Let => write!(f, "let"),
            Token::Print => write!(f, "print"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
        }
    }
}