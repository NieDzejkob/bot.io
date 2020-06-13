table! {
    chosen_problems (user_id) {
        user_id -> Int8,
        problem_id -> Int4,
    }
}

table! {
    problems (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        difficulty -> Text,
        formula -> Text,
        domain -> Text,
        score_query -> Int4,
        score_guess_correct -> Int4,
        score_guess_incorrect -> Int4,
        score_submit_incorrect -> Int4,
    }
}

joinable!(chosen_problems -> problems (problem_id));

allow_tables_to_appear_in_same_query!(
    chosen_problems,
    problems,
);
