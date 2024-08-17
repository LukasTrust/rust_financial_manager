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

    const dateLabel = contract.end_date ? 'End date' : 'Last payment date';
    const dateValue = contract.end_date ? formatDate(contract.end_date) : formatDate(last_payment_date);

    return `
        <div class="contract">
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

// Group contracts by bank
function groupContractsByBank(contracts) {
    return contracts.reduce((groups, contract) => {
        const bank = contract.bank;
        if (!groups[bank]) {
            groups[bank] = [];
        }
        groups[bank].push(contract);
        return groups;
    }, {});
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

        const groupedContracts = groupContractsByBank(contractsData);

        let index = 0;

        Object.keys(groupedContracts).forEach(bank => {
            const bankContracts = groupedContracts[bank];

            const bankSection = document.createElement('div');
            bankSection.classList.add('container-without-border');

            const bankTitle = document.createElement('h2');
            bankTitle.textContent = `Bank: ${bank}`;
            bankSection.appendChild(bankTitle);

            const openContractsWrapper = document.createElement('div');
            openContractsWrapper.classList.add('contracts-container');
            const closedContractsWrapper = document.createElement('div');
            closedContractsWrapper.classList.add('contracts-container');

            bankContracts.forEach(contractWithHistory => {
                const contractHTML = generateContractHTML(contractWithHistory, index);

                if (contractWithHistory.contract.end_date) {
                    closedContractsWrapper.insertAdjacentHTML('beforeend', contractHTML);
                } else {
                    openContractsWrapper.insertAdjacentHTML('beforeend', contractHTML);
                }
                index++;
            });

            const openContractsTitle = document.createElement('h3');
            openContractsTitle.textContent = 'Open Contracts';
            bankSection.appendChild(openContractsTitle);

            bankSection.appendChild(openContractsWrapper);


            const closedContractsTitle = document.createElement('h3');
            closedContractsTitle.textContent = 'Closed Contracts';
            bankSection.appendChild(closedContractsTitle);

            bankSection.appendChild(closedContractsWrapper);

            container.appendChild(bankSection);
        });

        // Delegate the event listener to the container
        container.addEventListener('click', (event) => {
            const toggleHistoryBtn = event.target.closest('.toggle-history-btn');
            if (toggleHistoryBtn) {
                const index = toggleHistoryBtn.getAttribute('data-index');
                const historyElement = document.getElementById(`contract-history-${index}`);
                const isHidden = historyElement.classList.toggle('hidden');
                toggleHistoryBtn.textContent = isHidden ? 'Show History' : 'Hide History';
                toggleHistoryBtn.setAttribute('aria-expanded', isHidden ? 'false' : 'true');
            }
        });

        log('Contracts loaded successfully.', 'loadContracts');
    } catch (err) {
        error(err.message, 'loadContracts');
    }
}
