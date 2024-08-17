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

    return contractHistory.map(history => `
        <li>
            <p>Old Amount: <span class="${history.old_amount < 0 ? 'negative' : 'positive'}">$${history.old_amount.toFixed(2)}</span></p>
            <p>New Amount: <span class="${history.new_amount < 0 ? 'negative' : 'positive'}">$${history.new_amount.toFixed(2)}</span></p>
            <p>Changed At: ${formatDate(history.changed_at) || 'N/A'}</p>
        </li>
    `).join('');
}

// Generate HTML for a single contract
function generateContractHTML(contractWithHistory, index) {
    const { contract, contract_history, total_amount_paid, last_payment_date } = contractWithHistory;

    const currentAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
    const totalAmountClass = total_amount_paid < 0 ? 'negative' : 'positive';

    // Check if the end_date exists, otherwise use the last_payment_date
    const dateLabel = contract.end_date ? 'End date' : 'Last payment date';
    const dateValue = contract.end_date ? formatDate(contract.end_date) : formatDate(last_payment_date);
    const dateClass = contract.end_date ? 'negative' : '';  // Only use 'negative' for end_date

    return `
        <div class="contract">
            <h3>${contract.name}</h3>
            <p>Current amount: <span class="${currentAmountClass}">$${contract.current_amount.toFixed(2)}</span></p>
            <p>Total amount over time: <span class="${totalAmountClass}">$${total_amount_paid.toFixed(2)}</span></p>
            <p>Months between Payment: ${contract.months_between_payment}</p>
            <p>${dateLabel}: <span class="${dateClass}">${dateValue}</span></p>
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

        const container = document.getElementById('contracts-container');
        container.innerHTML = '';

        if (contractsData.length === 0) {
            const message = document.createElement('h3');
            message.textContent = 'Info: No contracts available.';
            container.appendChild(message);
            log('No contracts available.', 'loadContracts');
            return;
        }

        contractsData.forEach((contractWithHistory, index) => {
            const contractHTML = generateContractHTML(contractWithHistory, index);
            container.insertAdjacentHTML('beforeend', contractHTML);

            const toggleHistoryBtn = container.querySelector(`.toggle-history-btn[data-index="${index}"]`);
            const historyElement = document.getElementById(`contract-history-${index}`);

            toggleHistoryBtn.setAttribute('aria-expanded', 'false');
            toggleHistoryBtn.addEventListener('click', () => {
                const isHidden = historyElement.classList.toggle('hidden');
                toggleHistoryBtn.textContent = isHidden ? 'Show History' : 'Hide History';
                toggleHistoryBtn.setAttribute('aria-expanded', isHidden ? 'false' : 'true');
            });
        });

        log('Contracts loaded successfully.', 'loadContracts');
    } catch (err) {
        error(err.message, 'loadContracts');
    }
}
