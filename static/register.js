document.getElementById('registrationForm').addEventListener('submit', handleSubmit);

async function handleSubmit(event) {
    event.preventDefault(); // Prevents the form from submitting the traditional way

    const form = document.getElementById('registrationForm');
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const successMessage = document.getElementById('success');
    const errorMessage = document.getElementById('error');

    try {
        submitButton.disabled = true;

        const response = await fetch(form.action, {
            method: 'POST',
            body: formData,
            headers: {
                'Accept': 'application/json',
            },
        });

        let result;
        const contentType = response.headers.get('Content-Type');
        if (contentType && contentType.includes('application/json')) {
            result = await response.json();
            console.log('Response body:', result);
        } else {
            result = {};
        }

        if (response.ok && result.success) {
            successMessage.textContent = result.success;
            successMessage.style.display = 'block';
            errorMessage.textContent = '';
            errorMessage.style.display = 'none';
        } else if (result.error) {
            errorMessage.textContent = result.error;
            errorMessage.style.display = 'block';
            successMessage.textContent = '';
            successMessage.style.display = 'none';
        } else {
            errorMessage.textContent = 'An unexpected error occurred. Please try again later.';
            errorMessage.style.display = 'block';
            successMessage.textContent = '';
            successMessage.style.display = 'none';
        }
    } catch (error) {
        console.error('Fetch error:', error);
        errorMessage.textContent = 'An unexpected error occurred. Please try again later.';
        errorMessage.style.display = 'block';
        successMessage.textContent = '';
        successMessage.style.display = 'none';
    } finally {
        submitButton.disabled = false;
    }

    return false; // Ensure the form does not submit traditionally
}
