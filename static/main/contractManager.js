import { log, error } from './main.js';
import { formatDate, displayCustomAlert, getGlobalLanguage } from './utils.js';

const translations = {
    English: {
        openContractsTitle: 'Open Contracts',
        closedContractsTitle: 'Closed Contracts',
        showHistory: 'Show History',
        hideHistory: 'Hide History',
        noHistoryAvailable: 'No History Available.',
        mergeSelected: 'Please select at least 2 contracts to merge.',
        deleteSelected: 'Please select at least 1 contract to delete.',
        currentAmount: 'Current Amount',
        totalAmountOverTime: 'Total Amount Over Time',
        monthsBetweenPayment: 'Months Between Payment',
        oldAmount: 'Old Amount',
        newAmount: 'New Amount',
        changedAt: 'Changed At',
        endDate: 'End Date',
        lastPaymentDate: 'Last Payment Date',
        contractHistory: 'Contract History',
    },
    German: {
        openContractsTitle: 'Offene Verträge',
        closedContractsTitle: 'Abgeschlossene Verträge',
        showHistory: 'Historie anzeigen',
        hideHistory: 'Historie verstecken',
        noHistoryAvailable: 'Keine Historie verfügbar.',
        mergeSelected: 'Bitte wählen Sie mindestens 2 Verträge zum Zusammenführen aus.',
        deleteSelected: 'Bitte wählen Sie mindestens 1 Vertrag zum Löschen aus.',
        currentAmount: 'Aktueller Betrag',
        totalAmountOverTime: 'Gesamtbetrag über die Zeit',
        monthsBetweenPayment: 'Monate zwischen Zahlungen',
        oldAmount: 'Alter Betrag',
        newAmount: 'Neuer Betrag',
        changedAt: 'Geändert am',
        endDate: 'Enddatum',
        lastPaymentDate: 'Letztes Zahlungsdatum',
        contractHistory: 'Vertragsgeschichte',
    }
};

const closedContractsWrapper = document.createElement('div');
const openContractsWrapper = document.createElement('div');

export function loadContracts() {
    const start = performance.now();
    log('Loading contracts', 'loadContracts');

    try {
        closedContractsWrapper.classList.add('display-container');
        closedContractsWrapper.style.display = 'none';
        openContractsWrapper.classList.add('display-container');

        get_contract_data();
        setupEventListeners();
        document.getElementById('toggle-closed-contracts').addEventListener('click', showClosedContracts);
    } catch (err) {
        error(`Error loading contracts: ${err.message}`, 'loadContracts');
        displayCustomAlert('error', 'Loading Error', err.message);
    } finally {
        const end = performance.now();
        log(`Finished loading contracts in ${end - start}ms`, 'loadContracts');
    }
}

async function get_contract_data() {
    const start = performance.now();
    log('Fetching contract data', 'get_contract_data');

    try {
        const response = await fetch('/bank/contract/data', { method: 'GET' });

        if (!response.ok) throw new Error('Failed to send request to the server');

        const contractsData = await response.json();

        if (contractsData.error) {
            displayCustomAlert('error', contractsData.header, contractsData.error);
            return;
        }

        if (contractsData.contracts) {
            const contractJSON = JSON.parse(contractsData.contracts);
            updateContractsView(contractJSON);
        }
    } catch (err) {
        error(`Error fetching contracts: ${err.message}`, 'get_contract_data');
        displayCustomAlert('error', 'Failed to load contracts', err.message);
    } finally {
        const end = performance.now();
        log(`Fetched contract data in ${end - start}ms`, 'get_contract_data');
    }
}

function updateContractsView(contractsData) {
    const start = performance.now();
    log('Updating contracts view', 'updateContractsView');

    try {
        const container = document.getElementById('contract-container');

        closedContractsWrapper.innerHTML = '';
        openContractsWrapper.innerHTML = '';

        contractsData.forEach((contractWithHistory, index) => {
            const contractHTML = generateContractHTML(contractWithHistory, index);
            const wrapper = contractWithHistory.contract.end_date ? closedContractsWrapper : openContractsWrapper;
            wrapper.insertAdjacentHTML('beforeend', contractHTML);
        });

        container.innerHTML = '';

        let openContractsTitle = document.getElementById('open-contracts-title');
        if (!openContractsTitle) {
            openContractsTitle = document.createElement('h3');
            openContractsTitle.id = 'open-contracts-title';
            openContractsTitle.textContent = translations[getGlobalLanguage()].openContractsTitle;
        }
        container.appendChild(openContractsTitle);
        container.appendChild(openContractsWrapper);

        let closedContractsTitle = document.getElementById('closed-contracts-title');
        if (!closedContractsTitle) {
            closedContractsTitle = document.createElement('h3');
            closedContractsTitle.id = 'closed-contracts-title';
            closedContractsTitle.textContent = translations[getGlobalLanguage()].closedContractsTitle;
        }
        const toggleButton = document.getElementById('toggle-closed-contracts');
        const slider = toggleButton.querySelector('.slider');

        const displayStyle = slider.classList.contains('active') ? 'flex' : 'none';

        closedContractsTitle.style.display = displayStyle;
        container.appendChild(closedContractsTitle);
        container.appendChild(closedContractsWrapper);

        attachContractEventListeners();
    } finally {
        const end = performance.now();
        log(`Updated contracts view in ${end - start}ms`, 'updateContractsView');
    }
}

function attachContractEventListeners() {
    const start = performance.now();
    log('Attaching event listeners to contracts', 'attachContractEventListeners');

    try {
        const container = document.querySelectorAll('.display');

        container.forEach((contractElement, index) => {
            contractElement.addEventListener('click', (event) => {
                const target = event.target;

                if (target.classList.contains('toggle-history-btn')) {
                    event.stopPropagation();
                    const index = target.getAttribute('data-index');
                    const historyElement = document.getElementById(`contract-history-${index}`);
                    const isHidden = historyElement.classList.toggle('hidden');
                    const lang = getGlobalLanguage();
                    const icon = target.querySelector('img');

                    // Update the button text based on the state
                    target.childNodes[1].textContent = isHidden ? translations[lang].showHistory : translations[lang].hideHistory;
                    target.setAttribute('aria-expanded', isHidden ? 'false' : 'true');

                    // Update the icon based on the state
                    icon.src = isHidden
                        ? '/static/images/show.png'
                        : '/static/images/hide.png';
                    icon.setAttribute('data-state', isHidden ? 'show' : 'hide');

                    return;
                }

                const contractElement = target.closest('.display');
                if (contractElement) {
                    const isSelected = contractElement.classList.contains('selected');

                    if (isSelected) {
                        contractElement.classList.remove('selected');
                    } else {
                        contractElement.classList.add('selected');
                    }
                }
            });

            const contractNameInput = contractElement.querySelector('.contract-name');
            contractNameInput.addEventListener('keydown', (event) => {
                if (event.key === 'Enter') {
                    handleContractNameChange(event);
                    contractNameInput.blur();
                }
            });

            contractNameInput.addEventListener('blur', (event) => {
                handleContractNameChange(event);
            });
        });
    } finally {
        const end = performance.now();
        log(`Attached event listeners in ${end - start}ms`, 'attachContractEventListeners');
    }
}

function handleContractNameChange(event) {
    const start = performance.now();
    log('Handling contract name change', 'handleContractNameChange');

    try {
        const index = event.target.getAttribute('data-index');
        const contractElement = document.getElementById(`display-${index}`);
        const contractID = contractElement.getAttribute('data-id');
        const contractName = event.target.value;

        fetch(`/bank/contract/nameChanged/${contractID}/${contractName}`, { method: 'GET' });
    } catch (err) {
        error(`Error handling contract name change: ${err.message}`, 'handleContractNameChange');
    } finally {
        const end = performance.now();
        log(`Handled contract name change in ${end - start}ms`, 'handleContractNameChange');
    }
}

function setupEventListeners() {
    const start = performance.now();
    log('Setting up event listeners', 'setupEventListeners');

    try {
        document.getElementById('merge-selected-btn').addEventListener('click', mergeSelectedContracts);
        document.getElementById('delete-selected-btn').addEventListener('click', deleteSelectedContracts);
        document.getElementById('scan-btn').addEventListener('click', scanContracts);
    } finally {
        const end = performance.now();
        log(`Setup event listeners in ${end - start}ms`, 'setupEventListeners');
    }
}

async function scanContracts() {
    const start = performance.now();
    log('Scanning contracts', 'scanContracts');

    try {
        const response = await fetch('/bank/contract/scan', { method: 'GET' });

        if (!response.ok) throw new Error('Failed to send request to the server');

        const result = await response.json();

        if (result.error) {
            displayCustomAlert('error', result.header, result.error);
            return;
        }

        if (result.success) {
            get_contract_data();
            displayCustomAlert('success', result.header, result.success);
        }
    } catch (err) {
        error(`Error scanning contracts: ${err.message}`, 'scanContracts');
        displayCustomAlert('error', 'Scan Failed', err.message);
    } finally {
        const end = performance.now();
        log(`Scanned contracts in ${end - start}ms`, 'scanContracts');
    }
}

async function deleteSelectedContracts() {
    const start = performance.now();
    log('Deleting selected contracts', 'deleteSelectedContracts');

    try {
        const selectedContracts = document.querySelectorAll('.selected');

        if (selectedContracts.length === 0) {
            displayCustomAlert('error', 'Delete contracts', 'Please select at least 1 contract to delete.');
            return;
        }

        const contractIDs = Array.from(selectedContracts).map(contract => parseInt(contract.getAttribute('data-id'), 10));

        const response = await fetch('/bank/contract/delete', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ids: contractIDs }),
        });

        if (!response.ok) throw new Error('Failed to send IDs to the server');

        const result = await response.json();

        if (result.error) {
            displayCustomAlert('error', result.header, result.error);
            return;
        }

        if (result.success) {
            selectedContracts.forEach(contract => contract.remove());
            displayCustomAlert('success', result.header, result.success);
        }
    } catch (err) {
        error(`Error deleting contracts: ${err.message}`, 'deleteSelectedContracts');
        displayCustomAlert('error', 'Delete Failed', err.message);
    } finally {
        const end = performance.now();
        log(`Deleted contracts in ${end - start}ms`, 'deleteSelectedContracts');
    }
}

async function mergeSelectedContracts() {
    const start = performance.now();
    log('Merging selected contracts', 'mergeSelectedContracts');

    try {
        const selectedContracts = document.querySelectorAll('.selected');

        if (selectedContracts.length < 2) {
            displayCustomAlert('error', 'Merge contracts', 'Please select at least 2 contracts to merge.');
            return;
        }

        const contractIDs = Array.from(selectedContracts).map(contract => parseInt(contract.getAttribute('data-id'), 10));

        const response = await fetch('/bank/contract/merge', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ids: contractIDs }),
        });

        if (!response.ok) throw new Error('Failed to send IDs to the server');

        const updatedContractsData = await response.json();

        if (updatedContractsData.error) {
            displayCustomAlert('error', updatedContractsData.header, updatedContractsData.error);
            return;
        }

        get_contract_data();
        displayCustomAlert('success', 'Merge Successful', 'Contracts have been successfully merged.');
    } catch (err) {
        error(`Error merging contracts: ${err.message}`, 'mergeSelectedContracts');
        displayCustomAlert('error', 'Merge Failed', err.message);
    } finally {
        const end = performance.now();
        log(`Merged contracts in ${end - start}ms`, 'mergeSelectedContracts');
    }
}

function generateContractHTML(contractWithHistory, index) {
    const start = performance.now();
    log('Generating contract HTML', 'generateContractHTML');

    const { contract, contract_history, total_amount_paid, last_payment_date } = contractWithHistory;
    const currentAmountClass = contract.current_amount < 0 ? 'negative' : 'positive';
    const totalAmountClass = total_amount_paid < 0 ? 'negative' : 'positive';
    const lang = getGlobalLanguage();

    // Localized strings
    const dateLabel = contract.end_date
        ? translations[lang].endDate
        : translations[lang].lastPaymentDate;
    const showHistoryText = translations[lang].showHistory;

    const showImageSrc = '/static/images/show.png';

    const dateValue = contract.end_date
        ? formatDate(contract.end_date)
        : formatDate(last_payment_date);

    const html = `
        <div class="display" id="display-${index}" data-id="${contract.id}">
            <div class="container-without-border-horizontally-header">
                <img src="/static/images/edit.png" alt="Edit Icon" class="edit-icon" />
                <input type="text" class="contract-name" value="${contract.name}" data-index="${index}" />
            </div>
            <p>${translations[lang].currentAmount}: <span class="${currentAmountClass}">$${contract.current_amount.toFixed(2)}</span></p>
            <p>${translations[lang].totalAmountOverTime}: <span class="${totalAmountClass}">$${total_amount_paid.toFixed(2)}</span></p>
            <p>${translations[lang].monthsBetweenPayment}: ${contract.months_between_payment}</p>
            <p>${dateLabel}: <span>${dateValue}</span></p>
            <button class="toggle-history-btn" data-index="${index}">
                <img src="${showImageSrc}" alt="Toggle History" data-state="show">
                ${showHistoryText}
            </button>
            <div id="contract-history-${index}" class="hidden contract-history">
                <h4>${translations[lang].contractHistory}:</h4>
                <ul>${generateHistoryHTML(contract_history)}</ul>
            </div>
        </div>
    `;

    const end = performance.now();
    log(`Generated contract HTML in ${end - start}ms`, 'generateContractHTML');

    return html;
}

function generateHistoryHTML(contractHistory) {
    const start = performance.now();
    log('Generating history HTML', 'generateHistoryHTML');

    const lang = getGlobalLanguage();

    const html = contractHistory.length === 0
        ? `<li>${translations[lang].noHistoryAvailable}</li>`
        : contractHistory.map(({ old_amount, new_amount, changed_at }) => `
            <li>
                <p>${translations[lang].oldAmount}: <span class="${old_amount < 0 ? 'negative' : 'positive'}">$${old_amount.toFixed(2)}</span></p>
                <p>${translations[lang].newAmount}: <span class="${new_amount < 0 ? 'negative' : 'positive'}">$${new_amount.toFixed(2)}</span></p>
                <p>${translations[lang].changedAt}: ${formatDate(changed_at)}</p>
            </li>
        `).join('');

    const end = performance.now();
    log(`Generated history HTML in ${end - start}ms`, 'generateHistoryHTML');

    return html;
}

function showClosedContracts() {
    const start = performance.now();
    log('Showing closed contracts', 'showClosedContracts');

    try {
        const toggleButton = document.getElementById('toggle-closed-contracts');
        const slider = toggleButton.querySelector('.slider');

        slider.classList.toggle('active');

        const closedContractsTitle = document.getElementById('closed-contracts-title');
        closedContractsTitle.style.display = closedContractsTitle.style.display === 'none' ? 'flex' : 'none';

        closedContractsWrapper.style.display = closedContractsWrapper.style.display === 'none' ? 'flex' : 'none';
    } finally {
        const end = performance.now();
        log(`Displayed closed contracts in ${end - start}ms`, 'showClosedContracts');
    }
}
