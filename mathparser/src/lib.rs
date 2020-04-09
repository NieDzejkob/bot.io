use num_bigint::BigUint;

#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(grammar);
pub use grammar::ExprParser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Clone, Debug)]
pub enum Expr<'a> {
    Func(&'a str, Vec<Expr<'a>>),
    Ident(&'a str),
    BinOp(Box<Expr<'a>>, BinOp, Box<Expr<'a>>),
    Neg(Box<Expr<'a>>),
    Num(BigUint),
}

#[cfg(test)]
mod tests {
}
