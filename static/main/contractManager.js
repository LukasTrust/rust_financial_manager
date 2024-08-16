import { log, error } from './logger.js';

export function loadContracts() {
    log('Loading contracts...', 'loadContracts');
    const contractsDataScript = document.getElementById('contracts-data');
    const contractsData = JSON.parse(contractsDataScript.textContent);

    if (Array.isArray(contractsData)) {
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
            const { contract, contract_history, total_amount_paid } = contractWithHistory;

            const contractElement = document.createElement('div');
            contractElement.className = 'contract';

            contractElement.innerHTML = `
                <h3>${contract.name}</h3>
                <p>Current amount: $${contract.current_amount.toFixed(2)}</p>
                <p>Months between Payment: ${contract.months_between_payment}</p>
                <p>Total amount over time: $${total_amount_paid.toFixed(2)}</p>
                ${contract.end_date ? `<p>End Date: ${contract.end_date}</p>` : ''}
                <button class="toggle-history-btn" data-index="${index}">Show History</button>
                <div id="contract-history-${index}" class="hidden contract-history">
                    <h4>Contract History:</h4>
                    <ul>
                        ${contract_history.length > 0 ? contract_history.map(history => `
                            <li>
                                <p>Old Amount: $${history.old_amount.toFixed(2)}</p>
                                <p>New Amount: $${history.new_amount.toFixed(2)}</p>
                                <p>Changed At: ${history.changed_at || 'N/A'}</p>
                            </li>
                        `).join('') : '<li>No history available.</li>'}
                    </ul>
                </div>
            `;
            container.appendChild(contractElement);
            const toggleHistoryBtn = contractElement.querySelector('.toggle-history-btn');
            const historyElement = document.getElementById(`contract-history-${index}`);
            toggleHistoryBtn.addEventListener('click', () => {
                historyElement.classList.toggle('hidden');
                toggleHistoryBtn.textContent = historyElement.classList.contains('hidden') ? 'Show History' : 'Hide History';
            });
        });

        log('Contracts loaded successfully.', 'loadContracts');
    } else {
        error('Unexpected data format:', 'loadContracts', contractsData);
    }
}
