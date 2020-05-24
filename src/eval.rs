//! Handles formulas sent to the bot.

use anyhow::{Context as _};
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;
use crate::errors::MathError;

pub fn handle_message(ctx: &mut Context, msg: &Message) -> CommandResult {
    let command = mathparser::parse_command(&msg.content);
    use mathparser::Command;
    match command {
        Ok(Command::Expr(e)) => {
            dbg!(e);
        }
        Err(why) => {
            let error: MathError = why.into();
            error.send_to_user(ctx, &msg.author, &msg.content)
                .context("Send parse error message")?;
        }
        _ => (),
    }

    Ok(())
}
