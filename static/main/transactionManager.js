import { formatDate, displayCustomAlert } from './utils.js';
import { error, log } from './main.js';

let filteredData = [];
let transactionsData = [];
let sortConfig = { key: 'date', ascending: false };
let dateRange = { start: null, end: null };
let showOrHideTransaction = false;

export const loadTransactions = () => {
    try {
        const transactionsDataScript = document.getElementById('transactions-data');
        if (!transactionsDataScript) {
            error('No transaction data found.', 'loadTransactions');
        }

        transactionsData = JSON.parse(transactionsDataScript.textContent);
        if (!Array.isArray(transactionsData)) {
            error('Invalid transaction data found.', 'loadTransactions');
            return;
        }

        filteredData = transactionsData;

        fillContractFilter();
        setupToggleButtons();
        setupEventListeners();
        filterTransactions();
    } catch (err) {
    }
};

function fillContractFilter() {
    const contractFilter = document.getElementById('contract-filter');
    const contractNames = [...new Set(transactionsData.map(t => t.contract?.name).filter(Boolean))];
    contractNames.forEach(name => {
        const option = document.createElement('option');
        option.value = name;
        option.textContent = name;
        contractFilter.appendChild(option);
    });
}

function setupToggleButtons() {
    let toggleButton = document.getElementById('toggle-hidden-transaction');
    let slider = toggleButton.querySelector('.slider');

    slider.classList.toggle('active');
    slider.classList.toggle('active');
}

function generateTransactionHTML({ transaction, contract }, index) {
    const amountClass = transaction.amount < 0 ? 'negative' : 'positive';
    const balanceClass = transaction.bank_balance_after < 0 ? 'negative' : 'positive';
    const rowClass = transaction.is_hidden ? 'hidden_transaction' : '';
    let displayStyle = 'table-row';

    if (transaction.is_hidden) {
        displayStyle = showOrHideTransaction ? 'table-row' : 'none';
    }

    let contractName = '';
    let contractAmount = '';
    let monthsBetweenPayment = '';
    let contractEndDate = '';
    let contractAction = '';

    if (contract) {
        const contractAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
        contractName = contract.name;
        contractAmount = `<span class="${contractAmountClass}">$${contract.current_amount.toFixed(2)}</span>`;
        monthsBetweenPayment = contract.months_between_payment;
        contractEndDate = contract.end_date ? formatDate(contract.end_date) : '';
        contractAction = `<button class="remove-contract-btn" data-index="${index}">Remove Contract</button>`;
    } else {
        contractAction = `<button class="add-contract-btn" data-index="${index}">Add Contract</button>`;
    }

    return `
        <tr class="transaction-row ${rowClass}" style="display: ${displayStyle}" data-index="${index}">
            <td>${transaction.counterparty}</td>
            <td class="${amountClass}">$${transaction.amount.toFixed(2)}</td>
            <td class="${balanceClass}">$${transaction.bank_balance_after.toFixed(2)}</td>
            <td>${formatDate(transaction.date)}</td>
            <td>${contractName}</td>
            <td>${contractAmount}</td>
            <td>${monthsBetweenPayment}</td>
            <td>${contractEndDate}</td>
            <td>${contractAction}</td>
        </tr>
    `;
}

function setupEventListeners() {
    document.getElementById('transaction-search').addEventListener('input', filterTransactions);
    document.getElementById('contract-filter').addEventListener('change', filterTransactions);

    document.querySelectorAll('.sortable').forEach(header => {
        header.addEventListener('click', () => sortColumn(header.dataset.key));
    });

    document.getElementById('hide-transaction').addEventListener('click', () => {
        handleHideOrRemove('hide');
    });

    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index, index + 1))
        .join('');

    document.getElementById('toggle-hidden-transaction').addEventListener('click', showHiddenTransactions);

    const tableBody = document.getElementById('transaction-table-body');

    // Handle click events for remove contract buttons
    tableBody.addEventListener('click', (event) => {
        const index = event.target.getAttribute('data-index');
        if (event.target.classList.contains('remove-contract-btn')) {
            removeContract(index);
        } else if (event.target.classList.contains('add-contract-btn')) {
            handleAddContract(event);
        }
    });

    flatpickr("#date-range", {
        mode: "range",  // Enable range selection mode
        dateFormat: "d-m-Y",  // Format the date as Year-Month-Day
        onChange: (selectedDates) => {
            // Set start and end dates when a range is selected
            dateRange.start = selectedDates[0];
            dateRange.end = selectedDates[1];
            filterTransactions();  // Apply filter when dates are selected
        }
    });

    updateTransactionTable();
};

function updateTransactionTable() {
    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index))
        .join('');

    attachRowEventListeners();
}

function attachRowEventListeners() {
    document.querySelectorAll('.transaction-row').forEach(row => {
        row.addEventListener('click', (event) => handleRowSelection(event, row));
    });
}

function sortColumn(key) {
    sortConfig.ascending = (sortConfig.key === key) ? !sortConfig.ascending : true;
    sortConfig.key = key;

    sortData();
    updateSortIcons();
    attachRowEventListeners();
};

function sortData() {
    if (!sortConfig.key) return;

    // Helper function to get the value based on sortConfig.key
    function getValue(item) {
        return item.transaction?.[sortConfig.key] ?? item.contract?.[sortConfig.key];
    }

    filteredData.sort((a, b) => {
        const aValue = getValue(a);
        const bValue = getValue(b);

        // Define the sort direction based on the sortConfig
        const direction = sortConfig.ascending ? 1 : -1;

        // Check if values are undefined and sort them accordingly
        if (aValue === undefined && bValue === undefined) return 0;
        if (aValue === undefined) return direction;
        if (bValue === undefined) return -direction;

        // Handle null values
        if (aValue === null && bValue === null) return 0;
        if (aValue === null) return direction;
        if (bValue === null) return -direction;

        // Compare non-null values
        if (aValue < bValue) return -direction;
        if (aValue > bValue) return direction;
        return 0;
    });

    updateTransactionTable();
}


function updateSortIcons() {
    document.querySelectorAll('.sortable').forEach(header => {
        const icon = header.querySelector('span');

        if (header.dataset.key === sortConfig.key) {
            icon.textContent = sortConfig.ascending ? '↑' : '↓';
        } else {
            icon.textContent = '↑';
        }
    });
};

function handleRowSelection(event, row) {
    row.classList.toggle('selected');
};

function filterTransactions() {
    const searchQuery = document.getElementById('transaction-search').value.toLowerCase();
    const selectedContract = document.getElementById('contract-filter').value;

    filteredData = transactionsData.filter(({ transaction, contract }) => {
        const { counterparty, date, amount } = transaction;
        const contractName = contract?.name || '';
        const formattedDate = formatDate(date);
        const amountString = amount.toFixed(2);

        const transactionDate = new Date(date);
        const withinDateRange = (!dateRange.start || !dateRange.end) ||
            (transactionDate >= dateRange.start && transactionDate <= dateRange.end);

        const matchesSearch = (
            counterparty.toLowerCase().includes(searchQuery) ||
            formattedDate.includes(searchQuery) ||
            amountString.includes(searchQuery) ||
            contractName.toLowerCase().includes(searchQuery)
        );

        const matchesContract = selectedContract ? contractName === selectedContract : true;

        return matchesSearch && matchesContract && withinDateRange;
    });

    sortData();
    updateTransactionTable();
}

function removeContract(index) {
    const transaction = filteredData[index].transaction;

    fetch(`/bank/transaction/remove/${transaction.id}`, {
        method: 'GET',
        headers: {
            'Content-Type': 'application/json',
        },
    })
        .then(async response => {
            if (response.ok) {
                filteredData[index].contract = null;
                transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).contract = null;

                updateTransactionTable();

                log('Contract removed successfully:', 'removeContract', response);

                const json = await response.json();

                if (json.success) {
                    displayCustomAlert('success', json.header, json.success, 'Close');
                } else if (json.error) {
                    displayCustomAlert('error', json.header, json.error, 'Close');
                }
            } else {
                error('Error removing contract:', 'removeContract', response);
                displayCustomAlert('error', 'Error removing contract.', 'An error occurred while trying to remove the contract.', 'Close');
            }
        })
        .catch(err => error('Error while trying to remove contract:', 'removeContract', err));
}

function handleHideOrRemove(action) {
    const selectedRows = document.querySelectorAll('.transaction-row.selected');

    const selectedIds = Array.from(selectedRows).map(row => parseInt(row.dataset.id));

    fetch(`/bank/transaction/${action}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ ids: selectedIds }),
    })
        .then(result => {
            const action_text = action === 'hide' ? 'hidden' : 'removed';

            if (result.ok) {
                selectedRows.forEach(row => {
                    const transactionIndex = filteredData.findIndex(item => item.transaction.id === parseInt(row.dataset.id));
                    if (transactionIndex !== -1) {
                        row.classList.remove('selected');

                        if (action === 'hide') {
                            row.classList.add('hidden_transaction');

                            filteredData[transactionIndex].transaction.is_hidden = true;
                        } else {
                            filteredData.splice(transactionIndex, 1);
                        }
                    }
                });

                const body_text = 'A total of ' + selectedIds.length + ' transactions have been ' + action_text + ' successfully.';
                displayCustomAlert('success', 'Transactions have been hidden successfully.', body_text, 'Close');
            } else {
                const body_text = 'An error occurred while trying to ' + action_text + ' a total of ' + selectedIds.length + ' transactions.';
                displayCustomAlert('error', 'Error ' + action_text + ' transactions.', body_text, 'Close');
            }
        })
        .catch(err => error('Error while trying to ' + action + ' transactions:', 'handleHideOrRemove', err));
};

function showHiddenTransactions() {
    const toggleButton = document.getElementById('toggle-hidden-transaction');
    const slider = toggleButton.querySelector('.slider');

    slider.classList.toggle('active');
    showOrHideTransaction = !showOrHideTransaction;

    const displayStyle = showOrHideTransaction ? 'table-row' : 'none';

    const hiddenTransactions = document.querySelectorAll('.hidden_transaction');

    hiddenTransactions.forEach(transactionRow => {
        if (transactionRow) {
            transactionRow.style.display = displayStyle;
        }
    });
}