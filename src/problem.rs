use std::convert::{TryFrom, TryInto};
use crate::models::{Problem, ProblemId};
use mathparser::{ast, parse_pred, errors::MathError};
use rent_problem::ParsedFormula;

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
