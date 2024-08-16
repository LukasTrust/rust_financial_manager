import { log } from './logger.js';

export function formatAndColorNumbers() {
    log('Formatting and coloring numbers:', 'formatAndColorNumbers');
    const elements = [
        document.getElementById("net_gain_loss"),
        document.getElementById("performance_percentage"),
        document.getElementById("average_transaction_amount"),
        document.getElementById("total_discrepancy"),
        document.getElementById("total_amount_per_year"),
        document.getElementById("one_month_contract_amount"),
        document.getElementById("three_month_contract_amount"),
        document.getElementById("six_month_contract_amount"),
    ];

    elements.forEach(element => {
        if (element) {
            let value = parseFloat(element.textContent).toFixed(2);

            if (element.id === "performance_percentage") {
                element.textContent = `${value} %`;
            } else {
                element.textContent = `${value} €`;
            }

            element.classList.toggle("positive", value >= 0);
            element.classList.toggle("negative", value < 0);
        }
    });

    log('Numbers formatted and colored based on value.', 'formatAndColorNumbers');
}

export function update_performance(result) {
    const total_transactions = document.getElementById("total_transactions");
    const net_gain_loss = document.getElementById("net_gain_loss");
    const performance_percentage = document.getElementById("performance_percentage");
    const average_transaction_amount = document.getElementById("average_transaction_amount");
    const total_discrepancy = document.getElementById("total_discrepancy");

    const total_contracts = document.getElementById("total_contracts");
    const total_amount_per_year = document.getElementById("total_amount_per_year");
    const one_month_contract_amount = document.getElementById("one_month_contract_amount");
    const three_month_contract_amount = document.getElementById("three_month_contract_amount");
    const six_month_contract_amount = document.getElementById("six_month_contract_amount");

    total_transactions.textContent = result.performance_value.total_transactions;
    net_gain_loss.textContent = result.performance_value.net_gain_loss;
    performance_percentage.textContent = result.performance_value.performance_percentage;
    average_transaction_amount.textContent = result.performance_value.average_transaction_amount;
    total_discrepancy.textContent = result.performance_value.total_discrepancy;

    total_contracts.textContent = result.performance_value.total_contracts;
    total_amount_per_year.textContent = result.performance_value.total_amount_per_year;
    one_month_contract_amount.textContent = result.performance_value.one_month_contract_amount;
    three_month_contract_amount.textContent = result.performance_value.three_month_contract_amount;
    six_month_contract_amount.textContent = result.performance_value.six_month_contract_amount;

    formatAndColorNumbers();

    log('Performance metrics updated and formatted successfully:', 'update_performance');
}
