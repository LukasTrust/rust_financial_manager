CREATE TABLE banks (
    Id SERIAL PRIMARY KEY,
    UserId INT NOT NULL,
    Name VARCHAR(50) NOT NULL,
    Link VARCHAR(200),
    StartDate DATE,
    EndDate DATE,
    CurrentAmount FLOAT,
    InterestRate FLOAT,
    FOREIGN KEY (UserId) REFERENCES users(Id)
);
