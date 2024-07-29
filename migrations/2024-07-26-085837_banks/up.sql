CREATE TABLE banks (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    name VARCHAR(50) NOT NULL,
    link VARCHAR(200),
    current_amount FLOAT,
    interest_rate FLOAT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
