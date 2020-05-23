CREATE TABLE problems (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    -- TODO: formula and domain are stored in the user-facing syntax, but this
    -- is a compatibility hazard. SMTLIB s-expressions seem to be a reasonable
    -- solution, but this adds a soft dependency between adding problems from
    -- the user interface and automatic grading.
    formula TEXT NOT NULL,
    domain TEXT NOT NULL,
    score_query INT NOT NULL,
    score_guess_correct INT NOT NULL,
    score_guess_incorrect INT NOT NULL,
    score_submit_incorrent INT NOT NULL
)
