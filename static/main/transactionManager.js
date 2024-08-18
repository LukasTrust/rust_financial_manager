import { log, error } from './logger.js';

let lastSelectedRowIndex = null;
let virtualizedStartIndex = 0;
const PAGE_SIZE = 20;

function formatDate(dateString) {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'N/A';
    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();
    return `${day}.${month}.${year}`;
}

function generateTransactionHTML(transactionWithContract, index) {
    const { transaction, contract } = transactionWithContract;
    const amountClass = transaction.amount < 0 ? 'negative' : 'positive';
    const balanceClass = transaction.bank_balance_after < 0 ? 'negative' : 'positive';
    const transactionRow = `
        <tr class="transaction-row" data-index="${index}">
            <td>${transaction.counterparty}</td>
            <td class="${amountClass}">$${transaction.amount.toFixed(2)}</td>
            <td class="${balanceClass}">$${transaction.bank_balance_after.toFixed(2)}</td>
            <td>${formatDate(transaction.date)}</td>
        </tr>
    `;
    let contractRow = '';
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
    return transactionRow + contractRow;
}

function setupEventListeners(transactionsData) {
    document.getElementById('transaction-search').addEventListener('input', () => filterTransactions(transactionsData));
    document.getElementById('contract-filter').addEventListener('change', () => filterTransactions(transactionsData));
    document.getElementById('no-contract-filter').addEventListener('change', () => filterTransactions(transactionsData));
}

export function loadTransactions() {
    try {
        log('Loading transactions...', 'loadTransactions');
        const transactionsDataScript = document.getElementById('transactions-data');
        if (!transactionsDataScript) throw new Error('Transactions data script element not found.');
        let transactionsData = JSON.parse(transactionsDataScript.textContent);
        if (!Array.isArray(transactionsData)) throw new Error('Unexpected data format.');
        const container = document.getElementById('display-container');
        container.innerHTML = '';

        if (transactionsData.length === 0) {
            container.innerHTML = '<h3>Info: No transactions available.</h3>';
            log('No transactions available.', 'loadTransactions');
            return;
        }

        const headerContainer = document.createElement('div');
        headerContainer.classList.add('container-without-border-horizontally');
        container.appendChild(headerContainer);

        const dateRangeSelector = `
            <label for="date-range">Select date range:</label>
            <input type="text" id="date-range" class="flatpickr">
        `;
        headerContainer.insertAdjacentHTML('beforeend', dateRangeSelector);
        flatpickr("#date-range", {
            mode: "range",
            dateFormat: "Y-m-d",
            onChange: () => filterByDateRange(transactionsData),
        });

        const contractNames = [...new Set(transactionsData.map(t => t.contract?.name).filter(Boolean))];
        const contractFilter = `
            <label for="contract-filter">Filter by Contract:</label>
            <select style="max-width: 300px" id="contract-filter">
                <option value="">All Contracts</option>
                ${contractNames.map(name => `<option value="${name}">${name}</option>`).join('')}
            </select>
        `;
        headerContainer.insertAdjacentHTML('beforeend', contractFilter);

        const noContractFilter = `
            <label>
                <input type="checkbox" id="no-contract-filter"> Show only transactions without contracts
            </label>
        `;
        headerContainer.insertAdjacentHTML('beforeend', noContractFilter);

        const searchInput = `
            <input style="width: auto; height: 15px" type="text" id="transaction-search" placeholder="Search transactions...">
        `;
        headerContainer.insertAdjacentHTML('beforeend', searchInput);

        const table = `
            <table class="transaction-table">
                <thead>
                    <tr>
                        <th>Counterparty</th>
                        <th>Amount</th>
                        <th>Bank Balance After</th>
                        <th>Date</th>
                    </tr>
                </thead>
                <tbody id="transaction-table-body">
                    ${renderVirtualizedRows(transactionsData, virtualizedStartIndex, PAGE_SIZE)}
                </tbody>
            </table>
        `;
        container.insertAdjacentHTML('beforeend', table);

        setupEventListeners(transactionsData);

        document.querySelectorAll('.transaction-row').forEach(row => {
            row.addEventListener('click', (event) => handleRowSelection(event, row));
        });

        let debounceTimeout;
        container.addEventListener('scroll', () => {
            if (debounceTimeout) clearTimeout(debounceTimeout);
            debounceTimeout = setTimeout(() => {
                const scrollHeight = container.scrollHeight;
                const scrollTop = container.scrollTop;
                const clientHeight = container.clientHeight;
                if (scrollTop + clientHeight >= scrollHeight - 10) {
                    virtualizedStartIndex += PAGE_SIZE;
                    renderTransactions(transactionsData);
                }
            }, 100);
        });

        log('Transactions loaded successfully.', 'loadTransactions');
    } catch (err) {
        error(err.message, 'loadTransactions');
    }
}

function renderVirtualizedRows(transactions, startIndex, pageSize) {
    return transactions.slice(startIndex, startIndex + pageSize)
        .map(generateTransactionHTML)
        .join('');
}

function renderTransactions(transactions) {
    const tableBody = document.getElementById('transaction-table-body');
    tableBody.innerHTML += renderVirtualizedRows(transactions, virtualizedStartIndex, PAGE_SIZE);
    document.querySelectorAll('.transaction-row').forEach(row => {
        row.addEventListener('click', (event) => handleRowSelection(event, row));
    });
}

function handleRowSelection(event, row) {
    const index = parseInt(row.dataset.index, 10);
    if (event.shiftKey && lastSelectedRowIndex !== null) {
        const start = Math.min(index, lastSelectedRowIndex);
        const end = Math.max(index, lastSelectedRowIndex);
        document.querySelectorAll('.transaction-row').forEach((r, i) => {
            if (i >= start && i <= end) {
                r.classList.add('selected');
            }
        });
    } else {
        row.classList.toggle('selected');
        lastSelectedRowIndex = index;
    }
}

function filterTransactions(transactions) {
    const searchQuery = document.getElementById('transaction-search').value.toLowerCase();
    const selectedContract = document.getElementById('contract-filter').value;
    const showNoContract = document.getElementById('no-contract-filter').checked;

    const filteredTransactions = transactions.filter(({ transaction, contract }) => {
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

    virtualizedStartIndex = 0;
    document.getElementById('transaction-table-body').innerHTML = '';
    renderTransactions(filteredTransactions);
}

function filterByDateRange(transactions) {
    const dateRange = document.getElementById('date-range').value.split(' to ');
    const startDate = new Date(dateRange[0]);
    const endDate = new Date(dateRange[1]);

    if (isNaN(startDate.getTime()) || isNaN(endDate.getTime()) || startDate > endDate) {
        // Handle invalid date range
        return;
    }

    const filteredTransactions = transactions.filter(({ transaction }) => {
        const transactionDate = new Date(transaction.date);
        return transactionDate >= startDate && transactionDate <= endDate;
    });

    virtualizedStartIndex = 0;
    document.getElementById('transaction-table-body').innerHTML = '';
    renderTransactions(filteredTransactions);
}
