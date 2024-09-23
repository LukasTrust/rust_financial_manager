
# Rust Financial Manager

[![Rust CI](https://github.com/LukasTrust/rust_financial_manager/workflows/Rust%20CI/badge.svg)](https://github.com/LukasTrust/rust_financial_manager/actions?query=workflow%3A%22Rust+CI%22)
[![Coverage](https://img.shields.io/badge/coverage-80%25-brightgreen)](./coverage/cobertura.xml)
[![Latest Release](https://img.shields.io/github/v/release/LukasTrust/rust_financial_manager)](https://github.com/LukasTrust/rust_financial_manager/releases/latest)

## About

Rust Financial Manager is a lightweight and efficient web-based financial management tool developed in Rust using the Rocket framework and Diesel ORM for PostgreSQL integration. It aims to provide a simple and intuitive interface for managing personal finances, tracking expenses, and generating reports.

### Features
- **Expense Tracking**: Easily add and categorize expenses.
- **Income Management**: Track various income sources and monitor financial growth.
- **Reports & Analytics**: Generate visual reports to understand spending habits.
- **Secure & Efficient**: Built with Rust, ensuring safety and performance.

## Screenshots

### Dashboard Overview
A summary of your financial status with key metrics.
![Dashboard](screenshots/dashboard.png)

### Expense Tracking
Add, edit, and delete expenses effortlessly.
![Expense Tracking](screenshots/expense_tracking.png)

### Income Management
Manage various income sources and track growth.
![Income Management](screenshots/income_management.png)

### Reports & Analytics
Visualize your financial data through interactive charts and reports.
![Reports & Analytics](screenshots/reports_analytics.png)

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
