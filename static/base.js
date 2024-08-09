// Function to load content dynamically
function loadContent(url) {
    document.getElementById('main-content').innerHTML = '';
    document.getElementById('main-content').innerHTML = '<p>Loading...</p>'; // Show loading state
    fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error('Network response was not ok');
            }
            return response.text();
        })
        .then(html => {
            document.getElementById('main-content').innerHTML = html;

            // Reinitialize chart and date picker for both dashboard and bank views
            if (url === '/dashboard' || /^\/bank\/\d+$/.test(url)) {
                initializeChartAndDatePicker();
                initializeFormHandling();
            }

            // Reinitialize form handling if on add bank page
            if (url === '/add-bank') {
                initializeFormHandling();
            }
        })
        .catch(error => {
            console.error('Error loading content:', error);
            document.getElementById('main-content').innerHTML = '<p>Error loading content. Please try again.</p>';
        });
}

// Function to initialize the Plotly chart and Flatpickr date range picker
function initializeChartAndDatePicker() {

    // Check if plotData exists
    var plotData = window.plotData || [];

    // Define Plotly chart layout and configuration
    var layout = {
        title: 'Bank Account Balances',
        xaxis: { title: 'Date', type: 'date' },
        yaxis: { title: 'Balance' },
        hovermode: 'closest',
        plot_bgcolor: 'rgba(0,0,0,0)',
        paper_bgcolor: 'rgba(0,0,0,0)',
    };

    var config = {
        displayModeBar: true,
        modeBarButtonsToRemove: [
            'zoom', 'pan', 'resetScale', 'hoverClosestCartesian',
            'hoverCompareCartesian', 'zoomIn2d', 'zoomOut2d',
            'pan2d', 'select2d', 'lasso2d', 'zoom3d', 'pan3d',
            'orbitRotation', 'tableRotation', 'resetCameraDefault3d',
            'resetCameraLastSave3d', 'toImage', 'sendDataToCloud',
            'toggleSpikelines', 'resetViews', 'zoomInGeo',
            'zoomOutGeo', 'resetGeo', 'resetMapbox'
        ],
        modeBarButtons: [['toImage']]
    };

    // Initialize Plotly chart if data is available
    if (plotData.length) {
        Plotly.newPlot('balance_graph', plotData, layout, config);
    }

    setTimeout(function () {
        flatpickr("#dateRange", {
            mode: "range",
            dateFormat: "Y-m-d",
            onChange: function (selectedDates) {
                if (selectedDates.length === 2) {
                    var startDate = selectedDates[0].toISOString().split('T')[0];
                    var endDate = selectedDates[1].toISOString().split('T')[0];

                    var update = {
                        'xaxis.range': [startDate, endDate]
                    };

                    Plotly.relayout('balance_graph', update);

                    // Make AJAX request to update date range
                    fetch(`/update_date_range/${startDate}/${endDate}`, {
                        method: 'GET',
                        headers: {
                            'Content-Type': 'application/json'
                        }
                    })
                        .then(response => response.json())
                        .then(data => {
                            // Handle response if needed
                            console.log('Date range updated successfully:', data);
                        })
                        .catch(error => {
                            console.error('Error updating date range:', error);
                        });
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
                console.log('Parsed JSON result:', result);
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
                    console.log('Updating graph with new data:', result.graph_data);
                    window.plotData = result.graph_data;
                    initializeChartAndDatePicker(); // Reinitialize chart with new data
                }
            } else if (result.error) {
                errorDiv.textContent = result.error;
                errorDiv.style.display = 'block';
                successDiv.style.display = 'none';
            }

            if (!result.error) form.reset();
        } catch (error) {
            console.error('An unexpected error occurred:', error);
            errorDiv.textContent = `An unexpected error occurred: ${error.message}`;
            errorDiv.style.display = 'block';
            successDiv.style.display = 'none';
        }
    });
}

// Function to initialize form handling for multiple forms
function initializeFormHandling() {
    const forms = document.querySelectorAll('form');

    forms.forEach(form => {
        handleFormSubmission(form);
    });
}

// Initialize event listeners when DOM content is loaded
document.addEventListener("DOMContentLoaded", function () {
    console.log('DOM fully loaded and parsed');

    // Attach event listeners to sidebar buttons
    document.querySelectorAll(".sidebar-left button").forEach(button => {
        button.addEventListener("click", function () {
            const url = this.getAttribute("onclick").match(/'([^']+)'/)[1];
            loadContent(url);
        });
    });

    // Optionally load initial content (e.g., dashboard) as default
    loadContent('/dashboard');
});
