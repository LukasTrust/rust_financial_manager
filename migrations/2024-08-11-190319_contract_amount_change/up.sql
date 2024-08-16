CREATE TABLE contract_history (
    id SERIAL PRIMARY KEY,
    contract_Id INT REFERENCES contracts(Id) ON DELETE CASCADE NOT NULL,
    old_amount FLOAT NOT NULL,
    new_amount FLOAT NOT NULL,
    changed_At DATE NOT NULL
);