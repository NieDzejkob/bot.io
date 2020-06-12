use std::convert::{TryFrom, TryInto};
use diesel::prelude::*;
use crate::prelude::*;
use crate::models::{Problem, ProblemId};
use crate::reactions::digit_as_emoji;
use crate::interactive::get_reaction_on_msg;
use joinery::iter::JoinableIterator;
use mathparser::{ast, parse_pred, errors::MathError};
use rent_problem::ParsedFormula;
use serenity::builder::CreateEmbed;

pub struct ParsedProblem {
    pub id: ProblemId,
    pub name: String,
    pub description: String,
    pub difficulty: String,
    pub formula: ParsedFormula,
    pub domain: String,
    pub score_query: i32,
    pub score_guess_correct: i32,
    pub score_guess_incorrect: i32,
    pub score_submit_incorrect: i32,
}

rental! {
    pub mod rent_problem {
        use mathparser::eval::FuncDef;

        #[rental]
        pub struct ParsedFormula {
            formula: String,
            func_def: (&'formula str, FuncDef<'formula>),
        }
    }
}

impl TryFrom<Problem> for ParsedProblem {
    type Error = MathError;

    fn try_from(problem: Problem) -> Result<Self, Self::Error> {
        let formula = ParsedFormula::try_new(
            problem.formula,
            |formula| {
                let pred = parse_pred(formula)?;
                let lhs = match &pred {
                    ast::Pred::Cmp(lhs, _, _) => lhs.1,
                };
                let func = ast::Command::Pred(pred).try_into()?;
                Ok((&formula[lhs.0..lhs.1], func))
            }
        ).map_err(|x: rental::RentalError<MathError, _>| x.0)?;

        Ok(ParsedProblem {
            id: problem.id,
            name: problem.name,
            description: problem.description,
            difficulty: problem.difficulty,
            formula,
            domain: problem.domain,
            score_query: problem.score_query,
            score_guess_correct: problem.score_guess_correct,
            score_guess_incorrect: problem.score_guess_incorrect,
            score_submit_incorrect: problem.score_submit_incorrect,
        })
    }
}

impl ParsedProblem {
    /// Returns a string like `f(x, y)`, sliced out of the formula that defines the function.
    fn get_function_declaration(&self) -> &str {
        self.formula.ref_rent(|tail| tail.0)
    }

    pub fn show_in_embed(&self, n: u8) -> String {
        iformat!("{digit_as_emoji(n)}  **{self.name} [{self.difficulty}]**\n\
                  {self.description}\n\n\
                  `{self.get_function_declaration()}` where `{self.domain}`")
    }
}

#[test]
fn function_declaration() {
    let problem = Problem {
        id: ProblemId(1),
        name: "Test".to_owned(),
        description: "Test".to_owned(),
        difficulty: "Test".to_owned(),
        formula: "f(x, y) = x + y".to_owned(),
        domain: "rational(x, y)".to_owned(),
        score_query: 1,
        score_guess_correct: 0,
        score_guess_incorrect: 2,
        score_submit_incorrect: 2,
    };

    let parsed: ParsedProblem = problem.try_into().unwrap();
    assert_eq!(parsed.get_function_declaration(), "f(x, y)");
}

#[command]
pub fn problems(rctx: &Context, msg: &Message) -> CommandResult {
    let ctx = rctx.clone();
    let user = msg.author.clone();
    InteractiveCommand {
        generator: Gen::new_boxed(|co| async move {
            use crate::schema::problems::dsl::*;

            let results = problems.load::<Problem>(&crate::db::get_connection(&ctx)?)
                .context("Fetch problems from database")?;
            let results = results.into_iter().map(ParsedProblem::try_from)
                .collect::<Result<Vec<_>, _>>()
                .context("Parse problems in the database")?;

            const PAGE_SIZE: usize = 3;
            let mut page = 0;
            let page_count = (results.len() + PAGE_SIZE - 1) / PAGE_SIZE;

            let embed_for_page = |e: &mut CreateEmbed, page| {
                e.color(Color::BLURPLE)
                    .title(format!("{} problems available", results.len()))
                    .footer(|f| f.text(format!("Page {} of {}", page + 1, page_count)))
                    .description(
                        results.iter()
                            .skip(PAGE_SIZE * page)
                            .take(PAGE_SIZE)
                            .enumerate()
                            .map(|(i, problem)| problem.show_in_embed(i as u8 + 1))
                            .join_with("\n\n"));
            };

            let mut msg = user.dm(&ctx, |m| m.embed(|e| { embed_for_page(e, page); e }))?;

            if page_count > 1 {
                msg.react(&ctx, ReactionType::try_from(crate::reactions::ARROW_LEFT).unwrap())?;
                msg.react(&ctx, ReactionType::try_from(crate::reactions::ARROW_RIGHT).unwrap())?;
            }

            let mut choice_reacts = vec![];
            let mut update_buttons = |ctx, msg: &Message, page| -> Result<()> {
                let count = if page != page_count - 1 {
                    PAGE_SIZE
                } else {
                    results.len() % PAGE_SIZE
                };

                while choice_reacts.len() < count {
                    let num = digit_as_emoji(choice_reacts.len() as u8 + 1);
                    choice_reacts.push(msg.react(ctx, ReactionType::try_from(num).unwrap())?);
                }

                while choice_reacts.len() > count {
                    choice_reacts.pop().unwrap().delete(&ctx)?;
                }

                Ok(())
            };

            update_buttons(&ctx, &msg, page)?;

            let mut update_page = |ctx, msg: &mut Message, page| -> Result<()> {
                msg.edit(ctx, |m| m.embed(|e| { embed_for_page(e, page); e }))?;
                update_buttons(ctx, msg, page)
            };

            loop {
                match get_reaction_on_msg(&co, msg.id).await.emoji {
                    ReactionType::Unicode(x) if x == crate::reactions::ARROW_LEFT => {
                        if page == 0 {
                            page = page_count - 1;
                        } else {
                            page -= 1;
                        }
                        update_page(&ctx, &mut msg, page)?;
                    }
                    ReactionType::Unicode(x) if x == crate::reactions::ARROW_RIGHT => {
                        page += 1;
                        if page == page_count {
                            page = 0;
                        }
                        update_page(&ctx, &mut msg, page)?;
                    }
                    _ => continue,
                }
            }
        }),
        abort_message: None,
    }.start(rctx, msg);
    Ok(())
}
