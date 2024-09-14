import { error, log } from './main.js';
import { displayCustomAlert, parseJsonResponse, getGlobalLanguage, setGlobalLanguage } from './utils.js';

const localizedStrings = {
    English: {
        delete_account_header: "Delete Account",
        delete_account_confirmation: "Are you sure you want to delete your account? This action cannot be undone.",
        delete_account_button: "Delete Account",
        cancel_button: "Cancel",
        change_password_header: "Change Password",
        success_message: "Operation successful",
        language_set_success: "Language set to ",
        language_set_error: "Failed to set language"
    },
    German: {
        delete_account_header: "Konto löschen",
        delete_account_confirmation: "Sind Sie sicher, dass Sie Ihr Konto löschen möchten? Diese Aktion kann nicht rückgängig gemacht werden.",
        delete_account_button: "Konto löschen",
        cancel_button: "Abbrechen",
        change_password_header: "Passwort ändern",
        success_message: "Aktion erfolgreich",
        language_set_success: "Sprache geändert zu ",
        language_set_error: "Fehler beim Einstellen der Sprache"
    }
};

// Function to get the localized string.
function getLocalizedString(key) {
    const language = getGlobalLanguage();
    if (!localizedStrings[language]) {
        error('Language not supported:', 'getLocalizedString', language);
        return '';
    }
    return localizedStrings[language][key] || key;
}

export function initializeSettings() {
    const startTime = performance.now();
    log('Initializing settings page', 'initializeSettings');

    const forms = document.getElementById('change_passwordForm');
    if (forms) {
        log('Change password form found', 'initializeSettings');
        handleChangePasswordForm(forms);
    } else {
        error('Change password form not found', 'initializeSettings');
    }

    document.getElementById('delete_account').addEventListener('click', function () {
        log('Delete account button clicked', 'initializeSettings');
        handleDeleteButton();
    });

    // Attach click event listeners to flag images
    document.getElementById('german-flag').addEventListener('click', function () {
        log('German flag clicked', 'initializeSettings');
        setLanguage('German');
    });

    document.getElementById('english-flag').addEventListener('click', function () {
        log('English flag clicked', 'initializeSettings');
        setLanguage('English');
    });

    const endTime = performance.now();
    log(`Settings page initialized in ${endTime - startTime} ms`, 'initializeSettings');
}

// Function to set the language
function setLanguage(new_language) {
    log(`Attempting to set language to ${new_language}`, 'setLanguage');
    fetch(`/user/set_language/${new_language}`, {
        method: 'GET',
        headers: {
            'Content-Type': 'application/json'
        }
    })
        .then(async response => {
            if (response.ok) {
                log(`Language set response received: ${new_language}`, 'setLanguage');

                let json = await response.json();
                log('Language set response JSON:', 'setLanguage', json);

                if (json.success) {
                    setGlobalLanguage(new_language);
                    log('Language set successfully, reloading page', 'setLanguage');
                    // Reload the page to update the language
                    window.location.reload();
                } else if (json.error) {
                    displayCustomAlert('error', json.header, json.error);
                    log('Language set error:', 'setLanguage', json.error);
                }
            } else {
                error('Failed to set language. Status: ' + response.status, 'setLanguage');
            }
        })
        .catch(err => {
            error('Error setting language', 'setLanguage', err);
        });
}

function handleChangePasswordForm(form) {
    log('Handling change password form submission', 'handleChangePasswordForm');
    form.addEventListener('submit', async function (event) {
        event.preventDefault();

        const formData = new FormData(form);
        log('Form submitted with data:', 'handleChangePasswordForm', Array.from(formData.entries()));

        try {
            const response = await fetch(form.action, {
                method: form.method,
                body: formData
            });

            if (!response.ok) throw new Error(`HTTP error! Status: ${response.status}`);

            const result = await parseJsonResponse(response);
            log('Form submission result:', 'handleChangePasswordForm', result);

            if (result.success) {
                displayCustomAlert('success', result.header, result.success);
            } else if (result.error) {
                const inputs = form.querySelectorAll('input');
                inputs.forEach(input => {
                    input.value = '';
                });
                displayCustomAlert('error', result.header, result.error);
            }

            if (!result.error) form.reset();
        } catch (err) {
            error('Error submitting form:', 'handleChangePasswordForm', err);
            const inputs = form.querySelectorAll('input');
            inputs.forEach(input => {
                input.value = '';
            });
        }
    });
}

function handleDeleteButton() {
    log('Prepare to handle delete account button click', 'handleDeleteButton');

    // Create backdrop div
    const backdrop = document.createElement('div');
    backdrop.id = 'deleteModal';
    backdrop.className = 'alert-backdrop';

    // Create the modal container
    const modal = document.createElement('div');
    modal.className = 'alert alert-info';

    const horizontalContainer = document.createElement('div');
    horizontalContainer.className = 'container-without-border-horizontally-center';

    // Add icon and header text
    const icon = document.createElement('img');
    icon.src = '/static/images/info.png';
    icon.alt = 'Icon';
    icon.classList.add('icon-big');

    const headerText = document.createElement('strong');
    headerText.textContent = getLocalizedString('delete_account_header');

    const flexDiv = document.createElement('div');
    flexDiv.style.flexGrow = '1';
    flexDiv.appendChild(headerText);

    horizontalContainer.appendChild(icon);
    horizontalContainer.appendChild(flexDiv);

    // Create body text
    const bodyText = document.createElement('p');
    bodyText.textContent = getLocalizedString('delete_account_confirmation');

    const buttonContainer = document.createElement('div');
    buttonContainer.classList.add('container-without-border-horizontally-center');

    // Create delete button
    const deleteButton = document.createElement('button');
    deleteButton.textContent = getLocalizedString('delete_account_button');
    deleteButton.classList.add('btn-danger', 'button', 'btn-secondary')
    deleteButton.onclick = sendDeleteRequest;

    // Create cancel button
    const cancelButton = document.createElement('button');
    cancelButton.textContent = getLocalizedString('cancel_button');
    cancelButton.classList.add('button', 'btn-secondary')
    cancelButton.onclick = closeModal;

    buttonContainer.appendChild(deleteButton);
    buttonContainer.appendChild(cancelButton);

    const container = document.createElement('div');
    container.className = 'container-without-border';
    container.appendChild(horizontalContainer);
    container.appendChild(bodyText);
    container.appendChild(buttonContainer);

    modal.appendChild(container);

    backdrop.appendChild(modal);

    document.body.appendChild(backdrop);

    log('Delete account modal displayed', 'handleDeleteButton');
}

function sendDeleteRequest() {
    log('Sending delete request', 'sendDeleteRequest');

    fetch('/delete_account', {
        method: 'GET'
    })
        .then(async response => {
            if (response.ok) {
                log('Delete request successful', 'sendDeleteRequest');

                let json = await response.json();
                log('Delete request response JSON:', 'sendDeleteRequest', json);

                if (json.success) {
                    displayCustomAlert('success', json.header, json.success, '', 5);
                    setTimeout(() => {
                        window.location.href = '/';
                    }, 5000);
                } else if (json.error) {
                    displayCustomAlert('error', json.header, json.error);
                }
            } else {
                error('Failed to delete account. Status: ' + response.status, 'sendDeleteRequest');
            }
        })
        .catch(err => {
            error('Error deleting account', 'sendDeleteRequest', err);
        });
}

function closeModal() {
    log('Closing delete account modal', 'closeModal');
    const modals = document.querySelectorAll('.alert-backdrop');
    modals.forEach(modal => modal.remove());
}
