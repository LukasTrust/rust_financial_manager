import { log, error } from './logger.js';
import { initializeFormHandling } from './formHandler.js';
import { initializeChartAndDatePicker } from './chartManager.js';
import { formatAndColorNumbers } from './performanceUpdater.js';
import { loadContracts } from './contractManager.js';

export function loadContent(url) {
    log('Loading content from URL:', 'loadContent', url);
    document.getElementById('main-content').innerHTML = '<p>Loading...</p>';

    fetch(url)
        .then(response => {
            if (!response.ok) throw new Error('Network response was not ok');
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
                    const jsonText = graphDataElement.textContent.trim();
                    window.plotData = JSON.parse(jsonText);
                    log('Graph data successfully parsed:', 'loadContent', window.plotData);
                } catch (e) {
                    error('Error parsing graph data:', 'loadContent', e);
                }
            }

            if (url === '/dashboard' || /^\/bank\/\d+$/.test(url)) {
                log('Reinitializing chart and date picker for URL:', 'loadContent', url);
                initializeFormHandling();
                formatAndColorNumbers();
                initializeChartAndDatePicker();
            }

            if (url === '/add-bank') {
                log('Reinitializing form handling for add bank page:', 'loadContent');
                initializeFormHandling();
            }

            if (url === '/contract') {
                loadContracts();
            }
        })
        .catch(err => {
            error('Error loading content:', 'loadContent', err);
            document.getElementById('main-content').innerHTML = '<p>Error loading content. Please try again.</p>';
        });
}
