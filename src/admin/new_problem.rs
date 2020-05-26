use crate::prelude::*;

#[command("newproblem")]
#[aliases("addproblem")]
fn new_problem(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    dbg!(args);
    Ok(())
}
