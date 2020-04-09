fn main() {
    dbg!(mathparser::ExprParser::new()
        .parse("(2 + 2 +"));
}
