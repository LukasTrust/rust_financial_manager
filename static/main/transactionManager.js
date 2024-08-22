import { formatDate, displayCustomAlert } from './utils.js';
import { error } from './main.js';

let filteredData = [];
let transactionsData = [];
let hidden_transactions = [];
let sortConfig = { key: 'date', ascending: false };
let dateRange = { start: null, end: null };
let showNoContract = true;
let showHiddenTransaction = true;

export const loadTransactions = () => {
    try {
        const transactionsDataScript = document.getElementById('transactions-data');
        if (!transactionsDataScript) {
            error('No transaction data found.', 'loadTransactions');
        }

        transactionsData = JSON.parse(transactionsDataScript.textContent);
        if (!Array.isArray(transactionsData)) {
            error('Invalid transaction data found.', 'loadTransactions');
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

    toggleButton = document.getElementById('toggle-only-contract');
    slider = toggleButton.querySelector('.slider');
    slider.classList.toggle('active');
    slider.classList.toggle('active');
}

function generateTransactionHTML({ transaction, contract }, index) {
    const amountClass = transaction.amount < 0 ? 'negative' : 'positive';
    const balanceClass = transaction.bank_balance_after < 0 ? 'negative' : 'positive';
    let contractRow = '';

    if (transaction.is_hidden) {
        hidden_transactions.push(transaction.id);
    }

    const rowClass = transaction.is_hidden ? 'hidden_transaction' : '';

    if (contract) {
        const contractAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
        contractRow = `
            <tr class="contract-row">
                <td colspan="4">
                    <div class="contract-details">
                        <p>Contract Name: ${contract.name}</p>
                        <p>Contract Current Amount: <span class="${contractAmountClass}">$${contract.current_amount.toFixed(2)}</span></p>
                        <p>Months Between Payment: ${contract.months_between_payment}</p>
                        <p>Contract End Date: ${contract.end_date ? formatDate(contract.end_date) : 'N/A'}</p>
                    </div>
                </td>
            </tr>
        `;
    }

    return `
        <tr class="transaction-row ${rowClass}" data-index="${index}" data-id="${transaction.id}">
            <td>${transaction.counterparty}</td>
            <td class="${amountClass}">$${transaction.amount.toFixed(2)}</td>
            <td class="${balanceClass}">$${transaction.bank_balance_after.toFixed(2)}</td>
            <td>${formatDate(transaction.date)}</td>
        </tr>
        ${contractRow}
    `;
};

function setupEventListeners() {
    document.getElementById('transaction-search').addEventListener('input', filterTransactions);
    document.getElementById('contract-filter').addEventListener('change', filterTransactions);

    document.querySelectorAll('.sortable').forEach(header => {
        header.addEventListener('click', () => sortColumn(header.dataset.key));
    });

    document.getElementById('remove-transaction').addEventListener('click', () => {
        handleHideOrRemove('remove');
    });

    document.getElementById('hide-transaction').addEventListener('click', () => {
        handleHideOrRemove('hide');
    });

    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index, index + 1))
        .join('');

    document.getElementById('toggle-only-contract').addEventListener('click', showOnlyTransactionsWithContracts);

    document.getElementById('toggle-hidden-transaction').addEventListener('click', showHiddenTransactions);

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
};

function attachRowEventListeners() {
    const rows = document.querySelectorAll('.transaction-row');

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

    filteredData.sort((a, b) => {
        const aValue = a.transaction[sortConfig.key];
        const bValue = b.transaction[sortConfig.key];

        if (aValue < bValue) return sortConfig.ascending ? -1 : 1;
        if (aValue > bValue) return sortConfig.ascending ? 1 : -1;
        return 0;
    });

    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index, index + 1))
        .join('');
};

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

        // Check if transaction date is within the selected date range
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
        const matchesNoContract = showNoContract ? !contract : true;

        return matchesSearch && matchesContract && matchesNoContract && withinDateRange;
    });

    sortData();

    attachRowEventListeners();
};

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

                            hidden_transactions.push(row.getAttribute('data-id'));

                            const contractRow = row.nextElementSibling;
                            if (contractRow && contractRow.classList.contains('contract-row')) {
                                contractRow.style.display = 'none';
                            }
                        } else {
                            filteredData.splice(transactionIndex, 1);

                            const contractRow = row.nextElementSibling;
                            if (contractRow && contractRow.classList.contains('contract-row')) {
                                contractRow.remove();
                            }
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

    const state = slider.classList.contains('active');

    const displayStyle = state ? 'table-row' : 'none';

    hidden_transactions.forEach(id => {
        const row = document.querySelector(`.transaction-row[data-id="${id}"]`);
        if (row) {
            row.style.display = displayStyle;
        }
    });
};

function showOnlyTransactionsWithContracts() {
    const toggleButton = document.getElementById('toggle-only-contract');
    const slider = toggleButton.querySelector('.slider');

    slider.classList.toggle('active');

    showNoContract = !slider.classList.contains('active');

    filterTransactions();
}