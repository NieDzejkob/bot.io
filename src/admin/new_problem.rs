use crate::prelude::*;
use crate::models::NewProblem;
use crate::interactive::get_msg;
use diesel::prelude::*;
use enum_map::{enum_map, EnumMap};
use joinery::iter::JoinableIterator;
use mathparser::{eval::FuncDef, parse_command};
use std::convert::TryInto;
use strum::IntoEnumIterator;

#[derive(Clone, Copy, Debug, PartialEq, Eq, AsRefStr, EnumIter, EnumString, enum_map::Enum)]
#[strum(serialize_all = "snake_case")]
enum ScoringFactor {
    Query,
    GuessCorrect,
    GuessIncorrect,
    SubmitIncorrect,
}

fn show_scoring(scoring: &EnumMap<ScoringFactor, i32>) -> String {
    ScoringFactor::iter()
        .map(|factor| format!("`{}` {}\n", factor.as_ref(), scoring[factor]))
        .collect()
}

fn update_scoring(scoring: &mut EnumMap<ScoringFactor, i32>, command: &str)
    -> Result<(), &'static str>
{
    let mut parts = command.split_whitespace();
    let value = parts.next_back()
        .ok_or("Expected some command in your message, but couldn't find it")?;
    let factor = parts.join_with('_').to_string();
    if factor.is_empty() {
        return Err("Couldn't find a space in your messsage");
    }

    match (factor.parse(), value.parse()) {
        (Ok(factor), Ok(value)) => {
            scoring[factor] = value;
            Ok(())
        }
        (Ok(_), Err(_)) => Err("The value should be a number"),
        (Err(_), Ok(_)) => Err("Unknown scoring factor name. Typo?"),
        (Err(_), Err(_)) => Err("Come again? No idea what that's supposed to mean"),
    }
}

#[command("newproblem")]
#[aliases("addproblem")]
fn new_problem(rctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let ctx = rctx.clone();
    let user = msg.author.clone();
    let maybe_name = args.remains().map(|s| s.to_owned());
    InteractiveCommand {
        generator: Gen::new_boxed(|co| async move {
            let mut embed = serenity::builder::CreateEmbed::default();
            embed.color(Color::BLUE);
            let name = match maybe_name {
                Some(name) => name,
                None => {
                    user.dm(&ctx, |m| m.embed(|e| {
                        e.clone_from(&embed);
                        e.title("`<name?>`")
                    })).context("Send embed")?;
                    get_msg(&co).await
                }
            };

            embed.title(&name);

            user.dm(&ctx, |m| m.embed(|e| {
                e.clone_from(&embed);
                e.description("`<description?>`")
            })).context("Send embed")?;
            let description = get_msg(&co).await;
            embed.description(&description);

            user.dm(&ctx, |m| m.embed(|e| {
                e.clone_from(&embed);
                e.field("Difficulty", "`<difficulty?>`", false)
            })).context("Send embed")?;
            let difficulty = get_msg(&co).await;
            embed.field("Difficulty", &difficulty, false);

            user.dm(&ctx, |m| m.embed(|e| {
                e.clone_from(&embed);
                e.field("Formula", "`<formula?>`", true)
            })).context("Send embed")?;
            let formula = loop {
                let formula = get_msg(&co).await;
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
                e.field("Domain", "`<domain?>`", true);
                e.footer(|foot| foot.text("TODO: The domain field doesn't actually do anything yet."))
            })).context("Send embed")?;
            let domain = get_msg(&co).await;
            embed.field("Domain", format!("`{}`", domain), true);

            use ScoringFactor::*;

            let mut scoring = enum_map! {
                Query => 1,
                GuessCorrect => 0,
                GuessIncorrect => 2,
                SubmitIncorrect => 2,
            };

            'scoring_loop: loop {
                let mut scoring_string = show_scoring(&scoring);
                if scoring[SubmitIncorrect] < scoring[GuessIncorrect] {
                    scoring_string.push_str("\n\u{26a0}\u{fe0f} Submitting an incorrect answer \
                        reveals a counterexample (argument, value) pair. Are you sure it should \
                        cost less points than an incorrect guess?\n");
                } else if scoring[GuessIncorrect] < scoring[Query] {
                    scoring_string.push_str("\n\u{26a0}\u{fe0f} Incorrect guesses reveal the \
                        true value. Are you sure they should cost less than queries?\n");
                }

                scoring_string.push_str("\nChange the values with `submit_incorrect 3`, \
                    for example. Say `done` when you're... well... done.");

                user.dm(&ctx, |m| m.embed(|e| {
                    e.clone_from(&embed);
                    e.field("Scoring", scoring_string, false)
                })).context("Send embed")?;

                loop {
                    let command = get_msg(&co).await;
                    if command == "done" {
                        break 'scoring_loop;
                    }

                    if let Err(msg) = update_scoring(&mut scoring, &command) {
                        user.dm(&ctx, |m| m.content(msg))
                            .context("Send scoring command error message")?;
                    } else {
                        break;
                    }
                }
            }

            embed.field("Scoring", show_scoring(&scoring), false);

            let problem = NewProblem {
                name,
                description,
                difficulty,
                formula,
                domain,
                score_query: scoring[Query],
                score_guess_correct: scoring[GuessCorrect],
                score_guess_incorrect: scoring[GuessIncorrect],
                score_submit_incorrect: scoring[SubmitIncorrect],
            };

            let conn = crate::db::get_connection(&ctx)?;

            use crate::schema::problems;

            diesel::insert_into(problems::table)
                .values(&problem)
                .execute(&conn)
                .context("Insert problem into database")
                ?;

            user.dm(&ctx, |m| m.embed(|e| {
                *e = embed;
                e.footer(|foot| foot.text("Your problem has been created!"));
                e.color(Color::DARK_GREEN)
            })).context("Send embed")?;

            Ok(())
        }),
        abort_message: Some("Do you want to abort creating this problem?".to_owned()),
    }.start(rctx, msg);
    Ok(())
}
