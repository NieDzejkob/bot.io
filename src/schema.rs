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
