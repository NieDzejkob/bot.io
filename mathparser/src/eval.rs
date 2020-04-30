use num_rational::BigRational;
use num_traits::identities::Zero;
use std::collections::HashMap;
use crate::ast::*;

pub struct FuncDef<'a> {
    argument_names: Vec<&'a str>,
    value_expr: Expr<'a>,
}

pub enum SymbolValue<'a, T> {
    Func(FuncDef<'a>),
    Num(T),
}

pub struct Context<'a, T>(pub HashMap<String, SymbolValue<'a, T>>);
pub type ConcreteContext<'a> = Context<'a, BigRational>;

impl<T> Context<'_, T> {
    pub fn get_variable(&self, name: &str) -> Result<&T, EvalError> {
        if let Some(value) = self.0.get(name) {
            if let SymbolValue::Num(n) = value {
                Ok(n)
            } else {
                Err(EvalError::NotAVariable(name.to_owned()))
            }
        } else {
            Err(EvalError::UnknownVariable(name.to_owned()))
        }
    }

    pub fn get_function(&self, name: &str) -> Result<&FuncDef<'_>, EvalError> {
        if let Some(value) = self.0.get(name) {
            if let SymbolValue::Func(f) = value {
                Ok(f)
            } else {
                Err(EvalError::NotAFunction(name.to_owned()))
            }
        } else {
            Err(EvalError::UnknownFunction(name.to_owned()))
        }
    }

    pub fn new() -> Self {
        Context(HashMap::new())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    UnknownVariable(String),
    UnknownFunction(String),
    NotAVariable(String),
    NotAFunction(String),
    Arity {
        function: String,
        expected: usize,
        actual: usize,
    },
    DivisionByZero,
}

impl<'a> Expr<'a> {
    pub fn evaluate<'ctx>(&self,
        ctx: &'ctx ConcreteContext<'a>,
        scoring_callback: &mut impl FnMut(&str, &[BigRational]),
    ) -> Result<BigRational, EvalError> {
        Ok(match self {
            &Expr::Func(id, ref args) => {
                let func = ctx.get_function(id)?;
                if func.argument_names.len() != args.len() {
                    return Err(EvalError::Arity {
                        function: id.to_owned(),
                        expected: func.argument_names.len(),
                        actual: args.len(),
                    });
                }

                let values = args.iter()
                    .map(|arg| arg.evaluate(ctx, scoring_callback))
                    .collect::<Result<Vec<_>, _>>()?;
                scoring_callback(id, &values);
                let fn_ctx = Context(func.argument_names.iter().map(|&name| name.to_owned())
                                       .zip(values.into_iter().map(SymbolValue::Num))
                                       .collect());
                func.value_expr.evaluate(&fn_ctx, scoring_callback)?
            }
            Expr::Ident(id) => ctx.get_variable(id)?.clone(),
            Expr::If(cond, then, otherwise) => {
                let expr = if cond.evaluate(ctx, scoring_callback)? {
                    then
                } else {
                    otherwise
                };
                expr.evaluate(ctx, scoring_callback)?
            }
            Expr::BinOp(lhs, op, rhs) => {
                let lhs = lhs.evaluate(ctx, scoring_callback)?;
                let rhs = rhs.evaluate(ctx, scoring_callback)?;
                match op {
                    BinOp::Add => lhs + rhs,
                    BinOp::Sub => lhs - rhs,
                    BinOp::Mul => lhs * rhs,
                    BinOp::Div | BinOp::Mod if rhs.is_zero() => {
                        return Err(EvalError::DivisionByZero);
                    }
                    BinOp::Div => lhs / rhs,
                    BinOp::Mod => lhs % rhs,
                }
            }
            Expr::Neg(e) => -e.evaluate(ctx, scoring_callback)?,
            Expr::Num(n) => BigRational::from_integer(n.clone().into()),
        })
    }
}

impl<'a> Pred<'a> {
    pub fn evaluate<'ctx>(&self,
        ctx: &'ctx ConcreteContext<'a>,
        scoring_callback: &mut impl FnMut(&str, &[BigRational]),
    ) -> Result<bool, EvalError> {
        Ok(match self {
            Pred::Cmp(lhs, op, rhs) => {
                let lhs = lhs.evaluate(ctx, scoring_callback)?;
                let rhs = rhs.evaluate(ctx, scoring_callback)?;
                match op {
                    Cmp::Eq => lhs == rhs,
                    Cmp::Lt => lhs < rhs,
                    Cmp::Le => lhs <= rhs,
                    Cmp::Gt => lhs > rhs,
                    Cmp::Ge => lhs >= rhs,
                }
            }
        })
    }
}

