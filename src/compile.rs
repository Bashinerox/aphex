
use std::collections::HashMap;

use crate::Value;
use crate::Spanned;
use crate::Expr;
use crate::FunctionDefinition;
use crate::Error;
use crate::BinaryOp;
use crate::ast::FunctionSignature;


use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Linkage;
use inkwell::module::Module;

use inkwell::types::*;


use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, BasicValue, IntValue, FloatValue, FunctionValue, PointerValue};

use inkwell::OptimizationLevel;
use inkwell::FloatPredicate;


use std::error::Error as InkwellError;


pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module:  &'a Module<'ctx>,
}



impl<'a, 'ctx> Compiler<'a, 'ctx> {
    /// Creates a new stack allocation instruction in the entry block of the function.
    fn create_entry_block_alloca(&self, name: &str, var_type: BasicTypeEnum<'ctx>, fn_value: &FunctionValue<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = fn_value.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(var_type, name)
    }

    pub fn to_type(&self, typename: &str) -> BasicTypeEnum<'ctx> {
        match typename {
            "f32" => self.context.f32_type().into(),
            "f64" => self.context.f64_type().into(),
            "i32" => self.context.i32_type().into(),
            "i64" => self.context.i64_type().into(),
            &_ => panic!("unknown type {}", typename)
        }
    }

    pub fn compile_function(
        &self,
        name: &String,
        func: &FunctionDefinition,
        func_map: &HashMap<String, FunctionSignature>,
        variables: &mut HashMap<String, PointerValue<'ctx>>
    ) -> Result<(FunctionType, FunctionValue<'ctx>), Error> {

        let param_types: Vec<BasicMetadataTypeEnum> = func.signature.params
            .iter()
            .map(|(_name, param_type)| Into::<BasicMetadataTypeEnum>::into(self.to_type(param_type)))
            .collect::<Vec<BasicMetadataTypeEnum>>();

        let return_type = self.to_type(&func.signature.return_type);
        let func_type = return_type.fn_type(param_types.as_slice(), false);

        //TODO: set up linkage
        let function = self.module.add_function(name, func_type, Some(Linkage::External));

        let entry_point = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_point);
        let compilation_result = self.compile_expression(&func.body, &func_map, variables, &function);
        match compilation_result {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        Ok((func_type, function))
    }


    pub fn compile_expression( &self,
        expr: &Spanned<Expr>,
        funcs: &HashMap<String, FunctionSignature>,
        variables: &mut HashMap<String, PointerValue<'ctx>>,
        current_function: &FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum, Error> {
        match &expr.0 {

            // Error expressions only get created by parser errors, so cannot exist in a valid AST
            Expr::Error => unreachable!(),

            Expr::Value(val) => {
                match val {
                    Value::Null => todo!(),
                    Value::Bool(_) => todo!(),
                    Value::Num(n) => Ok(self.context.i32_type().const_int(*n as u64, false).as_basic_value_enum()),
                    Value::Str(_) => todo!(),
                    Value::List(_) => todo!(),
                    Value::Func(_) => todo!(),
                }
            }

            Expr::List(_items) => {
                // Value::List(
                //     items
                //         .iter()
                //         .map(|item| self.compile(item, funcs, stack))
                //         .collect::<Result<_, _>>()?,
                // )
                panic!("List unimplemented!");
            }
            
            Expr::Local(name) => {
                match variables.get(name.as_str()) {
                    Some(var) => Ok(self.builder.build_load(*var, name.as_str())),
                    None => Err(Error{
                        msg: format!("The variable named {} does not exist.", name),
                        span: expr.1.clone()
                    }),
                }
            }
            
            Expr::Var(var_name, typename, val) => {
                // println!("let statement");
                // let val = self.compile(val, funcs, stack)?;
                // stack.push((local.clone(), val.clone()));
                // val

                let typ = self.to_type(typename);

                let alloca = self.create_entry_block_alloca(var_name, typ, current_function);
                let initial_val = self.compile_expression(val, funcs, variables, current_function);
                
                match initial_val {
                    Ok(expr) => {
                        self.builder.build_store(alloca, expr);
                        variables.insert(var_name.to_string(), alloca);

                        return initial_val;
                    },
                    Err(error) => {
                        return Err(error);
                    }
                }
            }
            
            Expr::Then(a, b) => {
                self.compile_expression(a, funcs, variables, current_function)?;
                self.compile_expression(b, funcs, variables, current_function)
            }

            Expr::Binary(a, BinaryOp::Add, b) => {
                let lhs = self.compile_expression(a, funcs, variables, current_function)?;
                let rhs = self.compile_expression(b, funcs, variables, current_function)?;
                
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(_), BasicValueEnum::IntValue(_))
                        => Ok(BasicValueEnum::IntValue(self.builder.build_int_add(
                            lhs.into_int_value(),
                            rhs.into_int_value(), "intadd")
                        )),

                    (BasicValueEnum::IntValue(_), BasicValueEnum::FloatValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::IntValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::FloatValue(_))
                        => Ok(BasicValueEnum::FloatValue(self.builder.build_float_add(
                            lhs.into_float_value(),
                            rhs.into_float_value(), "fltadd")
                        )),

                    _ => panic!("binary addition: unknown type")
                }
            }

            Expr::Binary(a, BinaryOp::Sub, b) => {
                let lhs = self.compile_expression(a, funcs, variables, current_function)?;
                let rhs = self.compile_expression(b, funcs, variables, current_function)?;
                
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(_), BasicValueEnum::IntValue(_))
                        => Ok(BasicValueEnum::IntValue(self.builder.build_int_sub(
                            lhs.into_int_value(),
                            rhs.into_int_value(), "intsub")
                        )),

                    (BasicValueEnum::IntValue(_), BasicValueEnum::FloatValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::IntValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::FloatValue(_))
                        => Ok(BasicValueEnum::FloatValue(self.builder.build_float_sub(
                            lhs.into_float_value(),
                            rhs.into_float_value(), "fltsub")
                        )),

                    _ => panic!("binary subtraction: unknown type")
                }
            }

            Expr::Binary(a, BinaryOp::Mul, b) => {
                let lhs = self.compile_expression(a, funcs, variables, current_function)?;
                let rhs = self.compile_expression(b, funcs, variables, current_function)?;
                
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(_), BasicValueEnum::IntValue(_))
                        => Ok(BasicValueEnum::IntValue(self.builder.build_int_mul(
                            lhs.into_int_value(),
                            rhs.into_int_value(), "intmul")
                        )),

                    (BasicValueEnum::IntValue(_), BasicValueEnum::FloatValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::IntValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::FloatValue(_))
                        => Ok(BasicValueEnum::FloatValue(self.builder.build_float_mul(
                            lhs.into_float_value(),
                            rhs.into_float_value(), "fltmul")
                        )),

                    _ => {
                        panic!("fltmul: unknown type combination!");
                    }
                }
            }

            Expr::Binary(a, BinaryOp::Div, b) => {
                let lhs = self.compile_expression(a, funcs, variables, current_function)?;
                let rhs = self.compile_expression(b, funcs, variables, current_function)?;
                
                //println!("{} + {}", a_, b_);
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(_), BasicValueEnum::IntValue(_))
                        => Ok(BasicValueEnum::IntValue(self.builder.build_int_signed_div(
                            lhs.into_int_value(),
                            rhs.into_int_value(), "intdiv")
                        )),

                    (BasicValueEnum::IntValue(_), BasicValueEnum::FloatValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::IntValue(_)) |
                    (BasicValueEnum::FloatValue(_), BasicValueEnum::FloatValue(_))
                        => Ok(BasicValueEnum::FloatValue(self.builder.build_float_div(
                            lhs.into_float_value(),
                            rhs.into_float_value(), "fltdiv")
                        )),

                    _ => {
                        panic!("fltdiv: unknown type combination!");
                    }
                }
            }

            Expr::Binary(a, BinaryOp::Eq, b) => {
                unimplemented!("operator==");
            }

            Expr::Binary(a, BinaryOp::NotEq, b) => {
                //Value::Bool(self.compile(a, funcs, stack)? != self.compile(b, funcs, stack)?)
                unimplemented!("operator!=");
            }

            Expr::Call(func, args) => {
                // let f = self.compile(func, funcs, stack)?;
                // match f {
                //     Value::Func(name) => {
                //         let f = &funcs[&name];
                //         let mut stack = if f.params.len() != args.len() {
                //             return Err(Error {
                //                 span: expr.1.clone(),
                //                 msg: format!("'{}' called with wrong number of arguments (expected {}, found {})", name, f.params.len(), args.len()),
                //             });
                //         } else {
                //             f.params
                //                 .iter()
                //                 .zip(args.iter())
                //                 .map(|((name, _type), arg)| Ok((name.clone(), self.compile(arg, funcs, stack)?)))
                //                 .collect::<Result<_, _>>()?
                //         };
                //         self.compile(&f.body, funcs, &mut stack)?
                //     }
                //     f => {
                //         return Err(Error {
                //             span: func.1.clone(),
                //             msg: format!("'{:?}' is not callable", f),
                //         })
                //     }
                // }
                return Err(Error {
                    span: expr.1.clone(),
                    msg: format!("function call unimplemented"),
                });

            }

            Expr::If(cond, a, b) => {
                // let c = self.compile(cond, funcs, stack)?;
                // match c {
                //     Value::Bool(true) => self.compile(a, funcs, stack)?,
                //     Value::Bool(false) => self.compile(b, funcs, stack)?,
                //     c => {
                //         return Err(Error {
                //             span: cond.1.clone(),
                //             msg: format!("Conditions must be booleans, found '{:?}'", c),
                //         })
                //     }
                // }
                return Err(Error {
                    span: expr.1.clone(),
                    msg: format!("if statement unimplemented"),
                });
            }

            Expr::Ret(ret_expr) => {
                let res = self.compile_expression(ret_expr, funcs, variables, current_function);
                match res {
                    Ok(ret_val) => {
                        self.builder.build_return(Some(&ret_val));
                    },
                    Err(err) => {
                        return Err(err);
                    }
                    
                }
                res
            },
        }
    }
}
