use crate::prelude::*;
use mathparser::errors::MathError;

pub trait MathErrorExt {
    fn send_to_user(&self, ctx: &Context, user: &User, input: &str, footer: &str);
}

impl MathErrorExt for MathError {
    fn send_to_user(&self, ctx: &Context, user: &User, input: &str, footer: &str) {
        user.dm(ctx, |m| m.embed(|e| {
            if let Some((left, right)) = self.span {
                let codeblock = format!("{}\n{:left$}{:^<size$}", input, "", "",
                                            left = left,
                                            size = right - left);
                e.description(MessageBuilder::new().push_codeblock_safe(codeblock, None));
            }

            e
                .color(Color::RED)
                .title(&self.message)
                .footer(|foot| foot.text(footer))
        })).context("Send parser error message").log_error();
    }
}
