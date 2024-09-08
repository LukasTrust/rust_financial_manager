import { error, log, state, setGloableLanguage, loadContent } from './main.js';
import { displayCustomAlert } from './utils.js';

export function initializeSettings() {
    const startTime = performance.now();
    log('Initializing settings page', 'initializeSettings');

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