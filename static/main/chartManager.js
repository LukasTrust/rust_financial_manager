import { log, error } from './logger.js';
import { update_graph } from './graphUpdater.js';
import { update_performance } from './performanceUpdater.js';

export function initializeChartAndDatePicker() {
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
                                log('Update date range form submitted successfully. Updating performance metrics:', 'initializeChartAndDatePicker', data.performance_value);
                            }

                            if (data.graph_data) {
                                window.plotData = JSON.parse(data.graph_data);
                                log('Update date range form submitted successfully. Reinitializing chart with new data:', 'initializeChartAndDatePicker', window.plotData);
                                update_graph();
                            }
                        })
                        .catch(err => error('Error updating date range:', 'initializeChartAndDatePicker', err));
                }
            }
        });
    }, 0);
}
