import { formatDate, displayCustomAlert, getGlobalLanguage } from './utils.js';
import { log, error } from './main.js';

let filteredData = [];
let transactionsData = [];
let sortConfig = { key: 'date', ascending: false };
let dateRange = { start: null, end: null };
let showOrHideTransaction = false;
let contracts = [];

const translations = {
    English: {
        allowContract: 'Allow contract',
        notAllowContract: 'Not allow contract',
        removeContract: 'Remove contract',
        addContract: 'Add contract',
        hide: 'Hide',
        display: 'Display',
        contractNotAllowed: 'Contract not allowed',
        pickContractHeader: 'Pick a contract from this list:',
        selectContractBody: 'Please select a contract from the list below:',
        addButton: 'Add',
        cancelButton: 'Cancel',
        contractAmountMismatch: 'The contract amount does not match the transaction amount. Please select an option:',
        newContractAmountOption: 'Set a new contract amount<br>(updates current amount and bases new transactions on it)',
        oldContractAmountOption: 'Mark as an old contract amount<br>(adds to contract history)',
        addToContractOption: 'Add to the contract<br>(included in calculations, but not in history)',
        submitButton: 'Submit'
    },
    German: {
        allowContract: 'Vertrag erlauben',
        notAllowContract: 'Vertrag nicht erlauben',
        removeContract: 'Vertrag entfernen',
        addContract: 'Vertrag hinzufügen',
        hide: 'Verbergen',
        display: 'Anzeigen',
        contractNotAllowed: 'Vertrag nicht erlaubt',
        pickContractHeader: 'Wählen Sie einen Vertrag aus dieser Liste:',
        selectContractBody: 'Bitte wählen Sie einen Vertrag aus der folgenden Liste:',
        addButton: 'Hinzufügen',
        cancelButton: 'Abbrechen',
        contractAmountMismatch: 'Der Vertragsbetrag stimmt nicht mit dem Transaktionsbetrag überein. Bitte wählen Sie eine Option:',
        newContractAmountOption: 'Einen neuen Vertragsbetrag festlegen<br>(aktualisiert den aktuellen Betrag und basiert neue Transaktionen darauf)',
        oldContractAmountOption: 'Als alten Vertragsbetrag markieren<br>(fügt der Vertragshistorie hinzu)',
        addToContractOption: 'Zum Vertrag hinzufügen<br>(in Berechnungen enthalten, aber nicht in der Historie)',
        submitButton: 'Einreichen'
    }
};

export async function setupTransactions() {
    listernerAdded = false;

    await loadTransactions();
}

let listernerAdded = false;

async function loadTransactions() {
    const start = performance.now();

    await get_transaction_data();

    filteredData = transactionsData;

    fillContractFilter();

    if (listernerAdded === false) {
        setupEventListeners();
        listernerAdded = true;
    }

    filterTransactions();

    const end = performance.now();
    log(`loadTransactions execution time: ${(end - start).toFixed(2)}ms`, 'loadTransactions');
}

async function get_transaction_data() {
    const start = performance.now();
    log('Getting transaction data', 'get_transaction_data');

    try {
        const response = await fetch('/bank/transaction/data', { method: 'GET' });

        if (!response.ok) throw new Error('Failed to send request to the server');

        const data = await response.json();

        if (data.error) {
            displayCustomAlert('error', data.header, data.error);
            return;
        }

        if (data.transactions) {
            transactionsData = JSON.parse(data.transactions);
        }

    } catch (err) {
        error('Error while getting transaction data:', 'get_transaction_data', err);
        displayCustomAlert('error', 'Error', 'Failed to get transaction data');
    } finally {
        const end = performance.now();
        log(`get_transaction_data execution time: ${(end - start).toFixed(2)}ms`, 'get_transaction_data');
    }
}

function fillContractFilter() {
    const contractFilter = document.getElementById('contract-filter');

    for (let i = contractFilter.options.length - 1; i >= 0; i--) {
        const optionText = contractFilter.options[i].textContent;

        if (optionText.includes('All Contracts') || optionText.includes('Alle Verträge')) {
            continue;
        } else {
            contractFilter.remove(i);
        }
    }


    const contractNames = [...new Set(transactionsData.map(t => t.contract?.name).filter(Boolean))];
    contractNames.forEach(name => {
        const option = document.createElement('option');
        option.value = name;
        option.textContent = name;
        contractFilter.appendChild(option);

        const contract = transactionsData.find(t => t.contract?.name === name).contract;

        if (!contracts.find(c => c.id === contract.id)) {
            contracts.push(contract);
        }
    });
}

function generateTransactionHTML({ transaction, contract }, index) {
    const amountClass = transaction.amount < 0 ? 'negative' : 'positive';
    const balanceClass = transaction.bank_balance_after < 0 ? 'negative' : 'positive';
    const rowClass = transaction.is_hidden ? 'hidden_transaction' : '';
    const t = translations[getGlobalLanguage()] || translations['en'];

    let displayStyle = 'table-row';
    if (transaction.is_hidden) {
        displayStyle = showOrHideTransaction ? 'table-row' : 'none';
    }

    let contractAllowed = transaction.contract_not_allowed ?
        `<button class="table_button button btn-secondary allow-contract-btn" data-index="${index}">${t.allowContract}</button>` :
        `<button class="table_button button btn-secondary not-allow-contract" data-index="${index}">${t.notAllowContract}</button>`;

    let dropdownMenu = ''; // Initialize the dropdownMenu variable
    let contractName = ''; // Initialize contractName
    let contractAmount = ''; // Initialize contractAmount

    // Determine if the transaction is hidden or shown and set button text accordingly
    const hideButtonText = transaction.is_hidden ? t.display : t.hide;
    const hideButtonClass = transaction.is_hidden ? 'show-btn' : 'hide-btn';

    if (contract) {
        // Determine class based on the contract amount
        const contractAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
        contractName = contract.name;
        contractAmount = `<span class="${contractAmountClass}">$${contract.current_amount.toFixed(2)}</span>`;

        // Generate dropdown for existing contracts
        dropdownMenu = `
                <div class="dropdown-content" style="display:none;">
                    <button class="table_button button btn-secondary remove-contract-btn" data-index="${index}">${t.removeContract}</button>
                    ${contractAllowed}
                    <button class="table_button button btn-secondary ${hideButtonClass}" data-index="${index}">${hideButtonText}</button>
                </div>`;
    } else {
        // Generate dropdown for adding a new contract
        dropdownMenu = `
                <div class="dropdown-content" style="display:none;">
                    <button class="table_button button btn-secondary add-contract-btn" data-index="${index}">${t.addContract}</button>
                    ${contractAllowed}
                    <button class="table_button button btn-secondary ${hideButtonClass}" data-index="${index}">${hideButtonText}</button>
                </div>`;
    }

    const emptyCellIcon = contract
        ? `<img src="/static/images/contract.png" alt="${t.contractAltText || 'Contract'}" class="icon">`
        : (transaction.contract_not_allowed
            ? `<img src="/static/images/not-allowed.png" alt="${t.notAllowedAltText || 'Not Allowed'}" class="icon">`
            : '');

    const html = `
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

    return html;
}

function setupEventListeners() {
    const start = performance.now();
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
    tableBody.addEventListener('click', async (event) => {
        const index = event.target.getAttribute('data-index');
        const targetClassList = event.target.classList;

        switch (true) {
            case targetClassList.contains('remove-contract-btn'):
                removeContract(index);
                break;
            case targetClassList.contains('add-contract-btn'):
                handleAddContract(index);
                break;
            case targetClassList.contains('hide-btn'):
                handleHideTransaction(index);
                break;
            case targetClassList.contains('show-btn'):
                handleShowTransaction(index);
                break;
            case targetClassList.contains('allow-contract-btn'):
                handleAllowContract(index);
                break;
            case targetClassList.contains('not-allow-contract'):
                handleNotAllowContract(index);
                break;
            default:
                // No matching case
                break;
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
    const end = performance.now();
    log(`setupEventListeners execution time: ${(end - start).toFixed(2)}ms`, 'setupEventListeners');
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
    const start = performance.now();
    sortConfig.ascending = (sortConfig.key === key) ? !sortConfig.ascending : true;
    sortConfig.key = key;

    sortData();
    updateSortIcons();
    const end = performance.now();
    log(`sortColumn execution time: ${(end - start).toFixed(2)}ms`, 'sortColumn');
};

function sortData() {
    const start = performance.now();
    if (!sortConfig.key) return;

    // Helper function to get the value based on sortConfig.key
    function getValue(item) {
        if (sortConfig.key === 'icon') {
            if (item.contract) {
                return 1
            } else if (item.transaction.contract_not_allowed) {
                return 2;
            }
            return 0;
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

    const noVisibleTransaction = filteredData.find(item => !item.transaction.is_hidden);

    if (!noVisibleTransaction && !showOrHideTransaction) {
        showHiddenTransactions();
    }

    updateTransactionTable();
    const end = performance.now();
    log(`sortData execution time: ${(end - start).toFixed(2)}ms`, 'sortData');
}

function updateSortIcons() {
    document.querySelectorAll('.sortable').forEach(header => {
        const icon = header.firstElementChild;

        if (header.dataset.key === sortConfig.key) {
            icon.src = sortConfig.ascending ? "/static/images/up.png" : "/static/images/down.png";
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
function handleTransactionOperation(url) {
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
                    displayCustomAlert('success', json.header, json.success);
                    return true;
                } else if (json.error) {
                    displayCustomAlert('error', json.header, json.error);
                    return false;
                }

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
    ).then(async success => {
        if (success) {
            // Update data
            filteredData[index].contract = null;
            transactionsData.find(t => t.transaction.id === filteredData[index].transaction.id).contract = null;

            await loadTransactions();
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
            hideButton.textContent = translations[getGlobalLanguage()].display;
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
            showButton.textContent = translations[getGlobalLanguage()].hide;
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

            updateTransactionTable();
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
            notAllowButton.textContent = translations[getGlobalLanguage()].allowContract;

            updateTransactionTable();
        }
    });
}

function handleAddContract(index) {
    const t = translations[getGlobalLanguage()];

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
    horizontalContainer.className = 'container-without-border-horizontally-center';

    // Add icon and header text
    const icon = document.createElement('img');
    icon.src = '/static/images/info.png';
    icon.alt = 'Icon';
    icon.classList.add('icon-big');

    const headerText = document.createElement('strong');
    headerText.textContent = t.pickContractHeader; // Localized header text

    // Flex-grow div to push the header to the left
    const flexDiv = document.createElement('div');
    flexDiv.style.flexGrow = '1';
    flexDiv.appendChild(headerText);

    // Append icon and headerText to horizontalContainer
    horizontalContainer.appendChild(icon);
    horizontalContainer.appendChild(flexDiv);

    // Create body text
    const bodyText = document.createElement('p');
    bodyText.textContent = t.selectContractBody; // Localized body text

    // Create the select element with contract options
    const select = document.createElement('select');
    select.classList.add('input');
    select.id = 'contractSelect';
    contracts.forEach(contract => {
        const option = document.createElement('option');
        option.value = contract.id;
        option.textContent = contract.name; // Use textContent for consistency
        select.appendChild(option);
    });

    // Create button container with localized buttons
    const buttonContainer = document.createElement('div');
    buttonContainer.classList.add('container-without-border-horizontally-center');

    const addButton = document.createElement('button');
    addButton.innerHTML = `
        <img src="/static/images/add.png" alt="${t.addButton}" class="button-icon">
        <span>${t.addButton}</span>
    `; // Image and text for add button
    addButton.classList.add('button', 'btn-secondary');
    addButton.addEventListener('click', () => addSelectedContract(index)); // Use addEventListener for consistency

    const cancelButton = document.createElement('button');
    cancelButton.innerHTML = `
        <img src="/static/images/back.png" alt="${t.cancelButton}" class="button-icon">
        <span>${t.cancelButton}</span>
    `; // Image and text for cancel button
    cancelButton.classList.add('button', 'btn-secondary');
    cancelButton.addEventListener('click', closeModal); // Use addEventListener for consistency

    buttonContainer.appendChild(addButton);
    buttonContainer.appendChild(cancelButton);

    // Create and append the final container
    const container = document.createElement('div');
    container.className = 'container-without-border';
    container.appendChild(horizontalContainer);
    container.appendChild(bodyText);
    container.appendChild(select);
    container.appendChild(buttonContainer);

    modal.appendChild(container);
    backdrop.appendChild(modal);
    document.body.appendChild(backdrop);
}

function addSelectedContract(index) {
    const t = translations[getGlobalLanguage()];

    const selectedContractId = document.getElementById('contractSelect').value;
    const selectedContract = contracts.find(contract => contract.id == selectedContractId);
    const transaction = filteredData[index].transaction;

    // Check if the contract amount matches the transaction amount
    if (selectedContract && selectedContract.current_amount !== transaction.amount) {
        // Create a modal for user choices
        const modal = document.createElement('div');
        modal.className = 'alert alert-info';
        modal.classList.add('container-without-border');

        const headerContainer = document.createElement('div');
        headerContainer.classList.add('container-without-border-horizontally-center');

        const icon = document.createElement('img');
        icon.src = '/static/images/info.png';
        icon.alt = 'Icon';
        icon.classList.add('icon-big');

        const headerText = document.createElement('strong');
        headerText.textContent = t.contractAmountMismatch;

        headerContainer.appendChild(icon);
        headerContainer.appendChild(headerText);

        const radioContainer = document.createElement('div');
        radioContainer.className = 'container-without-border';

        // Create radio button choices
        const options = [
            { id: 'new-contract-amount', label: t.newContractAmountOption },
            { id: 'old-contract-amount', label: t.oldContractAmountOption },
            { id: 'just-add-to-contract', label: t.addToContractOption }
        ];

        options.forEach(option => {
            const radio = document.createElement('input');
            radio.type = 'radio';
            radio.name = 'contractAmountChoice';
            radio.value = option.id;
            radio.id = option.id;

            const label = document.createElement('label');
            label.htmlFor = option.id;
            label.innerHTML = option.label;

            const optionContainer = document.createElement('div');
            optionContainer.classList.add('container-without-border-horizontally-center');
            optionContainer.appendChild(radio);
            optionContainer.appendChild(label);

            radioContainer.appendChild(optionContainer);
        });

        // Add submit and cancel buttons
        const buttonContainer = document.createElement('div');
        buttonContainer.classList.add('container-without-border-horizontally-center');

        const submitButton = document.createElement('button');
        submitButton.classList.add('button', 'btn-secondary');
        submitButton.textContent = 'Submit';
        submitButton.onclick = async () => {
            const selectedOption = document.querySelector('input[name="contractAmountChoice"]:checked');
            if (selectedOption) {
                handleContractChoice(selectedOption.value, index, selectedContractId);
            }
        };

        const cancelButton = document.createElement('button');
        cancelButton.classList.add('button', 'btn-secondary');
        cancelButton.textContent = t.cancelButton;
        cancelButton.onclick = closeModal;

        buttonContainer.appendChild(submitButton);
        buttonContainer.appendChild(cancelButton);

        modal.appendChild(headerContainer);
        modal.appendChild(radioContainer);
        modal.appendChild(buttonContainer);

        const backdrop = document.createElement('div');
        backdrop.id = 'contractModal';
        backdrop.className = 'alert-backdrop';
        backdrop.appendChild(modal);

        document.body.appendChild(backdrop);
        backdrop.style.display = 'flex';
    } else {
        // If amounts match, just add the contract
        handleTransactionOperation(
            `/bank/transaction/add_contract/${transaction.id}/${selectedContractId}`,
            'add contract'
        ).then(success => {
            if (success) {
                filteredData[index].contract = selectedContract;
                transactionsData.find(t => t.transaction.id === transaction.id).contract = selectedContract;

                updateTransactionTable();
                closeModal();
            }
        });
    }
}

function handleContractChoice(choice, index, selectedContractId) {
    const transaction = filteredData[index].transaction;
    let url = '';

    // Determine URL and error message based on user choice
    switch (choice) {
        case 'new-contract-amount':
            url = `/bank/transaction/update_contract_amount/${transaction.id}/${selectedContractId}`;
            break;
        case 'old-contract-amount':
            url = `/bank/transaction/set_old_amount/${transaction.id}/${selectedContractId}`;
            break;
        case 'just-add-to-contract':
            url = `/bank/transaction/add_contract/${transaction.id}/${selectedContractId}`;
            break;
    }

    handleTransactionOperation(url).then(async success => {
        if (success) {
            await loadTransactions();
            closeModal();
        }
    });
}

function closeModal() {
    const modals = document.querySelectorAll('.alert-backdrop');
    modals.forEach(modal => modal.remove());
}