use mathparser::{ConcreteContext, ExprParser};

fn main() {
    let ctx = ConcreteContext::new();
    let expr = ExprParser::new()
        .parse("2 + 2")
        .unwrap();
    dbg!(expr.evaluate(&ctx, &mut |_, _| ()));
}
