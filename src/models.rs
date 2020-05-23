#[derive(Queryable)]
pub struct Problem {
    pub id: i32,
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
