CREATE TABLE banks (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    name VARCHAR(50) NOT NULL,
    link VARCHAR(200),
    start_date DATE,
    end_date DATE,
    current_amount FLOAT,
    interest_rate FLOAT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
