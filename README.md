
# Rust Financial Manager

![Rust CI](https://github.com/LukasTrust/rust_financial_manager/workflows/Rust%20CI/badge.svg) 
[![codecov](https://codecov.io/github/LukasTrust/rust_financial_manager/graph/badge.svg?token=7VRB83BLUS)](https://codecov.io/github/LukasTrust/rust_financial_manager)
![Latest Release](https://img.shields.io/github/v/release/LukasTrust/rust_financial_manager)

---

## Overview

**Rust Financial Manager** is a lightweight web-based financial tool designed for seamless personal finance management. Built with Rust and powered by the Rocket framework, it ensures efficient performance and integrates with PostgreSQL through Diesel ORM. The user-friendly interface provides a comprehensive overview of your financial situation.

> **Important:** This app is designed for local use, as it currently lacks encryption. Running it on the open internet is not recommended.

### Key Features
- **Multi-User Support:** Multiple users can manage their finances at the same time.
- **Multiple Bank Accounts:** Manage multiple accounts for a consolidated financial view.
- **CSV Data Import:** Upload financial data via CSV files by mapping necessary fields (e.g., Counterparty, Amount, Balance).
- **Contract Detection & Closure:** Automatically detect recurring payments (contracts) and close inactive ones.

### Other Features
- **Simple Docker Deployment:** Easily set up and run using the included Docker setup.
- **Customizable Setup:** Adjust settings via the `.env` file for personalized configurations.
- **PostgreSQL Integration:** Full database support via PostgreSQL for data persistence.

---

## Screenshots

### Dashboard Overview
A omprehensive view displaying all bank accounts linked to a user.
![Dashboard](screenshots/dashboard.png)

### Bank Overview
A comprehensive view displaying details of one bank linked to a user.
![Bank Overview](screenshots/bank_view.png)

### Contracts Page
Displays recurring payments or receipts identified as contracts.
![Contracts](screenshots/contracts.png)

### Transactions Page
A grid view of all transactions related to a specific bank account.
![Transactions](screenshots/transactions.png)

### Add Transaction to Contract
An intuitive guide for adding a transaction to a specific contract.
![Add Transaction](screenshots/add_to_contract_1.png)
![Add Transaction 2](screenshots/add_to_contract_2.png)

### Login
The login page of the site
![Login](screenshots/login.png)

### Register
The register page of the site
![Register](screenshots/register.png)

### Settings
The settings page of the site
![Settings](screenshots/settings.png)

---

## Installation

### Prerequisites
- Docker & Docker Compose

### Setup Instructions

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/LukasTrust/rust_financial_manager.git
   cd rust_financial_manager
   ```

2. **Navigate to Docker Directory:**
   ```bash
   cd docker
   ```

3. **Configure the Environment:**
   Edit the `.env` file to set your Postgres credentials and database URL:
   ```bash
   POSTGRES_USER="myuser"
   POSTGRES_PASSWORD="mypassword"
   POSTGRES_DB="financial_manager"
   DB_URL="postgres://myuser:mypassword@postgres/financial_manager"
   ```

4. **Build and Run the Application:**
   ```bash
   docker-compose up --build
   ```
   > The build process may take a few minutes. 

5. **Access the Application:**
   Open your browser and go to [http://localhost:8000](http://localhost:8000).

---

## Updating the Application

To pull the latest changes and rebuild:

```bash
git pull origin main
docker-compose down
docker-compose up --build
```

---

## Additional Features (Detailed Explanations)
- **Automatic Contract Closure:** When a contract is detected but doesn't have any relevant transactions for a significant period, the application will automatically mark it as closed. This helps maintain an accurate financial overview by keeping only active contracts visible.
- **Automatic Contract Update:** If a contract is about to be closed but the application detects new transactions with a slightly modified amount (up to 10% difference), it will create a Contract History entry and update the contract with the new amount. This ensures contracts with small variations remain tracked properly.
- **Contract Merging:** Users can manually merge contracts when automatic merging isn't possible due to discrepancies. The contract with the most recent payment date becomes the primary contract, but this may be adjusted in future updates to allow user selection of the primary contract.
- **Contract Utilities:** Users can delete incorrectly merged contracts, and contracts can be recreated by re-scanning the data. This feature also includes the ability to review contract history and manage closed contracts.
- **Assign Transactions to Contracts:** If transactions that belong to a contract aren't automatically matched, users can manually assign them by selecting the transaction row and clicking Add contract. The app will guide the user through resolving discrepancies, such as different transaction amounts (as shown in [**Add_transaction_2**](#add-transaction-to-contract-window)).
- **Transaction Utilities:** Transactions that are incorrectly matched can be removed from contracts, and users can mark them as Contract not allowed to exclude them from future scans. Additionally, users can hide or unhide transactions as needed for better visibility.
- **Localization:** The application supports localization and is currently available in both English and German, ensuring users from different regions can comfortably interact with the tool in their preferred language.