import { log, error } from './logger.js';

// Utility function to format date in dd.mm.yyyy
function formatDate(dateString) {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'N/A';

    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();

    return `${day}.${month}.${year}`;
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
        <div class="display">
            <input type="checkbox" class="display-checkbox hidden" id="display-checkbox-${index}">
            <label for="display-checkbox-${index}">
                <h3>${contract.name}</h3>
            </label>
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

// Main function to load contracts
export function loadContracts() {
    try {
        log('Loading contracts...', 'loadContracts');

        const contractsDataScript = document.getElementById('contracts-data');
        if (!contractsDataScript) throw new Error('Contracts data script element not found.');

        const contractsData = JSON.parse(contractsDataScript.textContent);
        if (!Array.isArray(contractsData)) throw new Error('Unexpected data format.');

        const container = document.getElementById('display-container');
        container.innerHTML = '';

        if (contractsData.length === 0) {
            const message = document.createElement('h3');
            message.textContent = 'Info: No contracts available.';
            container.appendChild(message);
            log('No contracts available.', 'loadContracts');
            return;
        }

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
                const checkbox = contractElement.querySelector('.display-checkbox');
                if (checkbox) {
                    checkbox.checked = !checkbox.checked;
                    contractElement.classList.toggle('selected', checkbox.checked);
                }
            }
        });

        log('Contracts loaded successfully.', 'loadContracts');
    } catch (err) {
        error(err.message, 'loadContracts');
    }
}
