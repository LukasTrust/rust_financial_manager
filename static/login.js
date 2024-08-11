const urlParams = new URLSearchParams(window.location.search);
const successMessage = urlParams.get('success');

if (successMessage) {
    const successElement = document.getElementById('success');
    successElement.textContent = successMessage;
    successElement.style.display = 'block';
}

document.getElementById('loginForm').addEventListener('submit', handleSubmit);

async function handleSubmit(event) {
    event.preventDefault(); // Prevents the form from submitting the traditional way

    const form = document.getElementById('loginForm');
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const errorDiv = document.getElementById('error');

    // Reset messages
    errorDiv.textContent = '';
    errorDiv.style.display = 'none';

    try {
        submitButton.disabled = true;

        const response = await fetch(form.action, {
            method: 'POST',
            body: formData,
            headers: {
                'Accept': 'application/json',
            },
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        let result;
        const contentType = response.headers.get('Content-Type');
        if (contentType && contentType.includes('application/json')) {
            result = await response.json();
            console.log('Response body:', result);
        } else {
            result = {};
        }

        if (result.success) {
            // Redirect to login page with success message in the query string
            window.location.href = `/base`;
        } else if (result.error) {
            errorDiv.textContent = result.error;
            errorDiv.style.display = 'block';
            const successDiv = urlParams.get('success');
            successDiv.style.display = 'none';
        } else {
            throw new Error('Unexpected response format');
        }
    } catch (error) {
        console.error('Fetch error:', error);
        errorDiv.textContent = 'An unexpected error occurred. Please try again later.';
        errorDiv.style.display = 'block';
    } finally {
        submitButton.disabled = false;
    }

    return false; // Ensure the form does not submit traditionally
}
