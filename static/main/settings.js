import { error, log, loadContent } from './main.js';

export function initializeSettings() {
    const startTime = performance.now();
    log('Initializing settings page', 'initializeSettings');

    // Attach click event listeners to flag images
    document.getElementById('german-flag').addEventListener('click', function () {
        log('German flag clicked', 'initializeSettings');
        setLanguage('de');
    });

    document.getElementById('english-flag').addEventListener('click', function () {
        log('English flag clicked', 'initializeSettings');
        setLanguage('en');
    });

    const endTime = performance.now();
    log(`Settings page initialized in ${endTime - startTime} ms`, 'initializeSettings');
}

// Function to set the language
function setLanguage(language) {
    log(`Setting language to ${language}`, 'setLanguage');
    fetch(`/user/set_language/${language}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        }
    })
        .then(response => {
            if (response.ok) {
                log(`Language set to ${language}`, 'setLanguage');
                window.location.reload();
            } else {
                error('Failed to set language', 'setLanguage');
            }
        })
        .catch(err => {
            error('Error setting language', 'setLanguage', err);
        });
}