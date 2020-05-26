use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};

#[command("newproblem")]
#[aliases("addproblem")]
fn new_problem(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    dbg!(args);
    Ok(())
}
