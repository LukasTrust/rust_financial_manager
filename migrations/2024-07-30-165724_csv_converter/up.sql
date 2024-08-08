CREATE TABLE csv_converters (
    id SERIAL PRIMARY KEY,
    csv_bank_id INT NOT NULL,
    date_column INT,
    counterparty_column INT,
    amount_column INT,
    bank_balance_after_column INT,
    FOREIGN KEY (csv_bank_id) REFERENCES banks(id)
);
