CREATE TABLE banks (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    name text NOT NULL UNIQUE,
    link text,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
