use crate::prelude::*;

#[command("newproblem")]
#[aliases("addproblem")]
fn new_problem(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let (mctx, user) = ctx.minify(msg);
    let maybe_name = args.remains().map(|s| s.to_owned());
    InteractiveCommand {
        generator: Gen::new_boxed(sync_producer!({
            let name = match maybe_name {
                Some(name) => name,
                None => {
                    user.dm(&mctx, |m| m.embed(|e| e
                        .color(Color::BLUE)
                        .title("`<please enter problem name>`")))
                    .context("Send embed").log_error();
                    yield_!(())
                }
            };

            user.dm(&mctx, |m| m.embed(|e| e
                .color(Color::BLUE)
                .title(&name)
                .description("`<please enter problem description>`")))
            .context("Send embed").log_error();
            let description = yield_!(());
            log::info!("TODO: Create problem {:?} with description {:?}", name, description);
        })),
        abort_message: "Do you want to abort creating this problem?".to_owned(),
    }.start(ctx, msg);
    Ok(())
}
