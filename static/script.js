document.addEventListener("DOMContentLoaded", function () {
    // Define your Plotly chart data and layout
    var plotData = window.plotData || [];
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
        modeBarButtons: [
            ['toImage']
        ]
    };

    // Initialize Plotly chart
    Plotly.newPlot('balance_graph', plotData, layout, config);

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
            }
        }
    });
});
