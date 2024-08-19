import { log, error } from './logger.js';

let filteredData = [];
let transactionsData = [];
let hidden_transactions = [];
let sortConfig = { key: null, ascending: true };

const formatDate = (dateString) => {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'N/A';
    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();
    return `${day}.${month}.${year}`;
};

const generateTransactionHTML = ({ transaction, contract }, index) => {
    const amountClass = transaction.amount < 0 ? 'negative' : 'positive';
    const balanceClass = transaction.bank_balance_after < 0 ? 'negative' : 'positive';
    let contractRow = '';

    if (transaction.is_hidden) {
        hidden_transactions.push(transaction.id);
    }

    if (contract) {
        const contractAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
        contractRow = `
            <tr class="contract-row">
                <td colspan="5">
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
        <tr class="transaction-row" data-index="${index}" data-id="${transaction.id}">
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
    document.getElementById('no-contract-filter').addEventListener('change', filterTransactions);
    document.querySelectorAll('.sortable').forEach(header => {
        header.addEventListener('click', () => sortColumn(header.dataset.key));
    });
};

export const loadTransactions = () => {
    try {
        log('Loading transactions...', 'loadTransactions');

        const transactionsDataScript = document.getElementById('transactions-data');
        if (!transactionsDataScript) throw new Error('Transactions data script element not found.');

        transactionsData = JSON.parse(transactionsDataScript.textContent);
        if (!Array.isArray(transactionsData)) throw new Error('Unexpected data format.');

        filteredData = transactionsData;

        // Populate contract filter dropdown
        const contractFilter = document.getElementById('contract-filter');
        const contractNames = [...new Set(transactionsData.map(t => t.contract?.name).filter(Boolean))];
        contractNames.forEach(name => {
            const option = document.createElement('option');
            option.value = name;
            option.textContent = name;
            contractFilter.appendChild(option);
        });

        // Populate the table with all transactions at once
        document.getElementById('transaction-table-body').innerHTML = filteredData
            .map((item, index) => generateTransactionHTML(item, index, index + 1))
            .join('');

        setupEventListeners();

        // Add event listeners for rows
        document.querySelectorAll('.transaction-row').forEach(row => {
            row.addEventListener('click', (event) => handleRowSelection(event, row));
        });

        document.getElementById('remove-transaction').addEventListener('click', () => {
            handleButtonClick('remove');
        });

        document.getElementById('hide-transaction').addEventListener('click', () => {
            handleButtonClick('hide');
        });

        log('Transactions loaded successfully.', 'loadTransactions');
    } catch (err) {
        error(err.message, 'loadTransactions');
    }
};

const sortColumn = (key) => {
    sortConfig.ascending = (sortConfig.key === key) ? !sortConfig.ascending : true;
    sortConfig.key = key;

    sortData();
    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index, index + 1))
        .join('');

    updateSortIcons();
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
    const showNoContract = document.getElementById('no-contract-filter').checked;

    filteredData = transactionsData.filter(({ transaction, contract }) => {
        const { counterparty, date, amount } = transaction;
        const contractName = contract?.name || '';
        const formattedDate = formatDate(date);
        const amountString = amount.toFixed(2);

        const matchesSearch = (
            counterparty.toLowerCase().includes(searchQuery) ||
            formattedDate.includes(searchQuery) ||
            amountString.includes(searchQuery) ||
            contractName.toLowerCase().includes(searchQuery)
        );

        const matchesContract = selectedContract ? contractName === selectedContract : true;
        const matchesNoContract = showNoContract ? !contract : true;

        return matchesSearch && matchesContract && matchesNoContract;
    });

    sortData();
    document.getElementById('transaction-table-body').innerHTML = filteredData
        .map((item, index) => generateTransactionHTML(item, index, index + 1))
        .join('');
};

const handleButtonClick = (action) => {
    const selectedRows = document.querySelectorAll('.transaction-row.selected');

    // Convert the dataset IDs to integers
    const selectedIds = Array.from(selectedRows).map(row => parseInt(row.dataset.id));

    fetch(`bank/transaction/${action}`, {
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
                            document.getElementById('success').innerHTML = `Transactions have been hidden action was successful`;

                            // Mark the transaction as hidden and hide the row
                            filteredData[transactionIndex].transaction.is_hidden = true;
                            row.style.display = 'none';

                            // Hide the associated contract row if it exists
                            const contractRow = row.nextElementSibling;
                            if (contractRow && contractRow.classList.contains('contract-row')) {
                                contractRow.style.display = 'none';
                            }
                        } else if (action === 'remove') {
                            document.getElementById('success').innerHTML = `Contracts of the selected transactions have been removed`;

                            // Remove the transaction and its contract row from data and DOM
                            filteredData.splice(transactionIndex, 1);

                            const contractRow = row.nextElementSibling;
                            if (contractRow && contractRow.classList.contains('contract-row')) {
                                contractRow.remove();
                            }
                        }
                    }
                });

            } else if (result.Err) {
                document.getElementById('error').innerHTML = `Failed to ${action.replace('-', ' ')}: ${result.Err}`;
                document.getElementById('error').style.display = 'block';
                document.getElementById('success').style.display = 'none';
            }
        })
        .catch(error => console.error('Error:', error));
};