document.addEventListener("DOMContentLoaded", function () {
    // Function to load content into the main-content div
    function loadContent(url) {
        fetch(url)
            .then(response => {
                if (!response.ok) {
                    throw new Error('Network response was not ok');
                }
                return response.text();
            })
            .then(html => {
                document.getElementById('main-content').innerHTML = html;
                // Optionally, trigger any additional scripts needed after loading new content
                executeScripts();
            })
            .catch(error => {
                console.error('Error loading content:', error);
                document.getElementById('main-content').innerHTML = '<p>Error loading content. Please try again.</p>';
            });
    }

    // Attach event listeners to sidebar buttons
    document.querySelectorAll(".sidebar-left button").forEach(button => {
        button.addEventListener("click", function () {
            var url = this.getAttribute("onclick").match(/'([^']+)'/)[1];
            loadContent(url);
        });
    });

    // Load initial content (dashboard) as default
    loadContent('/dashboard');

    // Function to execute any additional scripts after loading content
    function executeScripts() {
        // Check if the loaded content includes a specific script to execute
        if (typeof initializeDashboard === "function") {
            initializeDashboard();  // Call the dashboard-specific initialization
        }
    }
});
