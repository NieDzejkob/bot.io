use crate::{ast, eval::FuncDef, ParseError};
use std::convert::TryFrom;

pub struct MathError {
    pub span: Option<(usize, usize)>,
    pub message: String,
}

impl From<ParseError<'_>> for MathError {
    fn from(error: ParseError) -> Self {
        match error {
            ParseError::InvalidToken { location } => {
                MathError {
                    span: Some((location, location + 1)),
                    message: "You lost me here...".into(),
                }
            }
            ParseError::UnrecognizedToken { token, .. } |
            ParseError::ExtraToken { token } => {
                let (left, _, right) = token;
                MathError {
                    span: Some((left, right)),
                    message: "You lost me here...".into(),
                }
            }
            ParseError::UnrecognizedEOF { location, .. } => {
                MathError {
                    span: Some((location, location + 1)),
                    message: "Expression ended unexpectedly".into(),
                }
            }
            ParseError::User { error } => {
                eprintln!("ParseError::User: {:?}", error);
                MathError {
                    span: None,
                    message: "An unknown error occured while parsing your expression".into(),
                }
            }
        }
    }
}

impl<'a> TryFrom<ast::Command<'a>> for FuncDef<'a> {
    type Error = MathError;

    fn try_from(cmd: ast::Command<'a>) -> Result<Self, MathError> {
        let format_error = |desc, span| {
            Err(MathError {
                span,
                message: format!("Expected an equation, got {} instead", desc),
            })
        };

        match cmd {
            ast::Command::Pred(ast::Pred::Cmp(lhs, op, rhs)) if op.0 == ast::Cmp::Eq => {
                match lhs.0 {
                    ast::Expr::Func(name, args) => {
                        let args = args.iter()
                            .map(|arg| match &arg.0 {
                                ast::Expr::Ident(id) => Ok(id.0),
                                e => Err(MathError {
                                    span: Some(arg.1),
                                    message: format!("Expected an argument name, \
                                        got {} instead", e.describe()),
                                }),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(FuncDef {
                            name: name.0,
                            argument_names: args,
                            value_expr: rhs.0,
                        })
                    }
                    e => Err(MathError {
                        span: Some(lhs.1),
                        message: format!("Expected a function application \
                            on the left side of the equality, got {} instead", e.describe()),
                    })
                }
            }
            ast::Command::Pred(ast::Pred::Cmp(_, op, _)) => {
                format_error("a comparison", Some(op.1))
            }
            ast::Command::Expr(e) => {
                format_error(e.describe(), None)
            }
        }
    }
}

impl ast::Expr<'_> {
    fn describe(&self) -> &str {
        use crate::ast::Expr::*;
        match self {
            Func(_, _) => "a function application",
            Ident(_) => "a variable name",
            If(_, _, _) => "a conditional expression",
            BinOp(_, op, _) => match op.0 {
                ast::BinOp::Add => "a sum",
                ast::BinOp::Sub => "a subtraction",
                ast::BinOp::Mul => "a product",
                ast::BinOp::Div => "a quotient",
                ast::BinOp::Mod => "a remainder",
            },
            Neg(_) => "a negation",
            Num(_) => "a number",
        }
    }
}
