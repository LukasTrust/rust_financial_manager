import { initializeFormHandling } from './formHandler.js';
import { update_performance } from './performanceUpdater.js';
import { initializeChartAndDatePicker } from './chartManager.js';
import { loadContracts } from './contractManager.js';
import { setupTransactions } from './transactionManager.js';
import { initializeSettings } from './settings.js';

export function log(message, context = '', ...data) {
    if (release_mode) {
        return;
    }
    console.log(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}

export function error(message, context = '', ...data) {
    if (release_mode) {
        return;
    }
    console.error(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}

let oldUrl = localStorage.getItem('old_url') || '/dashboard';
let release_mode = false;

document.addEventListener("DOMContentLoaded", function () {
    const release_mode_element = document.getElementById('release_mode');

    if (release_mode_element) {
        console.log(release_mode_element.value);
        release_mode = release_mode_element.value === 'True';
    }

    log('DOM content loaded. Initializing sidebar buttons and loading default content:', 'DOMContentLoaded');

    document.querySelectorAll("button").forEach(button => {
        const url = button.getAttribute('url');
        if (url) {
            button.addEventListener("click", function () {
                loadContent(url);
                oldUrl = url;
                if (url !== '/logout') {
                    localStorage.setItem('old_url', oldUrl);
                } else {
                    localStorage.removeItem('old_url');
                }
            });
        }
    });

    loadContent(oldUrl);
});

// Main function to load content
export async function loadContent(url) {
    try {
        log('Loading content from URL:', 'loadContent', url);

        const html = await fetchContent(url);

        if (url === '/logout') {
            window.location.href = '/';
            return;
        }

        // Set the main content to the fetched HTML
        document.getElementById('main-content').innerHTML = html;

        if (/^\/bank\/\d+$/.test(url)) {
            handleBankPage(url);
        } else {
            if (url !== '/bank/contract' && url !== '/bank/transaction') {
                hideAllSubButtons();
            }

            switch (url) {
                case '/dashboard':
                case /^\/bank\/\d+$/.test(url) ? url : null:
                    initializeFormHandling();
                    handlePageWithGraphData();
                    break;
                case '/bank/contract':
                    loadContracts();
                    break;
                case '/bank/transaction':
                    setupTransactions();
                    break;
                case '/add-bank':
                    initializeFormHandling();
                    break;
                case '/settings':
                    initializeSettings();
                    break;
                default:
                    log('No specific content handler for URL:', 'loadContent', url);
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

    if (!response.ok) throw new Error(`Network response was not ok: ${response.statusText}`);

    return response.text();
}

// Function to handle bank page specific logic
function handleBankPage(url) {
    const bankId = url.split('/').pop();

    log('Displaying sub-buttons for the bank:', 'handleBankPage', bankId);

    hideAllSubButtons();

    const bankButtonContainer = document.querySelector(`div[data-bank-id="${bankId}"]`);
    if (bankButtonContainer) {
        const subButtons = bankButtonContainer.querySelector('.bank-sub-buttons');
        if (subButtons) {
            subButtons.style.display = 'block';
        }
    }

    initializeFormHandling();
    handlePageWithGraphData();
}

// Function to hide all sub-buttons when not on a bank page
function hideAllSubButtons() {
    document.querySelectorAll('.bank-sub-buttons').forEach(subButtons => {
        subButtons.style.display = 'none';
    });
}

// Function to fetch graph data
async function fetchGraphData() {
    const response = await fetch('/get/graph/data');

    if (!response.ok) throw new Error(`Network response was not ok: ${response.statusText}`);

    const json = await response.json();
    return json;
}

export async function handlePageWithGraphData() {
    try {
        const data = await fetchGraphData();

        initializeChartAndDatePicker(data.graph_data);
        update_performance(data.performance_value);
    } catch (err) {
        error('Error handling page with graph data:', 'handlePageWithGraphData', err);
    }
}
