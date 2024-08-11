CREATE TABLE contract_amount_change (
    id SERIAL PRIMARY KEY,
    contract_Id INT REFERENCES contracts(Id) ON DELETE CASCADE,
    old_amount FLOAT NOT NULL,
    new_amount FLOAT NOT NULL,
    changed_At TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);