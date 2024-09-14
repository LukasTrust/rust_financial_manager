import { update_performance } from './performanceUpdater.js';
import { log, error } from './main.js';
import { getGlobalLanguage } from './utils.js';

const localizedStrings = {
    English: {
        chart_title: 'Bank Account Balances',
        date: 'Date',
        balance: 'Balance',
    },
    German: {
        chart_title: 'Bankkontostände',
        date: 'Datum',
        balance: 'Kontostand',
    }
};

export function initializeChartAndDatePicker(plotData) {
    log('Initializing chart and date picker', 'initializeChartAndDatePicker');

    // Update graph initially with provided data
    update_graph(plotData);

    setTimeout(() => {
        log('Setting up date picker', 'initializeChartAndDatePicker');

        flatpickr("#dateRange", {
            mode: "range",
            dateFormat: "d-m-Y",
            onChange: function (selectedDates) {
                if (selectedDates.length === 2) {
                    const [startDate, endDate] = selectedDates.map(date => date.toISOString().split('T')[0]);

                    log(`Date range selected: ${startDate} to ${endDate}`, 'initializeChartAndDatePicker');

                    fetch(`/update_date_range/${startDate}/${endDate}`, {
                        method: 'GET',
                        headers: { 'Content-Type': 'application/json' }
                    })
                        .then(response => response.json())
                        .then(data => {
                            if (data.performance_value) {
                                log('Updating performance with new value', 'initializeChartAndDatePicker');
                                update_performance(data.performance_value);
                            } else {
                                log('No performance value received', 'initializeChartAndDatePicker');
                            }

                            if (data.graph_data) {
                                log('Updating graph with new data', 'initializeChartAndDatePicker');
                                update_graph(data.graph_data);
                            } else {
                                log('No graph data received', 'initializeChartAndDatePicker');
                            }
                        })
                        .catch(err => error('Error updating date range:', 'initializeChartAndDatePicker', err));
                } else {
                    log('Date range selection is invalid. Need exactly 2 dates.', 'initializeChartAndDatePicker');
                }
            }
        });
    }, 0);
}

function update_graph(plotData) {
    log('Updating graph with new data', 'update_graph');

    let data;
    try {
        data = JSON.parse(plotData);
    } catch (err) {
        error('Error parsing plot data:', 'update_graph', err);
        return; // Exit if data parsing fails
    }

    const layout = {
        title: localizedStrings[getGlobalLanguage()].chart_title,
        xaxis: { title: localizedStrings[getGlobalLanguage()].date, type: 'date' },
        yaxis: { title: localizedStrings[getGlobalLanguage()].balance },
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

    try {
        Plotly.newPlot('balance_graph', data, layout, config);
        log('Graph plotted successfully', 'update_graph');
    } catch (err) {
        error('Error plotting graph:', 'update_graph', err);
    }
}
