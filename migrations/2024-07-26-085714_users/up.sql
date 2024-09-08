CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    first_name text NOT NULL,
    last_name text NOT NULL,
    email text NOT NULL UNIQUE,
    password text NOT NULL,
    language text NOT NULL DEFAULT 'English'
);
