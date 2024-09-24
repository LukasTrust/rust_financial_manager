
# Rust Financial Manager

[![Rust CI](https://github.com/LukasTrust/rust_financial_manager/workflows/Rust%20CI/badge.svg)](https://github.com/LukasTrust/rust_financial_manager/actions?query=workflow%3A%22Rust+CI%22)
[![Coverage](https://img.shields.io/badge/coverage-80%25-brightgreen)](./coverage/cobertura.xml)
[![Latest Release](https://img.shields.io/github/v/release/LukasTrust/rust_financial_manager)](https://github.com/LukasTrust/rust_financial_manager/releases/latest)

## About

*Rust Financial Manager* is a lightweight and efficient web-based financial management tool, designed to help users easily track and manage their finances. Built with Rust, it leverages the Rocket framework for seamless web performance and utilizes Diesel ORM for robust PostgreSQL integration. With its user-friendly interface and powerful features, Rust Financial Manager empowers individuals to gain a clearer overview of their financial situation.
With the provided *docker-compose.yml*, Rust Financial Manager can be easily deployed on a home server. 
- **NOTE** This application should not be run on the open internet, as it currently lacks encryption. This focus on local use ensures your financial data remains secure and private.

### Features
- **Multi-User Support** The application allows multiple users to use it simultaneously
- **Multiple Bank Accounts per User** Users can track and manage multiple bank accounts simultaneously, providing a comprehensive overview of their financial situation.
- **CSV Data Import** Data can be uploaded only via CSV files. Users must specify which columns (starting from 0) contain the required data points: *Counterparty*, *Amount*, *Balance after transaction*, and *Date*. Without this setup, the upload will not be possible
- **Automatic Contract Detection** The application analyzes the data to identify recurring patterns that indicate a contract, such as regular payments or receipts at consistent intervals
- **Automatic Contract Closure** If a contract is detected but no longer has relevant transactions in the new data over a significant period, the application will automatically mark the contract as closed
- **Automatic Contract Update** Before a contract is closed, the application checks for transactions with a modified amount (up to 10% change) that still fit the rest of the contract criteria (interval and counterparty). If a match is found, a *Contract History* entry is created with the old amount and the change date, while the contract is updated with the new amount and remains active
- **Contract Merging** In cases where the application cannot automatically merge contracts due to discrepancies, users can manually select multiple contracts in the contract view and click Merge selected contracts to combine them into one. Note: The contract with the most recent payment date will become the new primary contract (this may be updated to allow user selection)
- **Contract Utilities** Contracts can be deleted if merged incorrectly. After deletion, users can re-scan for contracts, and the original contracts will be recreated from the data. Users can also view different contract histories and review closed contracts
- **Transaction View** If transactions belong to a contract but were not automatically matched, users can select the transaction row, click Add contract, and choose the appropriate contract. If the transaction amount differs from the contract, the application will prompt the user on how to handle the discrepancy, as shown in image *Add_transaction_2*
- **Assign Transactions to Contracts** If you have transactions that belonge to a contract but where not match to it you can click on thier row and click *Add contract* and then selecte the contract you want to add it to. If the amount of the transaction matches that of the contract that is is but if not you will be asked how the appliaction should handle the data, which is displayed in image [**Add_transaction_2**](#add-transaction-to-contract-window)
- **Transaction Utilities** Transactions can be removed from contracts if they were incorrectly matched or added by mistake. If they are re-added during subsequent scans, users can set them as Contract not allowed, preventing them from being considered in future scans. Transactions can also be hidden or unhidden as needed
- **Localization** The application includes built-in localization support and currently supports both English and German

## Screenshots

### Login
The login page of the site
![Login](screenshots/login.png)

### Register
The register page of the site
![Register](screenshots/register.png)

### Settings
The settings page of the site
![Settings](screenshots/settings.png)

### Dashboard Overview
A summary of your financial status with key metrics
![Dashboard](screenshots/dashboard.png)

### Add bank page
A view in which a new bank can be added
![Add_bank](screenshots/add_bank.png)

### Specific Bank page
A view that displayes the data of a bank and can be used to upload new data in form of CSV files
![Bank](screenshots/bank_view.png)

### Contracts page
A view that displayes the contracts of a specific bank that were found in the data. The contracts are displayed in a crad view
![Contracts](screenshots/contracts.png)

### Transactions page
A view that displayes the transactions of a specific bank that were found in the data. The transactions are displayed in a grid view
![Transactions](screenshots/transactions.png)

### Add transaction to contract window
The small windows that will guide you throw the add transaction to contract view
![Add_transaction](screenshots/add_to_contract_1.png)
![Add_transaction_2](screenshots/add_to_contract_2.png)

## Installation

### Prerequisites
- Docker and Docker Compose installed on your system.

### Setup & Run

1. **Clone the Repository**
    ```bash
    git clone https://github.com/LukasTrust/rust_financial_manager.git
    cd rust_financial_manager
    ```

2. **Navigate to Docker Directory**
    ```bash
    cd docker
    ```

3. **Edit the .env File**
    Make sure to replace the *POSTGRES_PASSWORD* with a strong password. And most importantly, edit the *DB_URL* to match the changes you made to the other variables.
    ```bash
    POSTGRES_USER="myuser"
    POSTGRES_PASSWORD="mypassword"
    POSTGRES_DB="financial_manager"
    DB_URL="postgres://myuser:mypassword@postgres/financial_manager"
    ```

4. **Build the Image**
    ```bash
    docker-compose up --build
    ```
    **Note:** The build process may take several minutes as the project is built locally within the Docker container. Please be patient.

5. **Access the Application**
    Open your web browser and go to [http://localhost:8000](http://localhost:8000) to access the Rust Financial Manager.

### Updating the Application
To update the application, follow these steps:
   ```bash
   git pull origin main
   docker-compose down
   docker-compose up --build
   ```
   This will pull the latest changes from the repository and rebuild the Docker image.
