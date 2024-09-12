export function formatAndColorNumbers() {
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
            let value = parseFloat(element.textContent);

            // Check if value is a valid number
            if (isNaN(value)) return;

            value = value.toFixed(2); // Format to two decimal places

            // Determine the icon based on the value
            let iconName;
            if (value > 0) {
                iconName = "positive.png";
            } else if (value < 0) {
                iconName = "negative.png";
            } else {
                iconName = "no-change.png";
            }

            // Remove any existing icon if present
            if (element.previousElementSibling && element.previousElementSibling.tagName === "IMG") {
                element.previousElementSibling.remove();
            }

            // Create the icon image element
            const icon = document.createElement("img");
            icon.src = `/static/images/${iconName}`;
            icon.alt = "Icon";

            // Format the text content
            let formattedText;
            if (element.id === "performance_percentage") {
                formattedText = `${value} %`;
            } else {
                formattedText = `${value} €`;
            }

            // Clear current content and add the icon and formatted text in order
            element.innerHTML = ''; // Clear existing content
            element.appendChild(icon); // Add the icon first
            element.insertAdjacentText('beforeend', formattedText); // Add the formatted text after the icon

            // Apply CSS classes for positive or negative values
            element.classList.toggle("positive", value > 0);
            element.classList.toggle("negative", value < 0);
        }
    });
}

export function update_performance(performance_value) {
    if (!performance_value || typeof performance_value !== 'object') return;

    const elements = {
        total_transactions: performance_value.total_transactions,
        net_gain_loss: performance_value.net_gain_loss,
        performance_percentage: performance_value.performance_percentage,
        average_transaction_amount: performance_value.average_transaction_amount,
        total_discrepancy: performance_value.total_discrepancy,
        total_contracts: performance_value.total_contracts,
        total_amount_per_year: performance_value.total_amount_per_year,
        one_month_contract_amount: performance_value.one_month_contract_amount,
        three_month_contract_amount: performance_value.three_month_contract_amount,
        six_month_contract_amount: performance_value.six_month_contract_amount
    };

    // Iterate over each element and update its content
    Object.keys(elements).forEach(id => {
        const element = document.getElementById(id);
        if (element && elements[id] !== undefined) {
            element.textContent = elements[id];
        }
    });

    formatAndColorNumbers();
}
