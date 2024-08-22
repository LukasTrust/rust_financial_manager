export function formatDate(dateString) {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'N/A';

    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();

    return `${day}.${month}.${year}`;
}

export function displayCustomAlert(type, header_text, body_text, button_text) {
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
                <div style="flex-grow: 1;">
                    <strong>${header_text}</strong>
                </div>
            </div>
            <p>${body_text}</p>
            <button class="alert-close">${button_text}</button>
        </div>
    `;

    // Append the backdrop and the alert to the body
    document.body.appendChild(backdrop);
    document.body.appendChild(alert);

    // Add click event to the close button
    const closeButton = alert.querySelector('.alert-close');
    closeButton.addEventListener('click', () => {
        document.body.removeChild(alert);
        document.body.removeChild(backdrop); // Remove the backdrop when the alert is closed
    });
}

export async function parseJsonResponse(response) {
    try {
        return await response.json();
    } catch {
        throw new Error('Error parsing JSON response');
    }
}