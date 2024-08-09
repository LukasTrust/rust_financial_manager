document.addEventListener("DOMContentLoaded", function () {
    initializeDashboard();

    // Function to load content into the main-content div
    function loadContent(url) {
        fetch(url)
            .then(response => {
                if (!response.ok) {
                    throw new Error('Network response was not ok');
                }
                return response.text();
            })
            .then(html => {
                document.getElementById('main-content').innerHTML = html;
                // After loading the content, re-initialize dashboard if needed
                initializeDashboard();
            })
            .catch(error => {
                console.error('Error loading content:', error);
                document.getElementById('main-content').innerHTML = '<p>Error loading content. Please try again.</p>';
            });
    }

    // Attach event listeners to sidebar buttons
    document.querySelectorAll(".sidebar-left button").forEach(button => {
        button.addEventListener("click", function () {
            var url = this.getAttribute("onclick").match(/'([^']+)'/)[1];
            loadContent(url);
        });
    });

    // Load initial content (dashboard) as default
    loadContent('/dashboard');

    // Function to initialize dashboard-specific functionality
    function initializeDashboard() {
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

        // Initialize Flatpickr for date range picker
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
    }
});
