#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(grammar);

mod ast;
pub mod eval;

pub use ast::*;

use lalrpop_util::lexer::Token;
pub type ParseError<'a> = lalrpop_util::ParseError<usize, Token<'a>, &'static str>;

pub fn parse_expr(input: &str) -> Result<Expr<'_>, ParseError<'_>> {
    grammar::ExprParser::new().parse(input)
}

pub fn parse_command(input: &str) -> Result<Command<'_>, ParseError<'_>> {
    grammar::CommandParser::new().parse(input)
}

#[cfg(test)]
mod tests {
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
        assert_eq!(empty_ctx("3 * x"), Err(EvalError::UnknownVariable("x".to_owned())));
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
            argument_names: vec!["x"],
            value_expr: parse_expr("x*x").unwrap(),
        }));

        let expr = parse_expr("f(2, f(7))").unwrap();
        let mut scoring_calls = vec![];
        let value = expr.evaluate(&ctx, &mut test_scoring(&mut scoring_calls));
        assert_eq!(value, Err(EvalError::Arity {
            function: "f".to_owned(),
            expected: 1,
            actual: 2,
        }));
    }

    #[test]
    fn function_lexical_scoping() {
        let mut ctx = ConcreteContext::new();
        ctx.0.insert("f".to_owned(), SymbolValue::Func(FuncDef {
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
