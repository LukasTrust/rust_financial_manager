CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    bank_id INT NOT NULL,
    type_of_t VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    other VARCHAR(200),
    comment VARCHAR(200),
    amount FLOAT NOT NULL,
    FOREIGN KEY (bank_id) REFERENCES banks(id)
);
