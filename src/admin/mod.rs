use crate::prelude::*;
use serenity::framework::standard::{
    CheckResult, CommandOptions,
    macros::{check, group},
};

mod new_problem;
use new_problem::*;

#[check]
#[name = "Admin"]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if ctx.data.read().get::<crate::Config>().unwrap().admin_users.contains(&msg.author.id) {
        true.into()
    } else {
        CheckResult::new_user("This command requires administrator privileges.")
    }
}

#[group]
#[checks(Admin)]
#[commands(new_problem)]
struct Admin;
