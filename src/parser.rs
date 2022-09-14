
use chumsky::prelude::*;
use chumsky::Parser;

use crate::ast;
use crate::ast::Span;
use crate::ast::Token;
use crate::ast::Class;
use crate::ast::FunctionSignature;
use crate::ast::NamedFunction;
use crate::ast::FunctionDefinition;
use crate::ast::Expr;
use crate::ast::Value;
use crate::ast::BinaryOp;
use crate::ast::ProgramUnit;

use crate::ast::Spanned;


pub fn function_declaration_parser() -> impl Parser<Token, (String, FunctionSignature), Error=Simple<Token>> + Clone {
    let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");

    let template_list = ident.clone()
        .separated_by(just(Token::Ctrl(',')))
        .delimited_by(
            just(Token::Ctrl('<')),
            just(Token::Ctrl('>'))
        )
        .repeated().at_most(1)
        .labelled("template type list");

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

    let function_declaration = just(Token::Fn)
        .ignore_then(ident.clone())
        .then(template_list)
        .then(params)
        .then_ignore(just(Token::Op("->".into())))
        .then(ident.clone());

        function_declaration.map(| (((name, generic_params), params), return_type) | {
            (
                name,
                FunctionSignature {
                    return_type: return_type,
                    generic_params:
                        if generic_params.len() > 0 {
                            generic_params[0].clone()
                        } else {
                            Vec::<String>::new()
                        },
                    params: params,
                }
            )        
        })
}

pub fn function_definition_parser() -> impl Parser<Token, NamedFunction, Error = Simple<Token>> + Clone {
    //let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");

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

    let function_definition =
        function_declaration_parser()
        .then(function_body)
        .map(|((name, signature), body)| {
            NamedFunction {
                name: name,
                definition: FunctionDefinition {
                    signature: signature,
                    body: body,
                }
            }
        })
        .labelled("function");

    function_definition
}

//parse the class.
//outputs a list of tuple of (class, function defintion list)
pub fn class_parser() -> impl Parser<Token, Vec<ProgramUnit>, Error = Simple<Token>> + Clone {
    let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");
    
    let classDefinition = function_definition_parser()
        .map(|function| {
            ProgramUnit::Function(function)
        })
        .repeated()
        .delimited_by(
            just(Token::Ctrl('{')),
            just(Token::Ctrl('}'))
        );

    let classDecl = 
        just(Token::Class)
        .ignore_then(ident)
        .map(|name| {
            ProgramUnit::Class(Class{
                name: name,
            })
        })
        .chain(classDefinition.clone())
        .map(|funcs| {
            funcs
        });

    classDecl
}

pub fn program_parser() -> impl Parser<Token, Vec<ProgramUnit>, Error = Simple<Token>> + Clone {
    class_parser()
    .or(
        function_definition_parser()
        .map(|function| {
            vec![ProgramUnit::Function(function)]
        })
    )
    .repeated()
    .then_ignore(end())
    .flatten()
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

            let _list = items
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

     // A var statement
     let var_statement = just(Token::Var)
     .ignore_then(ident)
     .then_ignore(just(Token::As))
     .then(ident)
     .then_ignore(just(Token::Op("=".to_string())))
     .then(raw_expr.clone())
     .then_ignore(just(Token::Ctrl(';')))
     .map_with_span(|((name, typename), val), span: Span| {
         (Expr::Var(name, typename, Box::new(val)), span)
     });

     let ret_statement = just(Token::Ret)
     .ignore_then(raw_expr.clone())
     .then_ignore(just(Token::Ctrl(';')))
     .map_with_span(|val, span: Span| {
         (Expr::Ret(Box::new(val)), span)
     });


 let expression_statement = 
     raw_expr.clone()
     .then_ignore(just(Token::Ctrl(';')));

  let statement = expression_statement.clone()
      .or(var_statement.clone())
      .or(ret_statement);

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