// Utility function to log messages with timestamp and context
function log(message, context = '', ...data) {
    console.log(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}

// Utility function to log errors with timestamp and context
function error(message, context = '', ...data) {
    console.error(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}

// Function to load content dynamically
function loadContent(url) {
    log('Loading content from URL:', 'loadContent', url);
    document.getElementById('main-content').innerHTML = '<p>Loading...</p>'; // Show loading state

    fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error('Network response was not ok');
            }

            return response.text();
        })
        .then(html => {

            if (html.includes('Please login again')) {
                log('Error validating the login. Redirecting to error page:', 'loadContent');
                window.location.href = '/error?error_title=Error%20validating%20the%20login!&error_message=Please%20login%20again.';
                return;
            }

            document.getElementById('main-content').innerHTML = html;

            const graphDataElement = document.getElementById('graph-data');
            if (graphDataElement) {
                try {
                    // Parse JSON data
                    const jsonText = graphDataElement.textContent.trim();
                    window.plotData = JSON.parse(jsonText);
                    log('Graph data successfully parsed:', 'loadContent', window.plotData);
                } catch (e) {
                    error('Error parsing graph data:', 'loadContent', e);
                }
            }

            // Reinitialize chart and date picker for dashboard and bank views
            if (url === '/dashboard' || /^\/bank\/\d+$/.test(url)) {
                log('Reinitializing chart and date picker for URL:', 'loadContent', url);
                initializeFormHandling();
                formatAndColorNumbers();
                initializeChartAndDatePicker();
            }

            // Reinitialize form handling if on add bank page
            if (url === '/add-bank') {
                log('Reinitializing form handling for add bank page:', 'loadContent');
                initializeFormHandling();
            }
        })
        .catch(err => {
            error('Error loading content:', 'loadContent', err);
            document.getElementById('main-content').innerHTML = '<p>Error loading content. Please try again.</p>';
        });
}

// Function to initialize the Plotly chart and Flatpickr date range picker
function initializeChartAndDatePicker() {
    log('Initializing Plotly chart and Flatpickr date range picker with data:', 'initializeChartAndDatePicker', window.plotData);

    update_graph();

    setTimeout(() => {
        flatpickr("#dateRange", {
            mode: "range",
            dateFormat: "Y-m-d",
            onChange: function (selectedDates) {
                if (selectedDates.length === 2) {
                    const [startDate, endDate] = selectedDates.map(date => date.toISOString().split('T')[0]);

                    fetch(`/update_date_range/${startDate}/${endDate}`, {
                        method: 'GET',
                        headers: { 'Content-Type': 'application/json' }
                    })
                        .then(response => response.json())
                        .then(data => {
                            if (data.performance_value) {
                                update_performance(data);
                                log('Update date range form submitted successfully. Updating performance metrics:', 'setTimeout', data.performance_value);
                            }

                            // Update the graph if `graph_data` is available
                            if (data.graph_data) {
                                window.plotData = JSON.parse(data.graph_data);
                                log('Update date range form submitted successfully. Reinitializing chart with new data:', 'setTimeout', window.plotData);
                                update_graph();
                            }
                        })

                        .catch(err => error('Error updating date range:', 'initializeChartAndDatePicker', err));
                }
            }
        });
    }, 0);
}

// Function to handle form submissions
async function handleFormSubmission(form) {
    form.addEventListener('submit', async function (event) {
        event.preventDefault();

        const formData = new FormData(form);
        const errorDiv = document.getElementById('error');
        const successDiv = document.getElementById('success');

        try {
            const response = await fetch(form.action, {
                method: form.method,
                body: formData
            });

            if (!response.ok) throw new Error(`HTTP error! Status: ${response.status}`);

            let result;
            try {
                result = await response.json();
            } catch (jsonError) {
                throw new Error('Error parsing JSON response');
            }

            // Handle success and error messages
            if (result.success) {
                successDiv.textContent = result.success;
                successDiv.style.display = 'block';
                errorDiv.style.display = 'none';

                // Update the graph if `graph_data` is available
                if (result.graph_data) {
                    window.plotData = JSON.parse(result.graph_data);
                    log('Form submitted successfully. Reinitializing chart with new data:', 'handleFormSubmission', window.plotData);
                    initializeChartAndDatePicker();
                }

                if (result.performance_value) {
                    update_performance(result);
                }

                if (result.banks) {
                    log('Updating bank list:', 'handleFormSubmission', result.banks);
                    const banksContainer = document.getElementById('banks');

                    if (banksContainer) {
                        const newBankIds = new Set(result.banks.map(bank => bank.id));

                        Array.from(banksContainer.children).forEach(button => {
                            const bankId = button.getAttribute('data-bank-id');
                            if (!newBankIds.has(bankId)) {
                                banksContainer.removeChild(button);
                            }
                        });

                        result.banks.forEach(bank => {
                            let bankButton = banksContainer.querySelector(`button[data-bank-id="${bank.id}"]`);

                            if (!bankButton) {
                                bankButton = document.createElement('button');
                                bankButton.setAttribute('data-bank-id', bank.id);
                                bankButton.setAttribute('data-url', `/bank/${bank.id}`);
                                bankButton.setAttribute('style', 'width: 100%');
                                bankButton.textContent = bank.name;

                                bankButton.addEventListener("click", function () {
                                    const url = this.getAttribute("data-url");
                                    log('Bank button clicked. Loading content from URL:', 'handleFormSubmission', url);
                                    loadContent(url);
                                });

                                banksContainer.appendChild(bankButton);
                            } else {
                                bankButton.textContent = bank.name;
                            }
                        });

                        log('Bank list updated.', 'handleFormSubmission');
                    }
                }
            } else if (result.error) {
                error('Form submission error:', 'handleFormSubmission', result.error);
                errorDiv.textContent = result.error;
                errorDiv.style.display = 'block';
                successDiv.style.display = 'none';
            }

            if (!result.error) form.reset();
        } catch (err) {
            error('An unexpected error occurred:', 'handleFormSubmission', err);
            errorDiv.textContent = `An unexpected error occurred: ${err.message}`;
            errorDiv.style.display = 'block';
            successDiv.style.display = 'none';
        }
    });
}

// Function to initialize form handling for multiple forms
function initializeFormHandling() {
    log('Initializing form handling for all forms:', 'initializeFormHandling');
    const forms = document.querySelectorAll('form');

    forms.forEach(form => {
        if (form.id !== 'logout-form') {
            handleFormSubmission(form);
        }
    });
}

// Initialize event listeners when DOM content is loaded
document.addEventListener("DOMContentLoaded", function () {
    log('DOM content loaded. Initializing sidebar buttons and loading default content:', 'DOMContentLoaded');
    document.querySelectorAll(".sidebar-left button").forEach(button => {
        button.addEventListener("click", function () {
            const url = this.getAttribute("onclick").match(/'([^']+)'/)[1];
            log('Sidebar button clicked. Loading content from URL:', 'DOMContentLoaded', url);
            loadContent(url);
        });
    });

    // Optionally load initial content (e.g., dashboard) as default
    loadContent('/dashboard');
});

// Function to format numbers and apply color based on their value
function formatAndColorNumbers() {
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
            let value = parseFloat(element.textContent).toFixed(2); // Format to 2 decimal places

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

function update_performance(result) {
    const total_transactions = document.getElementById("total_transactions");
    const net_gain_loss = document.getElementById("net_gain_loss");
    const performance_percentage = document.getElementById("performance_percentage");
    const average_transaction_amount = document.getElementById("average_transaction_amount");
    const total_discrepancy = document.getElementById("total_discrepancy");

    if (total_transactions && net_gain_loss && performance_percentage && average_transaction_amount && total_discrepancy) {
        total_transactions.textContent = result.performance_value.total_transactions;
        net_gain_loss.textContent = result.performance_value.net_gain_loss;
        performance_percentage.textContent = result.performance_value.performance_percentage;
        average_transaction_amount.textContent = result.performance_value.average_transaction_amount;
        total_discrepancy.textContent = result.performance_value.total_discrepancy;

        formatAndColorNumbers();

        log('Performance metrics updated and formatted successfully:', 'handleFormSubmission');
    }
    else {
        // Log missing elements
        error('One or more performance elements are missing.', 'handleFormSubmission', {
            total_transactions,
            net_gain_loss,
            performance_percentage,
            average_transaction_amount,
            total_discrepancy
        });
    }
}

function update_graph() {
    const layout = {
        title: 'Bank Account Balances',
        xaxis: { title: 'Date', type: 'date' },
        yaxis: { title: 'Balance' },
        hovermode: 'closest',
        plot_bgcolor: 'rgba(0,0,0,0)',
        paper_bgcolor: 'rgba(0,0,0,0)',
    };

    const config = {
        displayModeBar: true,
        modeBarButtonsToRemove: [
            'zoom', 'pan', 'hoverClosestCartesian', 'hoverCompareCartesian', 'zoomIn2d', 'zoomOut2d',
            'pan2d', 'select2d', 'lasso2d', 'zoom3d', 'pan3d', 'orbitRotation', 'tableRotation',
            'resetCameraDefault3d', 'resetCameraLastSave3d', 'toImage', 'sendDataToCloud',
            'toggleSpikelines', 'zoomInGeo', 'zoomOutGeo', 'resetGeo', 'resetMapbox'
        ],
        modeBarButtons: [['toImage', 'resetViews']]
    };

    // Initialize Plotly chart if data is available
    if (window.plotData && window.plotData.length) {
        log('Plotly chart data available:', 'initializeChartAndDatePicker', window.plotData);
        Plotly.newPlot('balance_graph', window.plotData, layout, config);
    } else {
        log('No plot data available for Plotly chart.', 'initializeChartAndDatePicker');
    }

}