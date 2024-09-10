import { log, error } from './main.js';

// Initialize the state from localStorage or default to 'English'
const state = {
    language: localStorage.getItem('language') || 'English',
};

// Function to get the current language from the state or localStorage
export function getGlobalLanguage() {
    return state.language;
}

// Function to set the global language and store it in localStorage
export function setGlobalLanguage(newLanguage) {
    state.language = newLanguage;
    localStorage.setItem('language', newLanguage);
}

export function formatDate(dateString) {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'N/A';

    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();

    return `${day}.${month}.${year}`;
}

export function displayCustomAlert(type, header_text, body_text, button_text = 'Close', countdown = 0) {
    log('Displaying custom alert:', 'displayCustomAlert', type, header_text, body_text, button_text, countdown);

    // Create the backdrop
    const backdrop = document.createElement('div');
    backdrop.className = 'alert-backdrop';

    // Create the alert container
    const alert = document.createElement('div');
    alert.className = `alert alert-${type}`;

    // Create the icon based on the type
    let icon;
    switch (type) {
        case 'error':
            icon = '❌';
            break;
        case 'info':
            icon = 'ℹ️';
            break;
        case 'success':
            icon = '✅';
            break;
        default:
            icon = 'ℹ️';
    }

    // Construct the HTML structure
    alert.innerHTML = `
        <div class="container-without-border">
            <div class="container-without-border-horizontally">
                <span class="alert-icon">${icon}</span>
                <div style="flex-grow: 1;" class="alert-header-text">
                    <strong>${header_text}</strong> <span class="alert-timer">${countdown > 0 ? `(${countdown}s)` : ''}</span>
                </div>
            </div>
            <p>${body_text}</p>
            ${button_text ? `<button class="alert-close" disabled>${button_text}</button>` : ''}
        </div>
    `;

    // Append the backdrop and the alert to the body
    document.body.appendChild(backdrop);
    document.body.appendChild(alert);

    // Timer logic
    if (countdown > 0) {
        const timerElement = alert.querySelector('.alert-timer');
        const closeButton = alert.querySelector('.alert-close');

        let timerInterval = setInterval(() => {
            countdown--;
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
            document.body.removeChild(alert);
            document.body.removeChild(backdrop);
        });
    }
}

export async function parseJsonResponse(response) {
    try {
        return await response.json();
    } catch {
        error('Error parsing JSON response:', 'parseJsonResponse', response);
        throw new Error('Error parsing JSON response');
    }
}