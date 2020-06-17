//! Handles formulas sent to the bot.

use crate::prelude::*;
use crate::Config;
use mathparser::errors::MathError;
use mathparser::eval::{ConcreteContext, SymbolValue};

pub fn handle_message(ctx: &Context, msg: &Message) -> CommandResult {
    let confusable_footer = || {
        format!(
            "Note: assuming your message is an expression you want me to calculate. \
            If you meant to issue a command, make sure to prefix it with {}",
            ctx.data.read().get::<Config>().unwrap().prefix)
    };
    let command = mathparser::parse_command(&msg.content);
    use mathparser::Command;
    match command {
        Ok(Command::Expr(expr)) => {
            let mut eval_ctx = ConcreteContext::new();
            let problem = crate::problem::get_chosen_problem(ctx, msg.author.id)?;
            if let Some(problem) = &problem {
                let func = problem.func_def();
                eval_ctx.0.insert(func.name.to_owned(), SymbolValue::Func(func.clone()));
            }

            match expr.evaluate(&eval_ctx, &mut |_, _| ()) {
                Ok(v) => {
                    msg.author.dm(ctx, |m| m.embed(|e| {
                        e.title(MessageBuilder::new()
                                .push_safe(&msg.content)
                                .build())
                            .description(format!(" = {}", v))
                    }))?;
                }
                Err(why) => {
                    let error: MathError = why.into();
                    error.send_to_user(ctx, &msg.author, &msg.content, &confusable_footer());
                }
            }
        }
        Err(why) => {
            let error: MathError = why.into();
            error.send_to_user(ctx, &msg.author, &msg.content, &confusable_footer());
        }
        _ => (),
    }

    Ok(())
}
