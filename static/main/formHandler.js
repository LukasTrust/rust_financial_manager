import { log, error } from './logger.js';
import { initializeChartAndDatePicker } from './chartManager.js';
import { update_performance } from './performanceUpdater.js';
import { loadContent } from './contentLoader.js';

// Function to handle JSON response parsing
async function parseJsonResponse(response) {
    try {
        return await response.json();
    } catch {
        throw new Error('Error parsing JSON response');
    }
}

// Function to handle success response
function handleSuccessResponse(result, successDiv, errorDiv) {
    successDiv.textContent = result.success;
    successDiv.style.display = 'block';
    errorDiv.style.display = 'none';

    if (result.graph_data) {
        window.plotData = JSON.parse(result.graph_data);
        log('Form submitted successfully. Reinitializing chart with new data:', 'handleFormSubmission', window.plotData);
        initializeChartAndDatePicker();
    }

    if (result.performance_value) {
        update_performance(result);
    }

    if (result.banks) {
        updateBankList(result.banks);
    }
}

// Function to handle error response
function handleError(result, errorDiv, successDiv) {
    error('Form submission error:', 'handleFormSubmission', result.error);
    errorDiv.textContent = result.error;
    errorDiv.style.display = 'block';
    successDiv.style.display = 'none';
}

// Function to update the bank list
function updateBankList(banks) {
    log('Updating bank list:', 'handleFormSubmission', banks);
    const banksContainer = document.getElementById('banks');

    if (banksContainer) {
        const newBankIds = new Set(banks.map(bank => bank.id));

        // Remove buttons for banks that no longer exist
        Array.from(banksContainer.children).forEach(bankButtonContainer => {
            const bankId = bankButtonContainer.getAttribute('data-bank-id');
            if (!newBankIds.has(bankId)) {
                banksContainer.removeChild(bankButtonContainer);
            }
        });

        // Add or update bank buttons
        banks.forEach(bank => updateOrCreateBankButton(bank, banksContainer));

        log('Bank list updated.', 'handleFormSubmission');
    }
}

// Function to create or update a bank button
function updateOrCreateBankButton(bank, banksContainer) {
    let bankButtonContainer = banksContainer.querySelector(`div[data-bank-id="${bank.id}"]`);

    if (!bankButtonContainer) {
        bankButtonContainer = createBankButtonContainer(bank);
        banksContainer.appendChild(bankButtonContainer);
    } else {
        const bankButton = bankButtonContainer.querySelector('.bank-button');
        if (bankButton) bankButton.textContent = bank.name;
    }
}

// Function to create a new bank button container
function createBankButtonContainer(bank) {
    const bankButtonContainer = document.createElement('div');
    bankButtonContainer.setAttribute('data-bank-id', bank.id);
    bankButtonContainer.classList.add('bank-button-container');

    const bankButton = createBankButton(bank);
    bankButtonContainer.appendChild(bankButton);

    const subButtonsContainer = createSubButtonsContainer();
    bankButtonContainer.appendChild(subButtonsContainer);

    return bankButtonContainer;
}

// Function to create the main bank button
function createBankButton(bank) {
    const bankButton = document.createElement('button');
    bankButton.classList.add('bank-button');
    bankButton.textContent = bank.name;
    bankButton.setAttribute('data-url', `/bank/${bank.id}`);

    bankButton.addEventListener("click", function () {
        const url = this.getAttribute("data-url");
        log('Bank button clicked. Loading content from URL:', 'handleFormSubmission', url);
        loadContent(url);

        const subButtonsContainer = this.nextElementSibling;
        subButtonsContainer.style.display = subButtonsContainer.style.display === 'none' ? 'block' : 'none';
    });

    return bankButton;
}

// Function to create sub-buttons for a bank
function createSubButtonsContainer() {
    const subButtonsContainer = document.createElement('div');
    subButtonsContainer.classList.add('bank-sub-buttons');
    subButtonsContainer.style.display = 'none';

    const contractButton = createSubButton('Contract', `/bank/contract`);
    const transactionButton = createSubButton('Transaction', `/bank/transaction`);

    subButtonsContainer.appendChild(contractButton);
    subButtonsContainer.appendChild(transactionButton);

    return subButtonsContainer;
}

// Function to create a single sub-button
function createSubButton(text, url) {
    const button = document.createElement('button');
    button.textContent = text;
    button.setAttribute('data-url', url);
    button.style.width = '100%';

    button.addEventListener("click", function () {
        log(`${text} button clicked. Loading content from URL:`, 'handleFormSubmission', url);
        loadContent(url);
    });

    return button;
}

// Main form submission handler
export async function handleFormSubmission(form) {
    form.addEventListener('submit', async function (event) {
        event.preventDefault();

        const formData = new FormData(form);
        const errorDiv = document.getElementById('error');
        const successDiv = document.getElementById('success');

        try {
            const response = await fetch(form.action, {
                method: form.method,
                body: formData
            });

            if (!response.ok) throw new Error(`HTTP error! Status: ${response.status}`);

            const result = await parseJsonResponse(response);

            if (result.success) {
                handleSuccessResponse(result, successDiv, errorDiv);
            } else if (result.error) {
                handleError(result, errorDiv, successDiv);
            }

            if (!result.error) form.reset();
        } catch (err) {
            error('An unexpected error occurred:', 'handleFormSubmission', err);
            errorDiv.textContent = `An unexpected error occurred: ${err.message}`;
            errorDiv.style.display = 'block';
            successDiv.style.display = 'none';
        }
    });
}

// Function to initialize form handling
export function initializeFormHandling() {
    log('Initializing form handling for all forms:', 'initializeFormHandling');
    const forms = document.querySelectorAll('form');

    forms.forEach(form => {
        if (form.id !== 'logout-form') {
            handleFormSubmission(form);
        }
    });
}