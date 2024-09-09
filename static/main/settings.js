import { error, log, setGloableLanguage } from './main.js';
import { displayCustomAlert, parseJsonResponse } from './utils.js';

export function initializeSettings() {
    const startTime = performance.now();
    log('Initializing settings page', 'initializeSettings');

    const forms = document.getElementById('change_passwordForm');

    handleChangePasswordForm(forms);

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
    log(`Setting language to ${new_language}`, 'setLanguage');
    fetch(`/user/set_language/${new_language}`, {
        method: 'GET',
        headers: {
            'Content-Type': 'application/json'
        }
    })
        .then(async response => {
            if (response.ok) {
                log(`Language set to ${new_language}`, 'setLanguage');

                let json = await response.json();

                if (json.success) {
                    setGloableLanguage(new_language);
                    // Reload the page to update the language
                    window.location.reload();
                }
                else if (json.error) {
                    displayCustomAlert('error', json.header, json.error);
                }
            } else {
                error('Failed to set language', 'setLanguage');
            }
        })
        .catch(err => {
            error('Error setting language', 'setLanguage', err);
        });
}

function handleChangePasswordForm(form) {
    form.addEventListener('submit', async function (event) {
        event.preventDefault();

        const formData = new FormData(form);

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
    log('Delete account button clicked', 'handleDeleteButton');

    // Create backdrop div
    const backdrop = document.createElement('div');
    backdrop.id = 'deleteModal';
    backdrop.className = 'alert-backdrop';

    // Create the modal container
    const modal = document.createElement('div');
    modal.className = 'alert alert-info';

    const horizontalContainer = document.createElement('div');
    horizontalContainer.className = 'container-without-border-horizontally';

    // Add icon and header text
    const icon = document.createElement('span');
    icon.className = 'alert-icon';
    icon.textContent = 'ℹ️';

    const headerText = document.createElement('strong');
    headerText.textContent = 'Delete Account';

    // Flex-grow div to push the header to the left
    const flexDiv = document.createElement('div');
    flexDiv.style.flexGrow = '1';
    flexDiv.appendChild(headerText);

    // Append icon and headerText to horizontalContainer
    horizontalContainer.appendChild(icon);
    horizontalContainer.appendChild(flexDiv);

    // Create body text
    const bodyText = document.createElement('p');
    bodyText.textContent = 'Are you sure you want to delete your account? This action cannot be undone.';

    const buttonContainer = document.createElement('div');
    buttonContainer.classList.add('container-without-border-horizontally-header');

    // Create delete button
    const deleteButton = document.createElement('button');
    deleteButton.textContent = 'Delete Account';
    deleteButton.style.backgroundColor = 'red';
    deleteButton.onclick = sendDeleteRequest;

    // Create cancel button
    const cancelButton = document.createElement('button');
    cancelButton.textContent = 'Cancel';
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
}

function sendDeleteRequest() {
    log('Sending delete request', 'sendDeleteRequest');

    fetch('/delete_account', {
        method: 'GET'
    })
        .then(async response => {
            if (response.ok) {
                log('Account deleted', 'sendDeleteRequest');

                let json = await response.json();

                if (json.success) {
                    displayCustomAlert('success', json.header, json.success, '', 5);
                    setTimeout(() => {
                        window.location.href = '/';
                    }, 5000);
                }
                else if (json.error) {
                    displayCustomAlert('error', json.header, json.error);
                }
            } else {
                error('Failed to delete account', 'sendDeleteRequest');
            }
        })
        .catch(err => {
            error('Error deleting account', 'sendDeleteRequest', err);
        });
}

function closeModal() {
    const modals = document.querySelectorAll('.alert-backdrop');
    modals.forEach(modal => modal.remove());
}