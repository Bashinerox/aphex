#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use ast::{Spanned, Expr, Value};
use chumsky::{prelude::*, Stream};
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{TargetMachine, Target, InitializationConfig, RelocMode, CodeModel, FileType};
use inkwell::types::IntType;
use inkwell::values::PointerValue;
use inkwell::passes::PassManager;

use std::path::Path;
use std::{collections::HashMap, env, fmt, fs};

//use ariadne:;

pub mod parser;
pub mod lexer;
pub mod compile;
pub mod ast;

use crate::parser::class_parser;
use crate::lexer::lexer;
use crate::compile::Compiler;

use crate::ast::Token;
// use crate::AST::Spanned;
// use crate::AST::Expr;
// use crate::AST::Value;
use crate::ast::Function;
use crate::ast::Error;
use crate::ast::BinaryOp;
use chumsky::Parser;

fn print_splash() {
    println!("┌                               ┐");
    println!("| Chip: right off the ol' block |");
    println!("└                               ┘");
}

fn get_host_cpu_name() -> String {
    TargetMachine::get_host_cpu_name().to_string()
}

fn get_host_cpu_features() -> String {
    TargetMachine::get_host_cpu_features().to_string()
}

fn ptr_sized_int_type<'ctx>(target_machine: &TargetMachine, context: &'ctx Context) -> IntType<'ctx> {
    let target_data = target_machine.get_target_data();
    context.ptr_sized_int_type(&target_data, None)
}

fn apply_target_to_module<'ctx>(target_machine: &TargetMachine, module: &Module) {
    module.set_triple(&target_machine.get_triple());
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
}


fn get_native_target_machine() -> TargetMachine {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    target
        .create_target_machine(
            &target_triple,
            &get_host_cpu_name(),
            &get_host_cpu_features(),
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap()
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    print_splash();

    // let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument"))
    //     .expect("Failed to read file");

     let src = fs::read_to_string(env::args().nth(1).unwrap_or("data/testProgram.aph".into()))
         .expect("Failed to read file");

    let context = inkwell::context::Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let pass_manager = PassManager::<Module>::create(());

    pass_manager.add_instruction_combining_pass();
    pass_manager.add_reassociate_pass();
    pass_manager.add_gvn_pass();
    pass_manager.add_cfg_simplification_pass();
    pass_manager.add_basic_alias_analysis_pass();
    pass_manager.add_promote_memory_to_register_pass();
    pass_manager.add_instruction_combining_pass();
    pass_manager.add_reassociate_pass();

    let compiler = Compiler {
        context: &context,
        builder: &builder,
        module: &module,
    };


    let (tokens, errs) = lexer().parse_recovery(src.as_str());

    let parse_errs = if let Some(tokens) = tokens {
        //dbg!(tokens.clone());
        let len = src.chars().count();
        let (ast, parse_errs) =
            class_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        //dbg!(ast.clone());


        if let Some(classes) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            for class in classes {
                println!("compiling {}...", class.name);
                let func_map: HashMap<String, Function> = class.funcs.clone().into_iter().collect();
                let mut variable_map: HashMap<String, PointerValue> = HashMap::new();


                for func in class.funcs.clone() {
                    println!("compiling {}::{}...", class.name, func.0);
                    let result = compiler.compile_function(&func.0, &func.1, &func_map, &mut variable_map);

                    match result {
                        Ok(result) => (),
                        Err(error) => println!("{}", error.msg.as_str())
                    };
                }
            }
        }

        pass_manager.run_on(&module);
        module.print_to_stderr();

        module.write_bitcode_to_path(Path::new("testoutput.bc"));
        


        let target_machine = get_native_target_machine();
        apply_target_to_module(&target_machine, &module);
        target_machine.write_to_file(&module, FileType::Object, Path::new("output.o"));


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
