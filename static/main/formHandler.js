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
                if (result.header === 'Successfully read the CSV file' || result.header === 'CSV-Datei erfolgreich gelesen') {
                    await handlePageWithGraphData();
                }

                displayCustomAlert('success', result.header, result.success);
            } else if (result.error) {
                displayCustomAlert('error', result.header, result.error);
            }

            if (!result.error) form.reset();
        } catch (err) {
            error(`Error submitting form: ${err.message}`, 'handleFormSubmission', err);
        }
    });
}

// Function to initialize form handling
export function initializeFormHandling() {
    const forms = document.querySelectorAll('form');

    forms.forEach(form => handleFormSubmission(form));
}

// Function to update the bank list
function updateBankList(banks) {
    const banksContainer = document.getElementById('banks');

    if (banksContainer) {
        // Add or update bank buttons
        banks.forEach(bank => createNewBankButton(bank, banksContainer));
    }
}

// Function to create or update a bank button
function createNewBankButton(bank, banksContainer) {
    let bankButtonContainer = banksContainer.querySelector(`div[data-bank-id="${bank.id}"]`);

    if (!bankButtonContainer) {
        log(`Creating bank button for bank ID ${bank.id}.`, 'updateOrCreateBankButton');
        bankButtonContainer = createBankButtonContainer(bank);
        banksContainer.appendChild(bankButtonContainer);
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
    // Create button element
    const bankButton = document.createElement('button');
    bankButton.classList.add('bank-button', 'button', 'btn-secondary');
    bankButton.setAttribute('url', `/bank/${bank.id}`);

    // Create img element for the icon
    const iconImage = document.createElement('img');
    iconImage.src = "/static/images/bank.png";
    iconImage.alt = "Icon";
    iconImage.style.marginRight = "5px";

    // Append the icon to the button
    bankButton.appendChild(iconImage);

    // Create text node for bank name and append it
    const buttonText = document.createTextNode(bank.name);
    bankButton.appendChild(buttonText);

    // Add click event listener
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
    subButtonsContainer.classList.add('button', 'bank-sub-buttons');
    subButtonsContainer.style.display = 'none';

    const contractButton = createSubButton('Contract', `/bank/contract`, '/static/images/contract.png');
    const transactionButton = createSubButton('Transaction', `/bank/transaction`, '/static/images/transaction.png');

    subButtonsContainer.appendChild(contractButton);
    subButtonsContainer.appendChild(transactionButton);

    return subButtonsContainer;
}

// Function to create a single sub-button
function createSubButton(text, url, imageUrl) {
    const button = document.createElement('button');
    button.classList.add('button', 'btn-secondary');
    button.setAttribute('url', url);
    button.style.width = '100%';

    const img = document.createElement('img');
    img.src = imageUrl;
    img.alt = `${text} Icon`;

    button.appendChild(img);
    button.appendChild(document.createTextNode(text));

    button.addEventListener("click", function () {
        loadContent(this.getAttribute("url"));
    });

    return button;
}
