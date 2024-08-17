import { log, error } from './logger.js';
import { initializeFormHandling } from './formHandler.js';
import { initializeChartAndDatePicker } from './chartManager.js';
import { formatAndColorNumbers } from './performanceUpdater.js';
import { loadContracts } from './contractManager.js';

// Function to fetch content from a URL
async function fetchContent(url) {
    log('Loading content from URL:', 'loadContent', url);
    document.getElementById('main-content').innerHTML = '<p>Loading...</p>';

    const response = await fetch(url);
    if (!response.ok) throw new Error('Network response was not ok');

    return response.text();
}

// Function to handle login validation errors
function handleLoginError(html) {
    if (html.includes('Please login again')) {
        log('Error validating the login. Redirecting to error page:', 'loadContent');
        window.location.href = '/error?error_title=Error%20validating%20the%20login!&error_message=Please%20login%20again.';
        return true;
    }
    return false;
}

// Function to parse and handle graph data
function parseGraphData() {
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
}

// Function to handle bank page specific logic
function handleBankPage(url) {
    // Log the action and URL
    log('Displaying sub-buttons for the bank:', 'loadContent', url);

    // Extract bank ID from the URL
    const bankId = url.split('/').pop();

    // Iterate over all bank sub-button containers
    document.querySelectorAll('.bank-sub-buttons').forEach(subButtons => {
        const parentContainer = subButtons.closest('div[data-bank-id]');
        if (parentContainer) {
            const currentBankId = parentContainer.getAttribute('data-bank-id');
            // Hide sub-buttons only if they do not belong to the current bankId
            if (currentBankId !== bankId) {
                subButtons.style.display = 'none';
            }
        }
    });

    // Find the container for the specific bank
    const bankButtonContainer = document.querySelector(`div[data-bank-id="${bankId}"]`);

    if (bankButtonContainer) {
        // Find the sub-buttons within the bank's container
        const subButtons = bankButtonContainer.querySelector('.bank-sub-buttons');

        if (subButtons) {
            // Display the sub-buttons for the current bank
            subButtons.style.display = 'block';
        }
    }

    // Reinitialize features with the provided URL
    reinitializeFeatures(url);
}


// Function to hide all sub-buttons when not on a bank page
function hideAllSubButtons(url) {
    log('Hiding all sub-buttons:', 'loadContent', url);
    document.querySelectorAll('.bank-sub-buttons').forEach(subButtons => {
        subButtons.style.display = 'none';
    });
}

// Function to reinitialize form handling and other features
function reinitializeFeatures(url) {
    log('Reinitializing features for URL:', 'loadContent', url);
    initializeFormHandling();
    formatAndColorNumbers();
    initializeChartAndDatePicker();
}

// Function to handle special pages (dashboard, contract, add-bank)
function handleSpecialPages(url) {
    if (url === '/dashboard' || /^\/bank\/\d+$/.test(url)) {
        reinitializeFeatures(url);
    } else if (url === '/contract') {
        loadContracts();
    } else if (url === '/add-bank') {
        log('Reinitializing form handling for add bank page:', 'loadContent');
        initializeFormHandling();
    }
}

// Main function to load content
export async function loadContent(url) {
    try {
        const html = await fetchContent(url);

        if (handleLoginError(html)) return;

        document.getElementById('main-content').innerHTML = html;

        parseGraphData();
        if (/^\/bank\/\d+$/.test(url)) {
            handleBankPage(url);
        } else {
            hideAllSubButtons(url);
            handleSpecialPages(url);
        }

    } catch (err) {
        error('Error loading content:', 'loadContent', err);
        document.getElementById('main-content').innerHTML = '<p>Error loading content. Please try again.</p>';
    }
}
