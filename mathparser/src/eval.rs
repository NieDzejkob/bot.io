use num_rational::BigRational;
use num_traits::identities::Zero;
use std::collections::HashMap;
use crate::ast::*;

pub struct FuncDef<'a> {
    pub name: &'a str,
    pub argument_names: Vec<&'a str>,
    pub value_expr: Expr<'a>,
}

pub enum SymbolValue<'a, T> {
    Func(FuncDef<'a>),
    Num(T),
}

pub struct Context<'a, T>(pub HashMap<String, SymbolValue<'a, T>>);
pub type ConcreteContext<'a> = Context<'a, BigRational>;

impl<T> Context<'_, T> {
    pub fn get_variable(&self, name: Span<&str>) -> Result<&T, EvalError> {
        if let Some(value) = self.0.get(name.0) {
            if let SymbolValue::Num(n) = value {
                Ok(n)
            } else {
                Err(EvalError::NotAVariable(name.map(ToOwned::to_owned)))
            }
        } else {
            Err(EvalError::UnknownVariable(name.map(ToOwned::to_owned)))
        }
    }

    pub fn get_function(&self, name: Span<&str>) -> Result<&FuncDef<'_>, EvalError> {
        if let Some(value) = self.0.get(name.0) {
            if let SymbolValue::Func(f) = value {
                Ok(f)
            } else {
                Err(EvalError::NotAFunction(name.map(ToOwned::to_owned)))
            }
        } else {
            Err(EvalError::UnknownFunction(name.map(ToOwned::to_owned)))
        }
    }

    pub fn new() -> Self {
        Context(HashMap::new())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    UnknownVariable(Span<String>),
    UnknownFunction(Span<String>),
    NotAVariable(Span<String>),
    NotAFunction(Span<String>),
    Arity {
        function: Span<String>,
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
                        function: id.map(ToOwned::to_owned),
                        expected: func.argument_names.len(),
                        actual: args.len(),
                    });
                }

                let values = args.iter()
                    .map(|arg| arg.evaluate(ctx, scoring_callback))
                    .collect::<Result<Vec<_>, _>>()?;
                scoring_callback(id.0, &values);
                let fn_ctx = Context(func.argument_names.iter().map(|&name| name.to_owned())
                                       .zip(values.into_iter().map(SymbolValue::Num))
                                       .collect());
                func.value_expr.evaluate(&fn_ctx, scoring_callback)?
            }
            &Expr::Ident(id) => ctx.get_variable(id)?.clone(),
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
                match op.0 {
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
            Expr::Num(n) => BigRational::from_integer(n.0.clone().into()),
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

#[cfg(test)]
mod tests {
    use crate::parse_expr;
    use super::*;

    fn ratio(numer: i128, denom: i128) -> BigRational {
        BigRational::new(numer.into(), denom.into())
    }

    fn empty_ctx(expr: &str) -> Result<BigRational, EvalError> {
        let ctx = ConcreteContext::new();
        with_ctx(&ctx, expr)
    }

    fn with_ctx<'a>(ctx: &ConcreteContext<'_>, expr: &'a str) -> Result<BigRational, EvalError> {
        parse_expr(expr).unwrap().evaluate(ctx, &mut |_, _| ())
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
        assert_eq!(empty_ctx("2 + 2"), Ok(ratio(4, 1)));
    }

    #[test]
    fn handles_negatives() {
        assert_eq!(empty_ctx("2 + -3"), Ok(ratio(-1, 1)));
    }

    #[test]
    fn div0_doesnt_explode() {
        assert_eq!(empty_ctx("2 / 0"), Err(EvalError::DivisionByZero));
    }

    #[test]
    fn if_needs_parentheses() {
        assert!(parse_expr("2 + if 1 = 1 then 1 else 0").is_err());
    }

    #[test]
    fn reduced_fractions_equal() {
        assert_eq!(empty_ctx("if 2/3 = 4/6 then 1 else 0"), Ok(ratio(1, 1)));
    }

    #[test]
    fn false_conditions() {
        assert_eq!(empty_ctx("if 3 < 2 then 1 else 0"), Ok(ratio(0, 1)));
    }

    #[test]
    fn unevaluated_branch_no_error() {
        assert_eq!(empty_ctx("if 0 = 1 then 3 / 0 else 7"), Ok(ratio(7, 1)));
    }

    #[test]
    fn unknown_variable() {
        assert_eq!(empty_ctx("3 * x"),
            Err(EvalError::UnknownVariable(Span("x".to_owned(), (4, 5)))));
    }

    #[test]
    fn handles_variable() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("x".to_owned(), SymbolValue::Num(ratio(7, 1)));
        assert_eq!(with_ctx(&ctx, "3 * x"), Ok(ratio(21, 1)));
    }

    #[test]
    fn handles_function() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("f".to_owned(), SymbolValue::Func(FuncDef {
            name: "f",
            argument_names: vec!["x"],
            value_expr: parse_expr("x*x").unwrap(),
        }));

        let expr = parse_expr("f(3) + 4").unwrap();
        let mut scoring_calls = vec![
            ("f", vec![ratio(3, 1)]),
        ];

        let value = expr.evaluate(&ctx, &mut test_scoring(&mut scoring_calls));
        assert_eq!(value, Ok(ratio(13, 1)));
        assert!(scoring_calls.is_empty());
    }

    #[test]
    fn detects_arity_error_early() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("f".to_owned(), SymbolValue::Func(FuncDef {
            name: "f",
            argument_names: vec!["x"],
            value_expr: parse_expr("x*x").unwrap(),
        }));

        let expr = parse_expr("f(2, f(7))").unwrap();
        let mut scoring_calls = vec![];
        let value = expr.evaluate(&ctx, &mut test_scoring(&mut scoring_calls));
        assert_eq!(value, Err(EvalError::Arity {
            function: Span("f".to_owned(), (0, 1)),
            expected: 1,
            actual: 2,
        }));
    }

    #[test]
    fn tracks_location_of_function_calls() {
        assert_eq!(empty_ctx("1 + foo(2)"),
            Err(EvalError::UnknownFunction(Span("foo".to_owned(), (4, 7)))));
    }

    #[test]
    fn function_lexical_scoping() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("f".to_owned(), SymbolValue::Func(FuncDef {
            name: "f",
            argument_names: vec!["x"],
            value_expr: parse_expr("x*x").unwrap(),
        }));
        ctx.0.insert("x".to_owned(), SymbolValue::Num(ratio(7, 1)));
        let expr = parse_expr("f(3) + x").unwrap();
        let mut scoring_calls = vec![
            ("f", vec![ratio(3, 1)]),
        ];
        let value = expr.evaluate(&ctx, &mut test_scoring(&mut scoring_calls));
        assert_eq!(value, Ok(ratio(16, 1)));
        assert!(scoring_calls.is_empty());
    }
}
