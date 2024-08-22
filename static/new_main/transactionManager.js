import { formatDate } from './utils.js';

let filteredData = [];
let transactionsData = [];
let hidden_transactions = [];
let sortConfig = { key: 'date', ascending: false };
let dateRange = { start: null, end: null };
let showNoContract = true;

export const loadTransactions = () => {
    try {
        const transactionsDataScript = document.getElementById('transactions-data');
        if (!transactionsDataScript) throw new Error('Transactions data script element not found.');

        transactionsData = JSON.parse(transactionsDataScript.textContent);
        if (!Array.isArray(transactionsData)) throw new Error('Unexpected data format.');

        filteredData = transactionsData;

        const contractFilter = document.getElementById('contract-filter');
        const contractNames = [...new Set(transactionsData.map(t => t.contract?.name).filter(Boolean))];
        contractNames.forEach(name => {
            const option = document.createElement('option');
            option.value = name;
            option.textContent = name;
            contractFilter.appendChild(option);
        });

        let toggleButton = document.getElementById('toggle-hidden-transaction');
        let slider = toggleButton.querySelector('.slider');

        slider.classList.toggle('active');
        slider.classList.toggle('active');

        toggleButton = document.getElementById('toggle-only-contract');
        slider = toggleButton.querySelector('.slider');
        slider.classList.toggle('active');
        slider.classList.toggle('active');

        setupEventListeners();

        filterTransactions();

        document.querySelectorAll('.transaction-row').forEach(row => {
            row.addEventListener('click', (event) => handleRowSelection(event, row));
        });
    } catch (err) {
    }
};

const generateTransactionHTML = ({ transaction, contract }, index) => {
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

const setupEventListeners = () => {
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

const sortColumn = (key) => {
    sortConfig.ascending = (sortConfig.key === key) ? !sortConfig.ascending : true;
    sortConfig.key = key;

    sortData();
    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index, index + 1))
        .join('');

    updateSortIcons();

    document.querySelectorAll('.transaction-row').forEach(row => {
        row.addEventListener('click', (event) => handleRowSelection(event, row));
    });
};

const sortData = () => {
    if (!sortConfig.key) return;

    filteredData.sort((a, b) => {
        const aValue = a.transaction[sortConfig.key];
        const bValue = b.transaction[sortConfig.key];

        if (aValue < bValue) return sortConfig.ascending ? -1 : 1;
        if (aValue > bValue) return sortConfig.ascending ? 1 : -1;
        return 0;
    });
};

const updateSortIcons = () => {
    document.querySelectorAll('.sortable').forEach(header => {
        const icon = header.querySelector('span');

        if (header.dataset.key === sortConfig.key) {
            icon.textContent = sortConfig.ascending ? '↑' : '↓';
        } else {
            icon.textContent = '↑';
        }
    });
};

const handleRowSelection = (event, row) => {
    row.classList.toggle('selected');
};

const filterTransactions = () => {
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
    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index, index + 1))
        .join('');
};

const handleHideOrRemove = (action) => {
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
            if (result.ok) {
                selectedRows.forEach(row => {
                    const transactionIndex = filteredData.findIndex(item => item.transaction.id === parseInt(row.dataset.id));
                    if (transactionIndex !== -1) {
                        document.getElementById('success').style.display = 'block';
                        document.getElementById('error').style.display = 'none';

                        row.classList.remove('selected');

                        if (action === 'hide') {
                            row.classList.add('hidden');

                            document.getElementById('success').innerHTML = 'Transactions have been hidden successfully.';

                            filteredData[transactionIndex].transaction.is_hidden = true;

                            hidden_transactions.push(row.getAttribute('data-id'));

                            const contractRow = row.nextElementSibling;
                            if (contractRow && contractRow.classList.contains('contract-row')) {
                                contractRow.style.display = 'none';
                            }
                        } else if (action === 'remove') {
                            document.getElementById('success').innerHTML = 'Selected transactions have been removed.';

                            filteredData.splice(transactionIndex, 1);

                            const contractRow = row.nextElementSibling;
                            if (contractRow && contractRow.classList.contains('contract-row')) {
                                contractRow.remove();
                            }
                        }
                    }
                });

            } else {
                document.getElementById('error').innerHTML = `Failed to ${action.replace('-', ' ')}: ${result.statusText}`;
                document.getElementById('error').style.display = 'block';
                document.getElementById('success').style.display = 'none';
            }
        })
        .catch(err => console.error('Error:', err));
};

const showHiddenTransactions = () => {
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

const showOnlyTransactionsWithContracts = () => {
    const toggleButton = document.getElementById('toggle-only-contract');
    const slider = toggleButton.querySelector('.slider');

    slider.classList.toggle('active');

    showNoContract = !slider.classList.contains('active');

    filterTransactions();
}