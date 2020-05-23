use diesel_derive_newtype::DieselNewType;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, DieselNewType, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProblemId(i32);

#[derive(Queryable)]
pub struct Problem {
    pub id: ProblemId,
    pub name: String,
    pub description: String,
    pub difficulty: String,
    pub formula: String,
    pub domain: String,
    pub score_query: i32,
    pub score_guess_correct: i32,
    pub score_guess_incorrect: i32,
    pub score_submit_incorrent: i32,
}
