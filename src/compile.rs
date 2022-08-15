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
            println!("let statement");
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