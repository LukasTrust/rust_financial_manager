import { update_performance } from './performanceUpdater.js';

export function initializeChartAndDatePicker() {
    update_graph();

    setTimeout(() => {
        flatpickr("#dateRange", {
            mode: "range",
            dateFormat: "d-m-Y",
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
                            }

                            if (data.graph_data) {
                                window.plotData = JSON.parse(data.graph_data);
                                update_graph();
                            }
                        })
                        .catch(err => error('Error updating date range:', 'initializeChartAndDatePicker', err));
                }
            }
        });
    }, 0);
}

export function update_graph() {
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
        Plotly.newPlot('balance_graph', window.plotData, layout, config);
    } else {
    }
}