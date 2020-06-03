use num_bigint::BigUint;

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
    Func(Span<&'a str>, Vec<Span<Expr<'a>>>),
    Ident(Span<&'a str>),
    If(BSpan<Pred<'a>>, BSpan<Expr<'a>>, BSpan<Expr<'a>>),
    BinOp(BSpan<Expr<'a>>, Span<BinOp>, BSpan<Expr<'a>>),
    Neg(BSpan<Expr<'a>>),
    Num(Span<BigUint>),
}

#[derive(Clone, Debug)]
pub enum Pred<'a> {
    Cmp(Span<Expr<'a>>, Span<Cmp>, Span<Expr<'a>>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span<T>(pub T, pub (usize, usize));

pub type BSpan<T> = Box<Span<T>>;

impl<T> Span<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Span<U> {
        Span(f(self.0), self.1)
    }

    pub fn as_ref(&self) -> Span<&T> {
        Span(&self.0, self.1)
    }
}

use std::ops::Deref;
impl<T> Deref for Span<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}
