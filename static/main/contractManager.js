import { formatDate, displayCustomAlert } from './utils.js';

// Main function to load contracts
export function loadContracts() {
    try {
        const contractsDataScript = document.getElementById('contracts-data');
        if (!contractsDataScript) throw new Error('Contracts data script element not found.');

        const contractsData = JSON.parse(contractsDataScript.textContent);
        if (!Array.isArray(contractsData)) throw new Error('Unexpected data format.');

        const container = document.getElementById('display-container');
        container.innerHTML = '';

        if (contractsData.length === 0) {
            return;
        }

        setupEventListeners();

        const openContractsWrapper = document.createElement('div');
        openContractsWrapper.classList.add('display-container');
        const closedContractsWrapper = document.createElement('div');
        closedContractsWrapper.classList.add('display-container');

        contractsData.forEach((contractWithHistory, index) => {
            const contractHTML = generateContractHTML(contractWithHistory, index);
            const wrapper = contractWithHistory.contract.end_date ? closedContractsWrapper : openContractsWrapper;
            wrapper.insertAdjacentHTML('beforeend', contractHTML);
        });

        const openContractsTitle = document.createElement('h3');
        openContractsTitle.textContent = 'Open Contracts';
        container.appendChild(openContractsTitle);
        container.appendChild(openContractsWrapper);

        const closedContractsTitle = document.createElement('h3');
        closedContractsTitle.textContent = 'Closed Contracts';
        container.appendChild(closedContractsTitle);
        container.appendChild(closedContractsWrapper);

        // Event listener to handle contract selection and history toggling
        container.addEventListener('click', (event) => {
            const target = event.target;

            // Handle button clicks for toggling history
            if (target.classList.contains('toggle-history-btn')) {
                event.stopPropagation(); // Prevent the event from bubbling up
                const index = target.getAttribute('data-index');
                const historyElement = document.getElementById(`contract-history-${index}`);
                const isHidden = historyElement.classList.toggle('hidden');
                target.textContent = isHidden ? 'Show History' : 'Hide History';
                target.setAttribute('aria-expanded', isHidden ? 'false' : 'true');
                return; // Exit to prevent further handling
            }

            // Handle contract box clicks
            const contractElement = target.closest('.display');
            if (contractElement) {
                const isSelected = contractElement.classList.contains('selected');

                if (isSelected) {
                    contractElement.classList.remove('selected');
                }
                else {
                    contractElement.classList.add('selected');
                }
            }
        });

    } catch (err) {
    }
}

// Generate HTML for contract history
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

// Generate HTML for a single contract
function generateContractHTML(contractWithHistory, index) {
    const { contract, contract_history, total_amount_paid, last_payment_date } = contractWithHistory;

    const currentAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
    const totalAmountClass = total_amount_paid < 0 ? 'negative' : 'positive';

    const dateLabel = contract.end_date ? 'End date' : 'Last payment date';
    const dateValue = contract.end_date ? formatDate(contract.end_date) : formatDate(last_payment_date);

    return `
        <div class="display" id="display-${index}" data-id="${index}">
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

function setupEventListeners() {
    document.getElementById('merge-selected-btn').addEventListener('click', mergeSelectedContracts);
    //document.getElementById('delete-selected-btn').addEventListener('click', deleteSelectedContracts);
    //document.getElementById('scan-btn').addEventListener('click', editSelectedContracts);
}

async function mergeSelectedContracts() {
    const selectedContracts = document.querySelectorAll('.selected');

    if (selectedContracts.length < 2) {
        displayCustomAlert('error', 'Merge contracts', 'Please select at least 2 contracts to merge.');
        return;
    }

    // Extract contract IDs
    const contractIDs = Array.from(selectedContracts).map((contract) => contract.getAttribute('data-id'));
    const contractIDsAsIntegers = contractIDs.map(id => parseInt(id, 10));

    const response = await fetch('/bank/contract/merge', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ ids: contractIDsAsIntegers }),
    });

    if (!response.ok) {
        throw new Error('Failed to send IDs to the server');
    }

    console.log(response);

    const data = await response.json();

    console.log(data);

    if (data.error) {
        displayCustomAlert('error', data.header, data.error);
        return;
    }
}