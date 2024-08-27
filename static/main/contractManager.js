import { formatDate, displayCustomAlert } from './utils.js';

const closedContractsWrapper = document.createElement('div');
const openContractsWrapper = document.createElement('div');

export function loadContracts() {
    try {
        const contractsDataScript = document.getElementById('contracts-data');
        if (!contractsDataScript) throw new Error('Contracts data script element not found.');

        const contractsData = JSON.parse(contractsDataScript.textContent);
        if (!Array.isArray(contractsData)) throw new Error('Unexpected data format.');

        closedContractsWrapper.classList.add('display-container');
        openContractsWrapper.classList.add('display-container');

        updateContractsView(contractsData);
        setupEventListeners();
    } catch (err) {
        displayCustomAlert('error', 'Loading Error', err.message);
    }
}

function updateContractsView(contractsData) {
    const container = document.getElementById('contract-container');

    // Clear the wrappers
    closedContractsWrapper.innerHTML = '';
    openContractsWrapper.innerHTML = '';

    // Populate the wrappers with updated contracts
    contractsData.forEach((contractWithHistory, index) => {
        const contractId = contractWithHistory.contract.id;
        const contractHTML = generateContractHTML(contractWithHistory, index);
        const wrapper = contractWithHistory.contract.end_date ? closedContractsWrapper : openContractsWrapper;
        wrapper.insertAdjacentHTML('beforeend', contractHTML);
    });

    // Clear and re-append the updated contract sections to the container
    container.innerHTML = '';

    const openContractsTitle = document.createElement('h3');
    openContractsTitle.textContent = 'Open Contracts';
    container.appendChild(openContractsTitle);
    container.appendChild(openContractsWrapper);

    const closedContractsTitle = document.createElement('h3');
    closedContractsTitle.textContent = 'Closed Contracts';
    container.appendChild(closedContractsTitle);
    container.appendChild(closedContractsWrapper);

    attachContractEventListeners();
}

function attachContractEventListeners() {
    const container = document.querySelectorAll('.display');

    container.forEach((contractElement, index) => {
        contractElement.addEventListener('click', (event) => {
            const target = event.target;

            if (target.classList.contains('toggle-history-btn')) {
                event.stopPropagation();
                const index = target.getAttribute('data-index');
                const historyElement = document.getElementById(`contract-history-${index}`);
                const isHidden = historyElement.classList.toggle('hidden');
                target.textContent = isHidden ? 'Show History' : 'Hide History';
                target.setAttribute('aria-expanded', isHidden ? 'false' : 'true');
                return;
            }

            const contractElement = target.closest('.display');
            if (contractElement) {
                const isSelected = contractElement.classList.contains('selected');

                if (isSelected) {
                    contractElement.classList.remove('selected');
                } else {
                    contractElement.classList.add('selected');
                }
            }
        })
    });
}

function setupEventListeners() {
    document.getElementById('merge-selected-btn').addEventListener('click', mergeSelectedContracts);
    // Additional event listeners...
}

async function mergeSelectedContracts() {
    const selectedContracts = document.querySelectorAll('.selected');

    if (selectedContracts.length < 2) {
        displayCustomAlert('error', 'Merge contracts', 'Please select at least 2 contracts to merge.');
        return;
    }

    const contractIDs = Array.from(selectedContracts).map(contract => parseInt(contract.getAttribute('data-id'), 10));

    try {
        const response = await fetch('/bank/contract/merge', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ ids: contractIDs }),
        });

        if (!response.ok) throw new Error('Failed to send IDs to the server');

        const updatedContractsData = await response.json();

        if (updatedContractsData.error) {
            displayCustomAlert('error', updatedContractsData.header, updatedContractsData.error);
            return;
        }

        const dateJSON = JSON.parse(updatedContractsData);

        updateContractsView(dateJSON);

        displayCustomAlert('success', 'Merge Successful', 'Contracts have been successfully merged.');

    } catch (err) {
        displayCustomAlert('error', 'Merge Failed', err.message);
    }
}

function generateContractHTML(contractWithHistory, index) {
    const { contract, contract_history, total_amount_paid, last_payment_date } = contractWithHistory;

    const currentAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
    const totalAmountClass = total_amount_paid < 0 ? 'negative' : 'positive';

    const dateLabel = contract.end_date ? 'End date' : 'Last payment date';
    const dateValue = contract.end_date ? formatDate(contract.end_date) : formatDate(last_payment_date);

    return `
        <div class="display" id="display-${index}" data-id="${contract.id}">
            <h3>${contract.name}</h3>
            <p>Current amount: <span class="${currentAmountClass}">$${contract.current_amount.toFixed(2)}</span></p>
            <p>Total amount over time: <span class="${totalAmountClass}">$${total_amount_paid.toFixed(2)}</span></p>
            <p>Months between Payment: ${contract.months_between_payment}</p>
            <p>${dateLabel}: <span>${dateValue}</span></p>
            <button class="toggle-history-btn" data-index="${index}">Show History</button>
            <div id="contract-history-${index}" class="hidden contract-history">
                <h4>Contract History:</h4>
                <ul>${generateHistoryHTML(contract_history)}</ul>
            </div>
        </div>
    `;
}

function generateHistoryHTML(contractHistory) {
    if (contractHistory.length === 0) {
        return '<li>No history available.</li>';
    }

    return contractHistory.map(({ old_amount, new_amount, changed_at }) => `
        <li>
            <p>Old Amount: <span class="${old_amount < 0 ? 'negative' : 'positive'}">$${old_amount.toFixed(2)}</span></p>
            <p>New Amount: <span class="${new_amount < 0 ? 'negative' : 'positive'}">$${new_amount.toFixed(2)}</span></p>
            <p>Changed At: ${formatDate(changed_at)}</p>
        </li>
    `).join('');
}
