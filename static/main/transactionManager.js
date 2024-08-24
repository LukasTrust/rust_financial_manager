import { formatDate, displayCustomAlert } from './utils.js';
import { error, log } from './main.js';

let filteredData = [];
let transactionsData = [];
let sortConfig = { key: 'date', ascending: false };
let dateRange = { start: null, end: null };
let showOrHideTransaction = false;
let contracts = [];

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

        const contract = transactionsData.find(t => t.contract?.name === name).contract;

        contracts.push(contract);
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

    let contractAllowed = transaction.contract_not_allowed ?
        `<button class="table_button allow-contract-btn" data-index="${index}">Allow Contract</button>` :
        `<button class="table_button not-allow-contract" data-index="${index}">Not Allow Contract</button>`;

    let contractName = '';
    let contractAmount = '';
    let dropdownMenu = '';

    if (contract) {
        const contractAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
        contractName = contract.name;
        contractAmount = `<span class="${contractAmountClass}">$${contract.current_amount.toFixed(2)}</span>`;
        dropdownMenu = `
            <div class="dropdown-content" style="display:none;">
                <button class="table_button remove-contract-btn" data-index="${index}">Remove Contract</button>
                ${contractAllowed}
                <button class="table_button hide-btn" data-index="${index}">${transaction.is_hidden ? 'Display' : 'Hide'}</button>
            </div>`;
    } else {
        dropdownMenu = `
            <div class="dropdown-content" style="display:none;">
                <button class="table_button add-contract-btn" data-index="${index}">Add Contract</button>
                ${contractAllowed}
                <button class="table_button hide-btn" data-index="${index}">${transaction.is_hidden ? 'Display' : 'Hide'}</button>
            </div>`;
    }

    let emptyCellIcon = contract ? '📄' : '';

    if (emptyCellIcon === '' && transaction.contract_not_allowed) {
        emptyCellIcon = '🚫';
    }

    return `
        <tr class="transaction-row ${rowClass}" style="display: ${displayStyle}" data-index="${index}">
            <td>
                <div class="dropdown">
                    ${emptyCellIcon}
                    ${dropdownMenu}
                </div>
            </td>
            <td>${transaction.counterparty}</td>
            <td class="${amountClass}">$${transaction.amount.toFixed(2)}</td>
            <td class="${balanceClass}">$${transaction.bank_balance_after.toFixed(2)}</td>
            <td>${formatDate(transaction.date)}</td>
            <td>${contractName}</td>
            <td>${contractAmount}</td>
        </tr>
    `;
}


function setupEventListeners() {
    document.getElementById('transaction-search').addEventListener('input', filterTransactions);
    document.getElementById('contract-filter').addEventListener('change', filterTransactions);

    document.querySelectorAll('.sortable').forEach(header => {
        header.addEventListener('click', () => sortColumn(header.dataset.key));
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
            handleAddContract(index);
        } else if (event.target.classList.contains('hide-btn')) {
            handleHideTransaction(index);
        } else if (event.target.classList.contains('show-btn')) {
            handleShowTransaction(index);
        } else if (event.target.classList.contains('allow-contract-btn')) {
            handleAllowContract(index);
        } else if (event.target.classList.contains('not-allow-contract')) {
            handleNotAllowContract(index);
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

    const rows = document.querySelectorAll('.transaction-row');
    let activeRow = null;  // Track the currently active row

    rows.forEach(row => {
        row.addEventListener('click', function () {
            // Close the currently active dropdown if another row is clicked
            if (activeRow && activeRow !== row) {
                activeRow.classList.remove('selected-row');
                activeRow.querySelector('.dropdown-content').style.display = 'none';
            }

            // Toggle the clicked row's dropdown
            const dropdownContent = row.querySelector('.dropdown-content');
            if (dropdownContent.style.display === 'none' || !dropdownContent.style.display) {
                dropdownContent.style.display = 'flex';
                row.classList.add('selected-row');
                activeRow = row;  // Set the active row to the current one
            } else {
                dropdownContent.style.display = 'none';
                row.classList.remove('selected-row');
                activeRow = null;  // Reset active row when closed
            }
        });
    });

    // Close dropdown if clicked outside
    document.addEventListener('click', function (event) {
        if (!event.target.closest('.transaction-row')) {
            rows.forEach(row => {
                row.classList.remove('selected-row');
                row.querySelector('.dropdown-content').style.display = 'none';
            });
            activeRow = null;
        }
    });
}

function sortColumn(key) {
    sortConfig.ascending = (sortConfig.key === key) ? !sortConfig.ascending : true;
    sortConfig.key = key;

    sortData();
    updateSortIcons();
};

function sortData() {
    if (!sortConfig.key) return;

    // Helper function to get the value based on sortConfig.key
    function getValue(item) {
        if (sortConfig.key === 'icon') {
            // Define how to evaluate the icon
            let iconValue = '';
            if (item.contract) {
                iconValue = '📄';
            } else if (item.transaction.contract_not_allowed) {
                iconValue = '🚫';
            }
            return iconValue;
        }

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

    const noVisableTransaction = filteredData.find(item => !item.transaction.is_hidden);

    if (!noVisableTransaction && !showOrHideTransaction) {
        showHiddenTransactions();
    }

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
}
// Generalized function to handle transaction operations
function handleTransactionOperation(url, errorMessage) {
    return fetch(url, {
        method: 'GET',
        headers: {
            'Content-Type': 'application/json',
        },
    })
        .then(async response => {
            if (response.ok) {
                const json = await response.json();

                if (json.success) {
                    displayCustomAlert('success', json.header, json.success, 'Close');
                    return true;
                } else if (json.error) {
                    displayCustomAlert('error', json.header, json.error, 'Close');
                    return false;
                }

                return false;
            } else {
                error(`Error ${errorMessage}:`, url, response);
                displayCustomAlert('error', `Error ${errorMessage}.`, `An error occurred while trying to ${errorMessage}.`, 'Close');
                return false;
            }
        })
        .catch(err => {
            error(`Error while trying to ${errorMessage}:`, url, err);
            return false;
        });
}

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

// Row button handlers
function removeContract(index) {
    handleTransactionOperation(
        `/bank/transaction/remove_contract/${filteredData[index].transaction.id}`,
        'remove contract'
    ).then(success => {
        if (success) {
            // Update data
            filteredData[index].contract = null;
            transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).contract = null;

            // Update UI
            const row = document.querySelector(`.transaction-row[data-index="${index}"]`);
            const contractName = row.cells[5];
            const contractAmount = row.cells[6];
            const contractActionButton = row.querySelector('.remove-contract-btn');

            // Clear contract information
            contractName.innerHTML = '';
            contractAmount.innerHTML = '';
            contractActionButton.classList.remove('remove-contract-btn');
            contractActionButton.classList.add('add-contract-btn');
            contractActionButton.textContent = 'Add Contract';
        }
    });
}

function handleHideTransaction(index) {
    handleTransactionOperation(
        `/bank/transaction/hide/${filteredData[index].transaction.id}`,
        'hide transaction'
    ).then(success => {
        if (success) {
            // Update data
            filteredData[index].transaction.is_hidden = true;
            transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).transaction.is_hidden = true;

            // Update UI
            const row = document.querySelector(`.transaction-row[data-index="${index}"]`);
            row.style.display = showOrHideTransaction ? 'table-row' : 'none';
            row.classList.add('hidden_transaction');

            const hideButton = row.querySelector('.hide-btn');
            hideButton.classList.remove('hide-btn');
            hideButton.classList.add('show-btn');
            hideButton.textContent = 'Display';
        }
    });
}

function handleShowTransaction(index) {
    handleTransactionOperation(
        `/bank/transaction/show/${filteredData[index].transaction.id}`,
        'show transaction'
    ).then(success => {
        if (success) {
            // Update data
            filteredData[index].transaction.is_hidden = false;
            transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).transaction.is_hidden = false;

            // Update UI
            const row = document.querySelector(`.transaction-row[data-index="${index}"]`);
            row.style.display = 'table-row';
            row.classList.remove('hidden_transaction');

            const showButton = row.querySelector('.show-btn');
            showButton.classList.remove('show-btn');
            showButton.classList.add('hide-btn');
            showButton.textContent = 'Hide';
        }
    });
}

function handleAllowContract(index) {
    handleTransactionOperation(
        `/bank/transaction/allow_contract/${filteredData[index].transaction.id}`,
        'allow contract'
    ).then(success => {
        if (success) {
            // Update data
            filteredData[index].transaction.contract_not_allowed = false;
            transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).transaction.contract_not_allowed = false;

            // Update UI
            const row = document.querySelector(`.transaction-row[data-index="${index}"]`);
            const allowButton = row.querySelector('.allow-contract-btn');
            allowButton.classList.remove('allow-contract-btn');
            allowButton.classList.add('not-allow-contract');
            allowButton.textContent = 'Not allow Contract';
        }
    });
}

function handleNotAllowContract(index) {
    handleTransactionOperation(
        `/bank/transaction/not_allow_contract/${filteredData[index].transaction.id}`,
        'not allow contract'
    ).then(success => {
        if (success) {
            // Update data
            filteredData[index].transaction.contract_not_allowed = true;
            transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).transaction.contract_not_allowed = true;

            // Update UI
            const row = document.querySelector(`.transaction-row[data-index="${index}"]`);
            const notAllowButton = row.querySelector('.not-allow-contract');
            notAllowButton.classList.remove('not-allow-contract');
            notAllowButton.classList.add('allow-contract-btn');
            notAllowButton.textContent = 'Allow Contract';

            // Optional: Re-render the row or update the table
            updateTransactionTable();
        }
    });
}

function handleAddContract(index) {
    // Check if the modal already exists; if so, remove it
    const existingModal = document.getElementById('contractModal');
    if (existingModal) {
        existingModal.remove();
    }

    // Create backdrop div
    const backdrop = document.createElement('div');
    backdrop.id = 'contractModal';
    backdrop.className = 'alert-backdrop';

    // Create the modal container
    const modal = document.createElement('div');
    modal.className = 'alert alert-info';

    // Create the inner container with horizontal layout
    const horizontalContainer = document.createElement('div');
    horizontalContainer.className = 'container-without-border-horizontally';

    // Add icon and header text
    const icon = document.createElement('span');
    icon.className = 'alert-icon';
    icon.textContent = 'ℹ️';

    const headerText = document.createElement('strong');
    headerText.textContent = 'Pick a contract from this list:';

    // Flex-grow div to push the header to the left
    const flexDiv = document.createElement('div');
    flexDiv.style.flexGrow = '1';
    flexDiv.appendChild(headerText);

    // Append icon and headerText to horizontalContainer
    horizontalContainer.appendChild(icon);
    horizontalContainer.appendChild(flexDiv);

    // Create body text
    const bodyText = document.createElement('p');
    bodyText.textContent = 'Please select a contract from the list below:';

    // Create select dropdown
    const select = document.createElement('select');
    select.id = 'contractSelect';
    contracts.forEach(contract => {
        const option = document.createElement('option');
        option.value = contract.id;
        option.text = contract.name;
        select.add(option);
    });

    // Create buttons container with horizontal alignment
    const buttonContainer = document.createElement('div');
    buttonContainer.classList.add('container-without-border-horizontally-header');

    // Create Add and Cancel buttons
    const addButton = document.createElement('button');
    addButton.textContent = 'Add';
    addButton.onclick = () => addSelectedContract(index);

    const cancelButton = document.createElement('button');
    cancelButton.textContent = 'Cancel';
    cancelButton.onclick = closeModal;

    // Append buttons to the buttonContainer
    buttonContainer.appendChild(addButton);
    buttonContainer.appendChild(cancelButton);

    // Create the main container and append all elements
    const container = document.createElement('div');
    container.className = 'container-without-border';
    container.appendChild(horizontalContainer);
    container.appendChild(bodyText);
    container.appendChild(select);
    container.appendChild(buttonContainer);

    // Append the main container to the modal
    modal.appendChild(container);

    // Append modal to the backdrop
    backdrop.appendChild(modal);

    // Append backdrop to the body
    document.body.appendChild(backdrop);

    // Show the modal
    backdrop.style.display = 'flex';
}

function addSelectedContract(index) {
    const selectedContractId = document.getElementById('contractSelect').value;

    handleTransactionOperation(
        `/bank/transaction/add_contract/${filteredData[index].transaction.id}/${selectedContractId}`,
        'add contract',
        { contract_id: selectedContractId }
    ).then(success => {
        if (success) {
            // Find the selected contract
            const selectedContract = contracts.find(contract => contract.id == selectedContractId);

            // Update data
            filteredData[index].contract = selectedContract;
            transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).contract = selectedContract;

            // Update UI
            const row = document.querySelector(`.transaction-row[data-index="${index}"]`);
            const contractName = row.cells[5];
            contractName.textContent = selectedContract.name;
            const contractAmount = row.cells[6];
            contractAmount.innerHTML = `<span class="${selectedContract.current_amount < 0 ? 'negative' : 'positive'}">$${selectedContract.current_amount.toFixed(2)}</span>`;

            const contractActionButton = row.querySelector('.add-contract-btn');
            contractActionButton.classList.remove('add-contract-btn');
            contractActionButton.classList.add('remove-contract-btn');
            contractActionButton.textContent = 'Remove Contract';

            // Close the modal after adding the contract
            closeModal();
        }
    });
}

function closeModal() {
    const modal = document.getElementById('contractModal');
    if (modal) {
        modal.remove();
    }
}