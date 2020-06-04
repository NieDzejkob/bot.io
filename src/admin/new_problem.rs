use crate::prelude::*;
use mathparser::{eval::FuncDef, parse_command};
use std::convert::TryInto;

#[command("newproblem")]
#[aliases("addproblem")]
fn new_problem(rctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let ctx = rctx.clone();
    let user = msg.author.clone();
    let maybe_name = args.remains().map(|s| s.to_owned());
    InteractiveCommand {
        generator: Gen::new_boxed(sync_producer!({
            let mut embed = serenity::builder::CreateEmbed::default();
            embed.color(Color::BLUE);
            let name = match maybe_name {
                Some(name) => name,
                None => {
                    user.dm(&ctx, |m| m.embed(|e| {
                        e.clone_from(&embed);
                        e.title("`<please enter problem name>`")
                    })).context("Send embed").log_error();
                    yield_!(())
                }
            };

            embed.title(&name);

            user.dm(&ctx, |m| m.embed(|e| {
                e.clone_from(&embed);
                e.description("`<please enter problem description>`")
            })).context("Send embed").log_error();
            let description = yield_!(());
            embed.description(&description);

            user.dm(&ctx, |m| m.embed(|e| {
                e.clone_from(&embed);
                e.field("Formula", "`<please enter the formula for this problem>`", true)
            })).context("Send embed").log_error();
            let formula = loop {
                let formula = yield_!(());
                let function = parse_command(&formula)
                    .map_err(From::from)
                    .and_then(|cmd| cmd.try_into());
                match function {
                    Ok(FuncDef { .. }) => break formula,
                    Err(why) => why.send_to_user(&ctx, &user, &formula,
                                                 "Please try again."),
                }
            };
            embed.field("Formula", format!("`{}`", formula), true);

            user.dm(&ctx, |m| m.embed(|e| {
                e.clone_from(&embed);
                e.field("Domain", "`<please enter the domain for this problem>`", true);
                e.footer(|foot| foot.text("TODO: The domain field doesn't actually do anything yet."))
            })).context("Send embed").log_error();
            let domain = yield_!(());
            embed.field("Domain", format!("`{}`", domain), true);

            log::info!("TODO: Create problem {:?} with description {:?}", name, description);
        })),
        abort_message: "Do you want to abort creating this problem?".to_owned(),
    }.start(rctx, msg);
    Ok(())
}
