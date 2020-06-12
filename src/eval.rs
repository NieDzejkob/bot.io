//! Handles formulas sent to the bot.

use crate::prelude::*;
use crate::Config;
use mathparser::errors::MathError;

pub fn handle_message(ctx: &Context, msg: &Message) -> CommandResult {
    let command = mathparser::parse_command(&msg.content);
    use mathparser::Command;
    match command {
        Ok(Command::Expr(e)) => {
            dbg!(e);
        }
        Err(why) => {
            let error: MathError = why.into();
            let footer = format!(
                "Note: assuming your message is an expression you want me to calculate. \
                If you meant to issue a command, make sure to prefix it with {}",
                ctx.data.read().get::<Config>().unwrap().prefix);
            error.send_to_user(ctx, &msg.author, &msg.content, &footer);
        }
        _ => (),
    }

    Ok(())
}
