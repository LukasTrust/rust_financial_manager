CREATE TABLE contracts (
    id SERIAL PRIMARY KEY,
    bank_id INT NOT NULL,
    name text NOT NULL,
    parse_name text NOT NULL,
    current_amount FLOAT NOT NULL,
    months_between_payment INT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE CASCADE
);