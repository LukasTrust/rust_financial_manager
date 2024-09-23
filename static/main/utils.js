import { error, log } from './main.js';

// Initialize the state from localStorage or default to 'English'
const state = {
    language: localStorage.getItem('language') || 'English',
};

// Function to get the current language from the state or localStorage
export function getGlobalLanguage() {
    log('Retrieving global language:', 'getGlobalLanguage', state.language);
    return state.language;
}

// Function to set the global language and store it in localStorage
export function setGlobalLanguage(newLanguage) {
    log('Setting global language:', 'setGlobalLanguage', newLanguage);
    state.language = newLanguage;
    localStorage.setItem('language', newLanguage);
}

export function formatDate(dateString) {
    log('Formatting date:', 'formatDate', dateString);
    const date = new Date(dateString);
    if (isNaN(date.getTime())) {
        log('Invalid date, returning N/A:', 'formatDate', dateString);
        return 'N/A';
    }

    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();

    const formattedDate = `${day}.${month}.${year}`;
    log('Formatted date:', 'formatDate', formattedDate);
    return formattedDate;
}

export function displayCustomAlert(type, header_text, body_text, button_text = 'Close', countdown = 0) {
    log('Displaying custom alert:', 'displayCustomAlert', { type, header_text, body_text, button_text, countdown });

    // Create the backdrop
    const backdrop = document.createElement('div');
    backdrop.className = 'alert-backdrop';

    // Create the alert container
    const alert = document.createElement('div');
    alert.className = `alert alert-${type}`;

    // Create the icon based on the type
    let iconSrc;
    switch (type) {
        case 'error':
            iconSrc = '/static/images/error.png';
            break;
        case 'info':
            iconSrc = '/static/images/info.png';
            break;
        case 'success':
            iconSrc = '/static/images/success.png';
            break;
        default:
            iconSrc = '/static/images/info.png';
    }

    // Construct the HTML structure with image icons
    alert.innerHTML = `
        <div class="container-without-border">
            <div class="container-without-border-horizontally-center">
                <img src="${iconSrc}" alt="${type}" class="icon-big">
                <div style="flex-grow: 1;" class="alert-header-text">
                    <strong>${header_text}</strong> <span class="alert-timer">${countdown > 0 ? `(${countdown}s)` : ''}</span>
                </div>
            </div>
            <p>${body_text}</p>
            ${button_text ? `<button class="alert-close button btn-secondary" disabled>${button_text}</button>` : ''}
        </div>
    `;

    // Append the backdrop and the alert to the body
    document.body.appendChild(backdrop);
    document.body.appendChild(alert);

    // Timer logic
    if (countdown > 0) {
        const timerElement = alert.querySelector('.alert-timer');
        const closeButton = alert.querySelector('.alert-close');

        const timerInterval = setInterval(() => {
            countdown--;
            log('Countdown in progress:', 'displayCustomAlert', countdown);
            if (countdown > 0) {
                timerElement.textContent = `(${countdown}s)`;
            } else {
                clearInterval(timerInterval);
                timerElement.textContent = ''; // Clear the timer text
                if (closeButton) {
                    closeButton.disabled = false; // Enable the button
                }
            }
        }, 1000);

        // Cleanup on alert removal
        alert.addEventListener('remove', () => clearInterval(timerInterval));
    } else {
        // If no countdown is set or it's zero, enable the button immediately
        const closeButton = alert.querySelector('.alert-close');
        if (closeButton) {
            closeButton.disabled = false;
        }
    }

    // Add click event to the close button
    const closeButton = alert.querySelector('.alert-close');
    if (closeButton) {
        closeButton.addEventListener('click', () => {
            log('Alert closed:', 'displayCustomAlert', { type, header_text });
            if (document.body.contains(alert)) {
                document.body.removeChild(alert);
            }

            if (document.body.contains(backdrop)) {
                document.body.removeChild(backdrop);
            }
        });
    }
}

export async function parseJsonResponse(response) {
    try {
        log('Parsing JSON response:', 'parseJsonResponse', response);
        return await response.json();
    } catch (err) {
        error('Error parsing JSON response:', 'parseJsonResponse', response);
        throw new Error('Error parsing JSON response');
    }
}

export function closeModal() {
    log('Closing delete account modal', 'closeModal');
    const modals = document.querySelectorAll('.alert-backdrop');
    modals.forEach(modal => modal.remove());
}

export function getLocalizedString(key) {
    const language = getGlobalLanguage();
    if (!localizedStrings[language]) {
        error('Language not supported:', 'getLocalizedString', language);
        return '';
    }
    return localizedStrings[language][key] || key;
}

const localizedStrings = {
    English: {
        delete_account_header: "Delete Account",
        delete_account_confirmation: "Are you sure you want to delete your account? This action cannot be undone.",
        delete_account_button: "Delete Account",
        cancel_button: "Cancel",
        delete_bank_header: "Delete Bank",
        delete_bank_confirmation: "Are you sure you want to delete this bank? This action cannot be undone.",
        delete_bank_button: "Delete Bank",
        change_password_header: "Change Password",
        success_message: "Operation successful",
        language_set_success: "Language set to ",
        language_set_error: "Failed to set language",
        openContractsTitle: 'Open Contracts',
        closedContractsTitle: 'Closed Contracts',
        showHistory: 'Show History',
        hideHistory: 'Hide History',
        noHistoryAvailable: 'No History Available.',
        mergeSelected: 'Merge Selected Contracts',
        mergeSelected_detail: 'Please select at least 2 contracts to merge.',
        deleteSelected: 'Delete Selected Contracts',
        deleteSelected_detail: 'Please select at least 1 contract to delete.',
        scanFailed: 'Scan Failed',
        deletionFalied: 'Deletion Failed',
        mergeFailed: 'Merge Failed',
        currentAmount: 'Current Amount',
        totalAmountOverTime: 'Total Amount Over Time',
        monthsBetweenPayment: 'Months Between Payment',
        oldAmount: 'Old Amount',
        newAmount: 'New Amount',
        changedAt: 'Changed At',
        endDate: 'End Date',
        lastPaymentDate: 'Last Payment Date',
        contractHistory: 'Contract History',
        error_loading: 'Error loading contracts',
        allowContract: 'Allow contract',
        notAllowContract: 'Contract not allowed',
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
        delete_account_header: "Konto löschen",
        delete_account_confirmation: "Sind Sie sicher, dass Sie Ihr Konto löschen möchten? Diese Aktion kann nicht rückgängig gemacht werden.",
        delete_account_button: "Konto löschen",
        cancel_button: "Abbrechen",
        delete_bank_header: "Bank löschen",
        delete_bank_confirmation: "Sind Sie sicher, dass Sie diese Bank löschen möchten? Diese Aktion kann nicht rückgängig gemacht werden.",
        delete_bank_button: "Bank löschen",
        change_password_header: "Passwort ändern",
        success_message: "Aktion erfolgreich",
        language_set_success: "Sprache geändert zu ",
        language_set_error: "Fehler beim Einstellen der Sprache",
        openContractsTitle: 'Offene Verträge',
        closedContractsTitle: 'Abgeschlossene Verträge',
        showHistory: 'Historie anzeigen',
        hideHistory: 'Historie verstecken',
        noHistoryAvailable: 'Keine Historie verfügbar.',
        mergeSelected: 'Ausgewählte Verträge zusammenführen',
        mergeSelected_detail: 'Bitte wählen Sie mindestens 2 Verträge zum Zusammenführen aus.',
        deleteSelected: 'Ausgewählte Verträge löschen',
        deleteSelected_detail: 'Bitte wählen Sie mindestens 1 Vertrag zum Löschen aus.',
        scanFailed: 'Scan fehlgeschlagen',
        deletionFalied: 'Löschen fehlgeschlagen',
        mergeFailed: 'Zusammenführen fehlgeschlagen',
        currentAmount: 'Aktueller Betrag',
        totalAmountOverTime: 'Gesamtbetrag über die Zeit',
        monthsBetweenPayment: 'Monate zwischen Zahlungen',
        oldAmount: 'Alter Betrag',
        newAmount: 'Neuer Betrag',
        changedAt: 'Geändert am',
        endDate: 'Enddatum',
        lastPaymentDate: 'Letztes Zahlungsdatum',
        contractHistory: 'Vertragsgeschichte',
        error_loading: 'Fehler beim Laden der Verträge',
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