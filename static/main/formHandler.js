import { error, log, loadContent, handlePageWithGraphData } from './main.js';
import { displayCustomAlert, parseJsonResponse } from './utils.js';

// Main form submission handler
async function handleFormSubmission(form) {
    form.addEventListener('submit', async function (event) {
        event.preventDefault();

        const formData = new FormData(form);

        try {
            const response = await fetch(form.action, {
                method: form.method,
                body: formData
            });

            if (!response.ok) throw new Error(`HTTP error! Status: ${response.status}`);

            const result = await parseJsonResponse(response);

            log('Form submission result:', 'handleFormSubmission', result);

            if (result.banks) {
                updateBankList(result.banks);
            }

            if (result.success) {
                if (result.header === 'Successfully read the csv file') {
                    await handlePageWithGraphData();
                }

                displayCustomAlert('success', result.header, result.success);
            } else if (result.error) {
                displayCustomAlert('error', result.header, result.error);
            }

            if (!result.error) form.reset();
        } catch (err) {
            error('Error submitting form:', 'handleFormSubmission', err);
        }
    });
}

// Function to initialize form handling
export function initializeFormHandling() {
    const forms = document.querySelectorAll('form');

    forms.forEach(form => {
        handleFormSubmission(form);
    });
}

// Function to update the bank list
function updateBankList(banks) {
    const banksContainer = document.getElementById('banks');

    if (banksContainer) {
        const newBankIds = new Set(banks.map(bank => bank.id));

        // Remove buttons for banks that no longer exist
        Array.from(banksContainer.children).forEach(bankButtonContainer => {
            const bankId = bankButtonContainer.getAttribute('data-bank-id');
            if (!newBankIds.has(bankId)) {
                log(`Removing bank button for bank ID ${bankId}.`, 'handleFormSubmission');
                banksContainer.removeChild(bankButtonContainer);
            }
        });

        // Add or update bank buttons
        banks.forEach(bank => updateOrCreateBankButton(bank, banksContainer));
    }
}

// Function to create or update a bank button
function updateOrCreateBankButton(bank, banksContainer) {
    let bankButtonContainer = banksContainer.querySelector(`div[data-bank-id="${bank.id}"]`);

    if (!bankButtonContainer) {
        log(`Creating bank button for bank ID ${bank.id}.`, 'handleFormSubmission');
        bankButtonContainer = createBankButtonContainer(bank);
        banksContainer.appendChild(bankButtonContainer);
    } else {
        const bankButton = bankButtonContainer.querySelector('.bank-button');
        if (bankButton) {
            log(`Updating bank button for bank ID ${bank.id}.`, 'handleFormSubmission');
            bankButton.textContent = bank.name;
        }
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
    bankButton.setAttribute('url', `/bank/${bank.id}`);

    bankButton.addEventListener("click", function () {
        loadContent(this.getAttribute("url"));

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
    button.setAttribute('url', url);
    button.style.width = '100%';

    button.addEventListener("click", function () {
        loadContent(this.getAttribute("url"));
    });

    return button;
}