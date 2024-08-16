import { log } from './logger.js';

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
        log('Plotly chart data available:', 'initializeChartAndDatePicker', window.plotData);
        Plotly.newPlot('balance_graph', window.plotData, layout, config);
    } else {
        log('No plot data available for Plotly chart.', 'initializeChartAndDatePicker');
    }
}