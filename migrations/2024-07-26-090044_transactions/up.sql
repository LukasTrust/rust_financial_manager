CREATE TABLE transactions (
    Id SERIAL PRIMARY KEY,
    BankId INT NOT NULL,
    Type VARCHAR(50) NOT NULL,
    Date DATE NOT NULL,
    Other VARCHAR(200),
    Comment VARCHAR(200),
    Amount FLOAT NOT NULL,
    FOREIGN KEY (BankId) REFERENCES banks(Id)
);
