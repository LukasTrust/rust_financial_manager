import { log, error } from './logger.js';
import { initializeChartAndDatePicker } from './chartManager.js';
import { update_performance } from './performanceUpdater.js';
import { loadContent } from './contentLoader.js';

export async function handleFormSubmission(form) {
    form.addEventListener('submit', async function (event) {
        event.preventDefault();

        const formData = new FormData(form);
        const errorDiv = document.getElementById('error');
        const successDiv = document.getElementById('success');

        try {
            const response = await fetch(form.action, {
                method: form.method,
                body: formData
            });

            if (!response.ok) throw new Error(`HTTP error! Status: ${response.status}`);

            let result;
            try {
                result = await response.json();
            } catch (jsonError) {
                throw new Error('Error parsing JSON response');
            }

            if (result.success) {
                successDiv.textContent = result.success;
                successDiv.style.display = 'block';
                errorDiv.style.display = 'none';

                if (result.graph_data) {
                    window.plotData = JSON.parse(result.graph_data);
                    log('Form submitted successfully. Reinitializing chart with new data:', 'handleFormSubmission', window.plotData);
                    initializeChartAndDatePicker();
                }

                if (result.performance_value) {
                    update_performance(result);
                }

                if (result.banks) {
                    log('Updating bank list:', 'handleFormSubmission', result.banks);
                    const banksContainer = document.getElementById('banks');

                    if (banksContainer) {
                        const newBankIds = new Set(result.banks.map(bank => bank.id));

                        Array.from(banksContainer.children).forEach(button => {
                            const bankId = button.getAttribute('data-bank-id');
                            if (!newBankIds.has(bankId)) {
                                banksContainer.removeChild(button);
                            }
                        });

                        result.banks.forEach(bank => {
                            let bankButton = banksContainer.querySelector(`button[data-bank-id="${bank.id}"]`);

                            if (!bankButton) {
                                bankButton = document.createElement('button');
                                bankButton.setAttribute('data-bank-id', bank.id);
                                bankButton.setAttribute('data-url', `/bank/${bank.id}`);
                                bankButton.setAttribute('style', 'width: 100%');
                                bankButton.textContent = bank.name;

                                bankButton.addEventListener("click", function () {
                                    const url = this.getAttribute("data-url");
                                    log('Bank button clicked. Loading content from URL:', 'handleFormSubmission', url);
                                    loadContent(url);
                                });

                                banksContainer.appendChild(bankButton);
                            } else {
                                bankButton.textContent = bank.name;
                            }
                        });

                        log('Bank list updated.', 'handleFormSubmission');
                    }
                }
            } else if (result.error) {
                error('Form submission error:', 'handleFormSubmission', result.error);
                errorDiv.textContent = result.error;
                errorDiv.style.display = 'block';
                successDiv.style.display = 'none';
            }

            if (!result.error) form.reset();
        } catch (err) {
            error('An unexpected error occurred:', 'handleFormSubmission', err);
            errorDiv.textContent = `An unexpected error occurred: ${err.message}`;
            errorDiv.style.display = 'block';
            successDiv.style.display = 'none';
        }
    });
}

export function initializeFormHandling() {
    log('Initializing form handling for all forms:', 'initializeFormHandling');
    const forms = document.querySelectorAll('form');

    forms.forEach(form => {
        if (form.id !== 'logout-form') {
            handleFormSubmission(form);
        }
    });
}
