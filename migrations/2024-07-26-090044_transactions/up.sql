CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    bank_id INT NOT NULL,
    date DATE NOT NULL,
    counterparty VARCHAR(200) NOT NULL,
    amount FLOAT NOT NULL,
    FOREIGN KEY (bank_id) REFERENCES banks(id),
    UNIQUE (date, counterparty, amount)
);
