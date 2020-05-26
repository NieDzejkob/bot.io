use crate::prelude::*;
use crate::Config;
use mathparser::ParseError;

pub struct MathError {
    span: Option<(usize, usize)>,
    message: String,
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

impl MathError {
    pub fn send_to_user(&self, ctx: &Context, user: &User, input: &str) -> Result<()> {
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
                .footer(|foot| foot.text(format!(
                "Note: assuming your message is an expression you want me to calculate. \
                If you meant to issue a command, make sure to prefix it with {}",
                ctx.data.read().get::<Config>().unwrap().prefix)))
        })).context("Send parser error message")?;
        Ok(())
    }
}
