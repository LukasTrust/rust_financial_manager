CREATE TABLE csv_converters (
    id SERIAL PRIMARY KEY,
    csv_bank_id INT NOT NULL,
    date_conv VARCHAR(200),
    counterparty_conv VARCHAR(200),
    amount_conv VARCHAR(200),
    FOREIGN KEY (csv_bank_id) REFERENCES banks(id)
);
