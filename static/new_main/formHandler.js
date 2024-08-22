import { error, log } from './main.js';
import { displayCustomAlert } from './utils.js';

// Main form submission handler
async function handleFormSubmission(form) {
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

            log('Form submission result:', 'handleFormSubmission', result);

            if (result.success) {
                displayCustomAlert('success', result.header, result.success, 'Close');
            } else if (result.error) {
                displayCustomAlert('error', result.header, result.error, 'Close');
            }

            if (!result.error) form.reset();
        } catch (err) {
            error('Error submitting form:', 'handleFormSubmission', err);
        }
    });
}

// Function to initialize form handling
export function initializeFormHandling() {
    const forms = document.querySelectorAll('form');

    forms.forEach(form => {
        handleFormSubmission(form);
    });
}

async function parseJsonResponse(response) {
    try {
        return await response.json();
    } catch {
        throw new Error('Error parsing JSON response');
    }
}