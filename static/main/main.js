import { initializeFormHandling } from './formHandler.js';
import { update_performance } from './performanceUpdater.js';
import { initializeChartAndDatePicker } from './chartManager.js';
import { loadContracts } from './contractManager.js';
import { setupTransactions } from './transactionManager.js';
import { initializeSettings } from './settings.js';
import { closeModal, displayCustomAlert, getLocalizedString } from './utils.js';

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

    addDeleteBankListener();

    initializeFormHandling();
    handlePageWithGraphData();
}

function addDeleteBankListener() {
    document.getElementById('delete_bank_button').addEventListener('click', handleDeleteBankButton);

    log('Delete bank button listener added', 'addDeleteBankListener');
}

function handleDeleteBankButton() {
    log('Prepare to handle delete bank button click', 'handleDeleteButton');

    // Create backdrop div
    const backdrop = document.createElement('div');
    backdrop.id = 'deleteModal';
    backdrop.className = 'alert-backdrop';

    // Create the modal container
    const modal = document.createElement('div');
    modal.className = 'alert alert-info';

    const horizontalContainer = document.createElement('div');
    horizontalContainer.className = 'container-without-border-horizontally-center';

    // Add icon and header text
    const icon = document.createElement('img');
    icon.src = '/static/images/info.png';
    icon.alt = 'Icon';
    icon.classList.add('icon-big');

    const headerText = document.createElement('strong');
    headerText.textContent = getLocalizedString('delete_bank_header');

    const flexDiv = document.createElement('div');
    flexDiv.style.flexGrow = '1';
    flexDiv.appendChild(headerText);

    horizontalContainer.appendChild(icon);
    horizontalContainer.appendChild(flexDiv);

    // Create body text
    const bodyText = document.createElement('p');
    bodyText.textContent = getLocalizedString('delete_bank_confirmation');

    const buttonContainer = document.createElement('div');
    buttonContainer.classList.add('container-without-border-horizontally-center');

    // Create delete button
    const deleteButton = document.createElement('button');
    deleteButton.textContent = getLocalizedString('delete_bank_button');
    deleteButton.classList.add('btn-danger', 'button', 'btn-secondary')
    deleteButton.onclick = sendDeleteBankRequest;

    // Create cancel button
    const cancelButton = document.createElement('button');
    cancelButton.textContent = getLocalizedString('cancel_button');
    cancelButton.classList.add('button', 'btn-secondary')
    cancelButton.onclick = closeModal;

    buttonContainer.appendChild(deleteButton);
    buttonContainer.appendChild(cancelButton);

    const container = document.createElement('div');
    container.className = 'container-without-border';
    container.appendChild(horizontalContainer);
    container.appendChild(bodyText);
    container.appendChild(buttonContainer);

    modal.appendChild(container);

    backdrop.appendChild(modal);

    document.body.appendChild(backdrop);

    log('Delete bank modal displayed', 'handleDeleteButton');
}

function sendDeleteBankRequest() {
    log('Sending bank delete request', 'sendDeleteBankRequest');

    fetch('/delete_bank', {
        method: 'GET'
    })
        .then(async response => {
            if (response.ok) {
                log('Delete bank request successful', 'sendDeleteBankRequest');

                let json = await response.json();
                log('Delete request response JSON:', 'sendDeleteBankRequest', json);

                if (json.success) {
                    displayCustomAlert('success', json.header, json.success, '', 5);
                    setTimeout(() => {
                        window.location.href = '/base';
                    }, 5000);
                } else if (json.error) {
                    displayCustomAlert('error', json.header, json.error);
                }
            } else {
                error('Failed to delete bank. Status: ' + response.status, 'sendDeleteBankRequest');
            }
        })
        .catch(err => {
            error('Error deleting bank', 'sendDeleteBankRequest', err);
        });
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
