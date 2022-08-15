#![allow(non_snake_case)]


use chumsky::prelude::*;
use chumsky::Parser;

use crate::AST::Span;
use crate::AST::Token;
use crate::AST::Class;
use crate::AST::Function;
use crate::AST::Expr;
use crate::AST::Value;
use crate::AST::BinaryOp;

use crate::AST::Spanned;



pub fn function_parser() -> impl Parser<Token, (String, Function), Error = Simple<Token>> + Clone {
    let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");
    
    let function_body =  expression_parser()
    // .then(just(Token::Return))
    .delimited_by(
        just(Token::Ctrl('{')),
        just(Token::Ctrl('}'))
    )
    .recover_with(nested_delimiters(
        Token::Ctrl('{'),
        Token::Ctrl('}'),
        [
            (Token::Ctrl('('), Token::Ctrl(')')),
            (Token::Ctrl('['), Token::Ctrl(']')),
        ],
        |span| (Expr::Error, span),
    ));

    let function_parameter = 
        ident.clone()
            .then_ignore(just(Token::Ctrl(':')))
            .then(ident.clone());
            // .map(|name, typedef| {
            //     Variable {
            //         name: name
            //         returnType: typedef)
            // });

    let params = function_parameter.clone()
        .separated_by(just(Token::Ctrl(',')))
        //.allow_trailing()
        .delimited_by(
            just(Token::Ctrl('(')),
            just(Token::Ctrl(')'))
        )
        .labelled("function params");

    let function_definition = just(Token::Fn)
        .ignore_then(ident.clone())
        .then(params)
        .then_ignore(just(Token::Ctrl(':')))
        .then(ident.clone()) //return type
        .then(function_body)
        .map(|(((name, params), return_type), body)| {
            (
                name,
                Function {
                    return_type: return_type,
                    params: params,
                    body: body,
                }
            )
        })
        .labelled("function");

    function_definition
}

pub fn class_parser() -> impl Parser<Token, Vec<Class>, Error = Simple<Token>> + Clone {
    let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");
    
    let classDefinition = function_parser()
        .repeated()
        .delimited_by(
            just(Token::Ctrl('{')),
            just(Token::Ctrl('}'))
        );


    let classDecl = 
    just(Token::Class)
    .ignore_then(ident)
    .then(classDefinition.clone())
    .map(|(name, funcs)| {
        Class {
            name: name,
            funcs: funcs
        }
    });

    classDecl.repeated()
    .then_ignore(end())
}

pub fn expression_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");

    
        let raw_expr = recursive(|raw_expr| {
            let val = select! {
                Token::Null => Expr::Value(Value::Null),
                Token::Bool(x) => Expr::Value(Value::Bool(x)),
                Token::Num(n) => Expr::Value(Value::Num(n.parse().unwrap())),
                Token::Str(s) => Expr::Value(Value::Str(s)),
            }
            .labelled("value");


            // A list of expressions
            let items = raw_expr
                .clone()
                .separated_by(just(Token::Ctrl(',')))
                .allow_trailing();

            let list = items
                .clone()
                .delimited_by(just(Token::Ctrl('[')), just(Token::Ctrl(']')))
                .map(Expr::List);

            // 'Atoms' are expressions that contain no ambiguity
            let atom = val
                .or(ident.map(Expr::Local))
//                .or(list)
                .map_with_span(|expr, span| (expr, span))
                // Atoms can also just be normal expressions, but surrounded with parentheses
                .or(raw_expr
                    .clone()
                    .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))))
                // Attempt to recover anything that looks like a parenthesised expression but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('('),
                    Token::Ctrl(')'),
                    [
                        (Token::Ctrl('['), Token::Ctrl(']')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| (Expr::Error, span),
                ))
                // Attempt to recover anything that looks like a list but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('['),
                    Token::Ctrl(']'),
                    [
                        (Token::Ctrl('('), Token::Ctrl(')')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| (Expr::Error, span),
                ));

            // Function calls have very high precedence so we prioritise them
            let call = atom
                .then(
                    items
                        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                        .map_with_span(|args, span: Span| (args, span))
                        .repeated(),
                )
                .foldl(|f, args| {
                    let span = f.1.start..args.1.end;
                    (Expr::Call(Box::new(f), args.0), span)
                });

            // Product ops (multiply and divide) have equal precedence
            let op = just(Token::Op("*".to_string()))
                .to(BinaryOp::Mul)
                .or(just(Token::Op("/".to_string())).to(BinaryOp::Div));
            let product = call
                .clone()
                .then(op.then(call).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            // Sum ops (add and subtract) have equal precedence
            let op = just(Token::Op("+".to_string()))
                .to(BinaryOp::Add)
                .or(just(Token::Op("-".to_string())).to(BinaryOp::Sub));
            let sum = product
                .clone()
                .then(op.then(product).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            // Comparison ops (equal, not-equal) have equal precedence
            let op = just(Token::Op("==".to_string()))
                .to(BinaryOp::Eq)
                .or(just(Token::Op("!=".to_string())).to(BinaryOp::NotEq));
            let compare = sum
                .clone()
                .then(op.then(sum).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            compare
        }); 

     // A let statement
     let let_statement = just(Token::Let)
     .ignore_then(ident)
     .then_ignore(just(Token::Op("=".to_string())))
     .then(raw_expr.clone())
     .then_ignore(just(Token::Ctrl(';')))
     .map_with_span(|(name, val), span: Span| {
         (Expr::Let(name, Box::new(val)), span)
     });

 let expression_statement = 
     raw_expr.clone()
     .then_ignore(just(Token::Ctrl(';')));

 let statement = expression_statement.clone()
     .or(let_statement.clone());
 let statement = let_statement.clone();


 // let block = expr
 // .clone()
 // .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
 // // Attempt to recover anything that looks like a block but contains errors
 // .recover_with(nested_delimiters(
 //     Token::Ctrl('{'),
 //     Token::Ctrl('}'),
 //     [
 //         (Token::Ctrl('('), Token::Ctrl(')')),
 //         (Token::Ctrl('['), Token::Ctrl(']')),
 //     ],
 //     |span| (Expr::Error, span),
 // ));

 // let block_expr = block
 //     //.or(if_)
 //     .labelled("block");

 // let block_chain = block_expr
 // .clone()
 // .then(block_expr.clone().repeated())
 // .foldl(|a, b| {
 //     let span = a.1.start..b.1.end;
 //     (Expr::Then(Box::new(a), Box::new(b)), span)
 // });

 
 statement.clone()
     .then(statement.clone().repeated())
     .foldl(|a, b| {
         let span = a.1.clone(); // TODO: Not correct
         (
             Expr::Then(
                 Box::new(a),
                 Box::new(b),
             ),
             span,
         )
     })
}