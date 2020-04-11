use num_bigint::BigUint;
use num_rational::BigRational;
use std::borrow::Cow;
use std::collections::HashMap;

#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(grammar);
pub use grammar::ExprParser;

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
    pub fn get_variable<'n>(&self, name: &'n str) -> Result<&T, EvalError<'n>> {
        if let Some(value) = self.0.get(name) {
            if let SymbolValue::Num(n) = value {
                Ok(n)
            } else {
                Err(EvalError::NotAVariable(name))
            }
        } else {
            Err(EvalError::UnknownVariable(name))
        }
    }

    pub fn new() -> Self {
        Context(HashMap::new())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EvalError<'a> {
    UnknownVariable(&'a str),
    UnknownFunction(&'a str),
    NotAVariable(&'a str),
    NotAFunction(&'a str),
    DivisionByZero,
}

impl Expr<'_> {
    pub fn evaluate(&self,
        ctx: &ConcreteContext<'_>,
        scoring_callback: &mut impl FnMut(&'static str, Vec<BigRational>),
    ) -> Result<BigRational, EvalError<'_>> {
        Ok(match self {
            Expr::Ident(id) => ctx.get_variable(id)?.clone(),
            Expr::Num(n) => BigRational::from_integer(n.clone().into()),
            Expr::Neg(e) => -e.evaluate(ctx, scoring_callback)?,
            Expr::BinOp(lhs, op, rhs) => {
                let lhs = lhs.evaluate(ctx, scoring_callback)?;
                let rhs = rhs.evaluate(ctx, scoring_callback)?;
                match op {
                    BinOp::Add => lhs + rhs,
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        })
    }
}

/// Represents a primary command the bot might have as input, for example:
///  - `round1(1, 2)`
///  - `round1(1, 2) = 3`
///  - `round1(x, y) = x + y`
/// A query without a guess is just an expression, while the rest is a predicate.
#[derive(Clone, Debug)]
pub enum Command<'a> {
    Pred(Pred<'a>),
    Expr(Expr<'a>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp {
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug)]
pub enum Expr<'a> {
    Func(&'a str, Vec<Expr<'a>>),
    Ident(&'a str),
    If(Box<Pred<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    BinOp(Box<Expr<'a>>, BinOp, Box<Expr<'a>>),
    Neg(Box<Expr<'a>>),
    Num(BigUint),
}

#[derive(Clone, Debug)]
pub enum Pred<'a> {
    Cmp(Expr<'a>, Cmp, Expr<'a>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let ctx = ConcreteContext::new();
        let expr = ExprParser::new()
            .parse("2 + 2")
            .unwrap();
        let val = expr.evaluate(&ctx, &mut |_, _| ());
        assert_eq!(val, Ok(BigRational::from_integer(4.into())));
    }
}
