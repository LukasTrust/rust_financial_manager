import { log, error } from './logger.js';

let virtualizedStartIndex = 0;
const PAGE_SIZE = 30;
const INITIAL_ROWS = 30;
let isLoading = false;
let filteredData = [];
let transactionsData = [];
let sortConfig = { key: null, ascending: true };

const formatDate = (dateString) => {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'N/A';
    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();
    return `${day}.${month}.${year}`;
};

const generateTransactionHTML = ({ transaction, contract }, index, rowNumber) => {
    const amountClass = transaction.amount < 0 ? 'negative' : 'positive';
    const balanceClass = transaction.bank_balance_after < 0 ? 'negative' : 'positive';
    let contractRow = '';

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
            <td>${rowNumber}</td>
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

        headerContainer.innerHTML = `
            <label for="date-range">Select date range:</label>
            <input type="text" id="date-range" class="flatpickr">
            <label for="contract-filter">Filter by Contract:</label>
            <select style="max-width: 300px" id="contract-filter">
                <option value="">All Contracts</option>
                ${[...new Set(transactionsData.map(t => t.contract?.name).filter(Boolean))].map(name => `<option value="${name}">${name}</option>`).join('')}
            </select>
            <label>
                <input type="checkbox" id="no-contract-filter"> Show only transactions without contracts
            </label>
            <input style="width: auto; height: 15px" type="text" id="transaction-search" placeholder="Search transactions...">
        `;

        flatpickr("#date-range", {
            mode: "range",
            dateFormat: "Y-m-d",
            onChange: filterByDateRange,
        });

        container.insertAdjacentHTML('beforeend', `
            <table class="transaction-table">
                <thead>
                    <tr>
                        <th>Row</th>
                        <th data-key="counterparty" class="sortable">Counterparty <span>↑</span></th>
                        <th data-key="amount" class="sortable">Amount <span>↑</span></th>
                        <th data-key="bank_balance_after" class="sortable">Bank Balance After <span>↑</span></th>
                        <th data-key="date" class="sortable">Date <span>↑</span></th>
                    </tr>
                </thead>
                <tbody id="transaction-table-body">
                    ${renderVirtualizedRows(filteredData, 0, INITIAL_ROWS)}
                </tbody>
            </table>
        `);

        setupEventListeners();

        document.querySelectorAll('.transaction-row').forEach(row => {
            row.addEventListener('click', (event) => handleRowSelection(event, row));
        });

        const scrollContainer = document.querySelector('.scroll-container');
        const debounce = (func, delay) => {
            let timeout;
            return (...args) => {
                clearTimeout(timeout);
                timeout = setTimeout(() => func.apply(this, args), delay);
            };
        };

        scrollContainer.addEventListener('scroll', debounce(() => {
            const { scrollHeight, scrollTop, clientHeight } = scrollContainer;

            if (scrollTop + clientHeight >= scrollHeight - 10 && !isLoading) {
                if (filteredData.length > virtualizedStartIndex + PAGE_SIZE) {
                    isLoading = true;
                    virtualizedStartIndex += PAGE_SIZE;
                    loadMoreRows(filteredData);
                }
            } else if (scrollTop === 0 && !isLoading) {
                if (virtualizedStartIndex > INITIAL_ROWS) {
                    isLoading = true;
                    virtualizedStartIndex = INITIAL_ROWS;
                    refreshInitialRows(filteredData);
                }
            }
        }, 100));

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

const renderVirtualizedRows = (transactions, startIndex, pageSize) => {
    const rowsHTML = transactions.slice(startIndex, startIndex + pageSize)
        .map((item, index) => generateTransactionHTML(item, startIndex + index, startIndex + index + 1))
        .join('');
    log(`Rendered rows from ${startIndex + 1} to ${startIndex + rowsHTML.split('</tr>').length - 1}`, 'renderVirtualizedRows');
    return rowsHTML;
};

const loadMoreRows = (transactions) => {
    const tableBody = document.getElementById('transaction-table-body');
    const newRowsHTML = renderVirtualizedRows(transactions, virtualizedStartIndex, PAGE_SIZE);

    if (newRowsHTML.trim() === '') {
        isLoading = false;
        return;
    }

    tableBody.insertAdjacentHTML('beforeend', newRowsHTML);
    document.querySelectorAll('.transaction-row').forEach(row => {
        row.addEventListener('click', (event) => handleRowSelection(event, row));
    });
    isLoading = false;
    log(`Loaded ${PAGE_SIZE} rows, total rows now: ${tableBody.children.length}`, 'loadMoreRows');
};

const refreshInitialRows = (transactions) => {
    const tableBody = document.getElementById('transaction-table-body');
    const initialRowsHTML = renderVirtualizedRows(transactions, 0, INITIAL_ROWS);

    tableBody.innerHTML = initialRowsHTML;
    document.querySelectorAll('.transaction-row').forEach(row => {
        row.addEventListener('click', (event) => handleRowSelection(event, row));
    });
    isLoading = false;
    log(`Refreshed to initial ${INITIAL_ROWS} rows, total rows now: ${tableBody.children.length}`, 'refreshInitialRows');
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
    virtualizedStartIndex = 0;
    document.getElementById('transaction-table-body').innerHTML = '';
    loadMoreRows(filteredData);
};

const sortColumn = (key) => {
    if (key === 'rowNumber') return; // Exclude sorting for the "Row" column

    sortConfig.ascending = (sortConfig.key === key) ? !sortConfig.ascending : true;
    sortConfig.key = key;

    sortData();
    virtualizedStartIndex = 0;
    document.getElementById('transaction-table-body').innerHTML = '';
    loadMoreRows(filteredData);

    updateSortIcons();
};

const sortData = () => {
    if (!sortConfig.key) return;

    filteredData.sort((a, b) => {
        const aValue = sortConfig.key === 'rowNumber' ? filteredData.indexOf(a) : a.transaction[sortConfig.key];
        const bValue = sortConfig.key === 'rowNumber' ? filteredData.indexOf(b) : b.transaction[sortConfig.key];

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
            icon.textContent = '↑'; // Default icon for unsorted columns
        }
    });
};

const filterByDateRange = () => {
    const dateRange = document.getElementById('date-range').value.split(' to ');
    const startDate = new Date(dateRange[0]);
    const endDate = new Date(dateRange[1]);

    if (isNaN(startDate.getTime()) || isNaN(endDate.getTime()) || startDate > endDate) {
        return;
    }

    filteredData = transactionsData.filter(({ transaction }) => {
        const transactionDate = new Date(transaction.date);
        return transactionDate >= startDate && transactionDate <= endDate;
    });

    sortData();
    virtualizedStartIndex = 0;
    document.getElementById('transaction-table-body').innerHTML = '';
    loadMoreRows(filteredData);
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