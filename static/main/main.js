import { initializeFormHandling } from './formHandler.js';
import { update_performance } from './performanceUpdater.js';
import { initializeChartAndDatePicker } from './chartManager.js';
import { loadContracts } from './contractManager.js';
import { loadTransactions } from './transactionManager.js';

export function log(message, context = '', ...data) {
    console.log(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}

export function error(message, context = '', ...data) {
    console.error(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}

document.addEventListener("DOMContentLoaded", function () {
    log('DOM content loaded. Initializing sidebar buttons and loading default content:', 'DOMContentLoaded');
    document.querySelectorAll("button").forEach(button => {
        const url = button.getAttribute('url');
        if (url) {
            button.addEventListener("click", function () {
                loadContent(url);
            });

        };
    });

    // Display the dashboard by default
    loadContent("/dashboard");
});

// Main function to load content
export async function loadContent(url) {
    try {
        log('Loading content from URL:', 'loadContent', url);

        const html = await fetchContent(url);

        if (url === '/logout') {
            window.location.href = `/`;
            return;
        }

        // Redirect to error page if login validation fails
        if (handleLoginError(html)) return;

        // Set the main content to the fetched HTML
        document.getElementById('main-content').innerHTML = html;

        if (/^\/bank\/\d+$/.test(url)) {
            handleBankPage(url);
        } else {
            if (url !== '/bank/contract' && url !== '/bank/transaction') {
                hideAllSubButtons();
            }

            if (url === '/dashboard' || /^\/bank\/\d+$/.test(url)) {
                initializeFormHandling();
                handlePageWithGraphData();
            } else if (url === '/bank/contract') {
                loadContracts();
            } else if (url === '/bank/transaction') {
                loadTransactions();
            }
            else if (url === '/add-bank') {
                initializeFormHandling();
            }
        }

    } catch (err) {
        error('Error loading content:', 'loadContent', err);
        document.getElementById('main-content').innerHTML = '<p>Error loading content. Please try again.</p>';
    }
}

// Function to fetch content from a URL
async function fetchContent(url) {
    const response = await fetch(url);

    if (!response.ok) throw new Error('Network response was not ok');

    return response.text();
}

// Function to handle login validation errors
function handleLoginError(html) {
    if (html.includes('Please login again')) {
        error('Error validating the login. Redirecting to error page:', 'loadContent');
        window.location.href = '/error?error_title=Error%20validating%20the%20login!&error_message=Please%20login%20again.';
        return true;
    }
    return false;
}

// Function to handle bank page specific logic
function handleBankPage(url) {
    // Extract bank ID from the URL
    const bankId = url.split('/').pop();

    log('Displaying sub-buttons for the bank:', 'handleBankPage', bankId);

    hideAllSubButtons();

    // Find the container for the specific bank
    const bankButtonContainer = document.querySelector(`div[data-bank-id="${bankId}"]`);

    // Find the sub-buttons within the bank's container
    const subButtons = bankButtonContainer.querySelector('.bank-sub-buttons');

    // Display the sub-buttons for the current bank
    subButtons.style.display = 'block';

    initializeFormHandling();
    handlePageWithGraphData();
}

// Function to hide all sub-buttons when not on a bank page
function hideAllSubButtons() {
    document.querySelectorAll('.bank-sub-buttons').forEach(subButtons => {
        subButtons.style.display = 'none';
    });
}

async function featchGraphData() {
    const response = await fetch('/get/graph/data');

    const json = await response.json();

    return json;
}

export async function handlePageWithGraphData() {
    const data = await featchGraphData();

    initializeChartAndDatePicker(data.graph_data);
    update_performance(data.performance_value);
}