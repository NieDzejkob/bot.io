#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(grammar);

pub mod ast;
pub mod eval;

pub use ast::{Expr, Pred, Command};

use lalrpop_util::lexer::Token;
pub type ParseError<'a> = lalrpop_util::ParseError<usize, Token<'a>, &'static str>;

pub fn parse_expr(input: &str) -> Result<Expr<'_>, ParseError<'_>> {
    grammar::ExprParser::new().parse(input)
}

pub fn parse_command(input: &str) -> Result<Command<'_>, ParseError<'_>> {
    grammar::CommandParser::new().parse(input)
}
