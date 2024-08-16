import { log } from './logger.js';
import { loadContent } from './contentLoader.js';

document.addEventListener("DOMContentLoaded", function () {
    log('DOM content loaded. Initializing sidebar buttons and loading default content:', 'DOMContentLoaded');
    document.querySelectorAll(".sidebar-left button").forEach(button => {
        button.addEventListener("click", function () {
            const url = this.getAttribute("onclick").match(/'([^']+)'/)[1];
            log('Sidebar button clicked. Loading content from URL:', 'DOMContentLoaded', url);
            loadContent(url);
        });
    });

    loadContent('/dashboard');
});