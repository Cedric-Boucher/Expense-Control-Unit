-- Add migration script here
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    description TEXT NOT NULL,
    amount NUMERIC NOT NULL
);
