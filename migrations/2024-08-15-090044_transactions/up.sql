CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    bank_id INT NOT NULL,
    contract_id INT,
    date DATE NOT NULL,
    counterparty VARCHAR(200) NOT NULL,
    amount FLOAT NOT NULL,
    bank_balance_after FLOAT NOT NULL,
    is_hidden BOOLEAN NOT NULL DEFAULT FALSE,
    contract_not_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE,
    FOREIGN KEY (contract_id) REFERENCES contracts(id) ON DELETE SET NULL,
    UNIQUE (date, counterparty, amount, bank_balance_after)
);
