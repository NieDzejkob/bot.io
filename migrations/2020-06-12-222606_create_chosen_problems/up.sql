-- Your SQL goes here
CREATE TABLE chosen_problems (
    user_id BIGINT PRIMARY KEY,
    problem_id INTEGER NOT NULL,
    FOREIGN KEY (problem_id) REFERENCES problems (id)
)
