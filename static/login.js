const urlParams = new URLSearchParams(window.location.search);
const successMessage = urlParams.get('success');

if (successMessage) {
    const successElement = document.createElement('p');
    successElement.className = 'success';
    successElement.textContent = successMessage;
    successElement.style.display = 'block';
    document.querySelector('.login-form').prepend(successElement);
}