CREATE TABLE contracts (
    id SERIAL PRIMARY KEY,
    bank_id INT NOT NULL,
    name VARCHAR(200) NOT NULL,
    current_amount FLOAT NOT NULL,
    months_between_payment INT NOT NULL,
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE
);