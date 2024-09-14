import { error, log } from './main.js';

export function update_performance(performance_value) {
    if (!performance_value || typeof performance_value !== 'object') {
        error('Invalid performance value received:', 'update_performance', performance_value);
        return;
    }

    const elements = {
        transactions_total_discrepancy: performance_value.transactions_total_discrepancy,
        transactions_count: performance_value.transactions_count,
        transactions_average_amount: performance_value.transactions_average_amount,
        transactions_max_amount: performance_value.transactions_max_amount,
        contracts_count: performance_value.contracts_count,
        contracts_amount_per_month: performance_value.contracts_amount_per_month,
        transactions_min_amount: performance_value.transactions_min_amount,
        transactions_net_gain_loss: performance_value.transactions_net_gain_loss,
        contracts_total_positive_amount: performance_value.contracts_total_positive_amount,
        contracts_total_negative_amount: performance_value.contracts_total_negative_amount,
        contracts_amount_per_time_span: performance_value.contracts_amount_per_time_span,
        contracts_amount_per_year: performance_value.contracts_amount_per_year,
    };

    log('Updating performance elements:', 'update_performance', elements);

    // Iterate over each element and update its content
    Object.keys(elements).forEach(id => {
        const element = document.getElementById(id);
        if (element && elements[id] !== undefined) {
            element.textContent = elements[id];
            log(`Updated element ${id} with value ${elements[id]}.`, 'update_performance');
        } else {
            log(`Element ${id} not found or value is undefined.`, 'update_performance');
        }
    });

    formatAndColorNumbers();
}

function formatAndColorNumbers() {
    const ids = [
        "transactions_count",
        "transactions_average_amount",
        "transactions_max_amount",
        "transactions_min_amount",
        "transactions_net_gain_loss",
        "transactions_total_discrepancy",
        "contracts_count",
        "contracts_amount_per_month",
        "contracts_total_positive_amount",
        "contracts_total_negative_amount",
        "contracts_amount_per_time_span",
        "contracts_amount_per_year",
    ];

    ids.forEach(id => {
        const element = document.getElementById(id);
        if (element) {
            let value = parseFloat(element.textContent);

            // Check if value is a valid number
            if (isNaN(value)) {
                log(`Value for element ${id} is not a valid number.`, 'formatAndColorNumbers');
                return;
            }

            value = value.toFixed(2); // Format to two decimal places
            log(`Formatting value for element ${id}: ${value}`, 'formatAndColorNumbers');

            // Determine the icon based on the value
            let iconName;
            if (value > 0) {
                iconName = "positive.png";
            } else if (value < 0) {
                iconName = "negative.png";
            } else {
                iconName = "no-change.png";
            }

            // Skip the icon addition for count elements
            if (id === "transactions_count" || id === "contracts_count") {
                return;
            }

            // Remove any existing icon if present
            if (element.previousElementSibling && element.previousElementSibling.tagName === "IMG") {
                element.previousElementSibling.remove();
                log(`Removed existing icon for element ${id}.`, 'formatAndColorNumbers');
            }

            // Create and insert the new icon
            const icon = document.createElement("img");
            icon.src = `/static/images/${iconName}`;
            icon.alt = "Icon";
            element.innerHTML = ''; // Clear existing content
            element.appendChild(icon); // Add the icon
            element.insertAdjacentText('beforeend', `${value} €`); // Add the formatted text

            // Apply CSS classes for positive or negative values
            element.classList.toggle("positive", value > 0);
            element.classList.toggle("negative", value < 0);

            log(`Updated element ${id} with formatted value: ${value} € and icon: ${iconName}.`, 'formatAndColorNumbers');
        } else {
            log(`Element ${id} not found.`, 'formatAndColorNumbers');
        }
    });
}
