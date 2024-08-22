document.getElementById('registrationForm').addEventListener('submit', handleSubmit);

async function handleSubmit(event) {
    event.preventDefault(); // Prevents the form from submitting the traditional way

    const form = document.getElementById('registrationForm');
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const errorMessage = document.getElementById('error');

    // Reset messages
    errorMessage.textContent = '';
    errorMessage.style.display = 'none';

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
        } else {
            result = {};
        }

        if (result.success) {
            // Redirect to login page with success message in the query string
            window.location.href = `/login?success=${encodeURIComponent(result.success)}`;
        } else if (result.error) {
            errorMessage.textContent = result.error;
            errorMessage.style.display = 'block';
        } else {
            throw new Error('Unexpected response format');
        }
    } catch (error) {
        console.error('Fetch error:', error);
        errorMessage.textContent = 'An unexpected error occurred. Please try again later.';
        errorMessage.style.display = 'block';
    } finally {
        submitButton.disabled = false;
    }

    return false; // Ensure the form does not submit traditionally
}
