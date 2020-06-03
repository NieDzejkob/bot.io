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
    Func(Span<&'a str>, Vec<Expr<'a>>),
    Ident(Span<&'a str>),
    If(Box<Pred<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    BinOp(Box<Expr<'a>>, BinOp, Box<Expr<'a>>),
    Neg(Box<Expr<'a>>),
    Num(BigUint),
}

#[derive(Clone, Debug)]
pub enum Pred<'a> {
    Cmp(Expr<'a>, Cmp, Expr<'a>),
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
