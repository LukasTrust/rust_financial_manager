// Define the function to load content dynamically
function loadContent(url) {
    document.getElementById('main-content').innerHTML = '<p>Loading...</p>'; // Show loading state
    fetch(url)
        .then(response => response.text())
        .then(html => {
            document.getElementById('main-content').innerHTML = html;
            if (url === '/add-bank') {
                add_bank_form();
            }
        })
        .catch(error => {
            console.error('Error loading content:', error);
            document.getElementById('main-content').innerHTML = '<p>Error loading content.</p>';
        });
}

// Define the function to initialize form handling
function add_bank_form() {
    const form = document.getElementById('add_bankForm');
    if (form) {
        console.log('Form found');
        form.addEventListener('submit', async function (event) {
            event.preventDefault();

            const formData = new FormData(form);

            try {
                const response = await fetch(form.action, {
                    method: form.method,
                    body: formData
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! Status: ${response.status}`);
                }

                let result;
                try {
                    result = await response.json();
                    console.log('Parsed JSON result:', result);
                } catch (jsonError) {
                    throw new Error('Error parsing JSON response');
                }

                const errorDiv = document.getElementById('error');
                const successDiv = document.getElementById('success');

                if (result.success) {
                    successDiv.textContent = result.success;
                    successDiv.style.display = 'block';
                }
                else if (result.error) {
                    errorDiv.textContent = result.error;
                    errorDiv.style.display = 'block';
                }

                if (!errorMessage) {
                    form.reset();
                }
            } catch (error) {
                console.error('An unexpected error occurred:', error);
                document.getElementById("error").textContent = `An unexpected error occurred: ${error.message}`;
            }
        });
    } else {
        console.warn('Form not found');
    }
}

// Initialize event listeners when DOM content is loaded
document.addEventListener("DOMContentLoaded", function () {
    console.log('DOM fully loaded and parsed');

    // Attach event listeners to sidebar buttons
    document.querySelectorAll(".sidebar-left button").forEach(button => {
        button.addEventListener("click", function () {
            const url = this.getAttribute("onclick").match(/'([^']+)'/)[1];
            loadContent(url);
        });
    });

    // Optionally load initial content
    loadContent('/dashboard'); // Or another default content URL
});
