export function update_performance(performance_value) {
    if (!performance_value || typeof performance_value !== 'object') return;

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

    // Iterate over each element and update its content
    Object.keys(elements).forEach(id => {
        const element = document.getElementById(id);
        if (element && elements[id] !== undefined) {
            element.textContent = elements[id];
        }
    });

    formatAndColorNumbers();
    updateHints(performance_value);
}

function formatAndColorNumbers() {
    const elements = [
        document.getElementById("transactions_count"),
        document.getElementById("transactions_average_amount"),
        document.getElementById("transactions_max_amount"),
        document.getElementById("transactions_min_amount"),
        document.getElementById("transactions_net_gain_loss"),
        document.getElementById("transactions_total_discrepancy"),
        document.getElementById("contracts_count"),
        document.getElementById("contracts_amount_per_month"),
        document.getElementById("contracts_total_positive_amount"),
        document.getElementById("contracts_total_negative_amount"),
        document.getElementById("contracts_amount_per_time_span"),
        document.getElementById("contracts_amount_per_year"),
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

            if (element.id === "transactions_count" ||
                element.id === "contracts_count") {
                return;
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
            let formattedText = `${value} €`;

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

function updateHints(performance_value) {
    const boxElements = [
        document.getElementById("transactions_count_box"),
        document.getElementById("transactions_average_amount_box"),
        document.getElementById("transactions_max_amount_box"),
        document.getElementById("transactions_min_amount_box"),
        document.getElementById("transactions_net_gain_loss_box"),
        document.getElementById("transactions_total_discrepancy_box"),
        document.getElementById("contracts_count_box"),
        document.getElementById("contracts_amount_per_month_box"),
        document.getElementById("contracts_total_positive_amount_box"),
        document.getElementById("contracts_total_negative_amount_box"),
        document.getElementById("contracts_amount_per_time_span_box"),
        document.getElementById("contracts_amount_per_year_box"),
    ];

    // Iterate through each box element and assign the corresponding hint from the data
    boxElements.forEach(box => {
        if (box) {
            // Get the ID of the box and find the corresponding hint in the `hints` object
            const boxId = box.id;
            const hintKey = boxId.replace("_box", "_hint");  // Remove the "_box" to match the key in the `hints` object

            // If a hint exists for this box, add it as a data-hint attribute
            if (performance_value[hintKey]) {
                box.setAttribute("data-hint", performance_value[hintKey]);
            }
        }
    });
}