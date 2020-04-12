use num_bigint::BigUint;
use num_rational::BigRational;
use num_traits::identities::Zero;
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

    fn empty_ctx(expr: &str) -> Result<BigRational, EvalError> {
        let ctx = ConcreteContext::new();
        with_ctx(&ctx, expr)
    }

    fn with_ctx<'a>(ctx: &ConcreteContext<'_>, expr: &'a str) -> Result<BigRational, EvalError> {
        let expr = ExprParser::new()
            .parse(expr)
            .unwrap();
        expr.evaluate(ctx, &mut |_, _| ())
    }

    fn test_scoring<'a>(expected: &'a mut Vec<(&'static str, Vec<BigRational>)>)
        -> impl FnMut(&str, &[BigRational]) + 'a
    {
        move |func, args| {
            let index = expected.iter().position(|&(f, ref a)| f == func && args == &a[..])
                .expect("unexpected call to scoring callback");
            expected.swap_remove(index);
        }
    }

    #[test]
    fn it_works() {
        assert_eq!(empty_ctx("2 + 2"), Ok(BigRational::from_integer(4.into())));
    }

    #[test]
    fn handles_negatives() {
        assert_eq!(empty_ctx("2 + -3"), Ok(BigRational::from_integer((-1).into())));
    }

    #[test]
    fn div0_doesnt_explode() {
        assert_eq!(empty_ctx("2 / 0"), Err(EvalError::DivisionByZero));
    }

    #[test]
    fn unknown_variable() {
        assert_eq!(empty_ctx("3 * x"), Err(EvalError::UnknownVariable("x".to_owned())));
    }

    #[test]
    fn handles_variable() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("x".to_owned(), SymbolValue::Num(BigRational::from_integer(7.into())));
        assert_eq!(with_ctx(&ctx, "3 * x"), Ok(BigRational::from_integer(21.into())));
    }

    #[test]
    fn handles_function() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("f".to_owned(), SymbolValue::Func(FuncDef {
            argument_names: vec!["x"],
            value_expr: ExprParser::new().parse("x*x").unwrap(),
        }));

        let expr = ExprParser::new().parse("f(3) + 4").unwrap();
        let mut scoring_calls = vec![
            ("f", vec![BigRational::from_integer(3.into())]),
        ];

        let value = expr.evaluate(&ctx, &mut test_scoring(&mut scoring_calls));
        assert_eq!(value, Ok(BigRational::from_integer(13.into())));
        assert!(scoring_calls.is_empty());
    }

    #[test]
    fn detects_arity_error_early() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("f".to_owned(), SymbolValue::Func(FuncDef {
            argument_names: vec!["x"],
            value_expr: ExprParser::new().parse("x*x").unwrap(),
        }));

        let expr = ExprParser::new().parse("f(2, f(7))").unwrap();
        let mut scoring_calls = vec![];
        let value = expr.evaluate(&ctx, &mut test_scoring(&mut scoring_calls));
        assert_eq!(value, Err(EvalError::Arity {
            function: "f".to_owned(),
            expected: 1,
            actual: 2,
        }));
    }
}
