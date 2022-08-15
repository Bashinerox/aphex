#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]


use AST::{Spanned, Expr, Value};
use chumsky::{prelude::*, Stream};
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use std::{collections::HashMap, env, fmt, fs};

//use ariadne:;

pub mod AphexParser;
pub mod AphexLexer;
pub mod AST;

use crate::AphexParser::class_parser;
use crate::AphexLexer::lexer;

use crate::AST::Token;
// use crate::AST::Spanned;
// use crate::AST::Expr;
// use crate::AST::Value;
use crate::AST::Function;
use crate::AST::Error;
use crate::AST::BinaryOp;
use chumsky::Parser;

fn main() {

    // let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument"))
    //     .expect("Failed to read file");

     let src = fs::read_to_string(env::args().nth(1).unwrap_or("data/testProgram.aph".into()))
         .expect("Failed to read file");


    let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

    let parse_errs = if let Some(tokens) = tokens {
        //dbg!(tokens.clone());
        let len = src.chars().count();
        let (ast, parse_errs) =
            class_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        //dbg!(ast.clone());

        if let Some(classes) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            for class in classes {
                //println!("{}", class.name);
                let funcMap: HashMap<String, Function> = class.funcs.clone().into_iter().collect();

                for func in class.funcs.clone() {
                    println!("{}", func.0);
                    compile(&func.1.body, &funcMap, &mut Vec::new());
                }
            }
        }
        

        // if let Some(funcs) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
        //     if let Some(main) = funcs.get("main") {
        //         assert_eq!(main.args.len(), 0);
        //         // match eval_expr(&main.body, &funcs, &mut Vec::new()) {
        //         //     Ok(val) => println!("Return value: {}", val),
        //         //     Err(e) => errs.push(Simple::custom(e.span, e.msg)),
        //         // }
        //     } else {
        //         panic!("No main function!");
        //     }
        // }

        parse_errs
    } else {
        Vec::new()
    };

    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_label(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_color(Color::Yellow),
                    )
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Must be closed before this {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}, expected {}",
                        if e.found().is_some() {
                            "Unexpected token in input"
                        } else {
                            "Unexpected end of input"
                        },
                        if e.expected().len() == 0 {
                            "something else".to_string()
                        } else {
                            e.expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ))
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Unexpected token {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
            };

            report.finish().print(Source::from(&src)).unwrap();
        });
}


fn compile(
    expr: &Spanned<Expr>,
    funcs: &HashMap<String, Function>,
    stack: &mut Vec<(String, Value)>,
) -> Result<Value, Error> {
    Ok(match &expr.0 {

        // Error expressions only get created by parser errors, so cannot exist in a valid AST
        Expr::Error => unreachable!(),

        Expr::Value(val) => val.clone(),
        
        Expr::List(items) => {
            Value::List(
                items
                    .iter()
                    .map(|item| compile(item, funcs, stack))
                    .collect::<Result<_, _>>()?,
            )
        }
        
        Expr::Local(name) => {
            stack
                .iter()
                .rev()
                .find(|(l, _)| l == name)
                .map(|(_, v)| v.clone())
                .or_else(|| Some(Value::Func(name.clone())).filter(|_| funcs.contains_key(name)))
                .ok_or_else(|| Error {
                    span: expr.1.clone(),
                    msg: format!("No such variable '{}' in scope", name),
                })?
        }
        
        Expr::Let(local, val) => {
            let val = compile(val, funcs, stack)?;
            stack.push((local.clone(), val.clone()));
            val
        }
        
        Expr::Then(a, b) => {
            compile(a, funcs, stack)?;
            compile(b, funcs, stack)?
        }

        Expr::Binary(a, BinaryOp::Add, b) => {
            let a_ = compile(a, funcs, stack)?.num(a.1.clone())?;
            let b_ = compile(b, funcs, stack)?.num(b.1.clone())?;
            
            println!("{} + {}", a_, b_);

            let num = Value::Num(a_ + b_);
            num
        }

        Expr::Binary(a, BinaryOp::Sub, b) => {
            let a_ = compile(a, funcs, stack)?.num(a.1.clone())?;
            let b_ = compile(b, funcs, stack)?.num(b.1.clone())?;
            
            println!("{} - {}", a_, b_);

            let num = Value::Num(a_ + b_);
            num
        }

        Expr::Binary(a, BinaryOp::Mul, b) => {
            let a_ = compile(a, funcs, stack)?.num(a.1.clone())?;
            let b_ = compile(b, funcs, stack)?.num(b.1.clone())?;
            
            println!("{} * {}", a_, b_);

            let num = Value::Num(a_ + b_);
            num
        }

        Expr::Binary(a, BinaryOp::Div, b) => {
            Value::Num(
                compile(a, funcs, stack)?.num(a.1.clone())?
                / compile(b, funcs, stack)?.num(b.1.clone())?,
            )
        }

        Expr::Binary(a, BinaryOp::Eq, b) => {
            Value::Bool(compile(a, funcs, stack)? == compile(b, funcs, stack)?)
        }

        Expr::Binary(a, BinaryOp::NotEq, b) => {
            Value::Bool(compile(a, funcs, stack)? != compile(b, funcs, stack)?)
        }

        Expr::Call(func, args) => {
            let f = compile(func, funcs, stack)?;
            match f {
                Value::Func(name) => {
                    let f = &funcs[&name];
                    let mut stack = if f.params.len() != args.len() {
                        return Err(Error {
                            span: expr.1.clone(),
                            msg: format!("'{}' called with wrong number of arguments (expected {}, found {})", name, f.params.len(), args.len()),
                        });
                    } else {
                        f.params
                            .iter()
                            .zip(args.iter())
                            .map(|((name, _type), arg)| Ok((name.clone(), compile(arg, funcs, stack)?)))
                            .collect::<Result<_, _>>()?
                    };
                    compile(&f.body, funcs, &mut stack)?
                }
                f => {
                    return Err(Error {
                        span: func.1.clone(),
                        msg: format!("'{:?}' is not callable", f),
                    })
                }
            }
        }

        Expr::If(cond, a, b) => {
            let c = compile(cond, funcs, stack)?;
            match c {
                Value::Bool(true) => compile(a, funcs, stack)?,
                Value::Bool(false) => compile(b, funcs, stack)?,
                c => {
                    return Err(Error {
                        span: cond.1.clone(),
                        msg: format!("Conditions must be booleans, found '{:?}'", c),
                    })
                }
            }
        }

        Expr::Print(a) => {
            let val = compile(a, funcs, stack)?;
            println!("{}", val);
            val
        }
    })

}