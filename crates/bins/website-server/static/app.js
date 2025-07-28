// Simple JavaScript for the Route Handler

document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 Route Handler loaded!');
    
    // Add copy functionality to code blocks
    const codeElements = document.querySelectorAll('code, pre');
    codeElements.forEach(function(element) {
        element.addEventListener('click', function() {
            navigator.clipboard.writeText(element.textContent).then(function() {
                showToast('Copied to clipboard!', 'success');
            }).catch(function() {
                console.log('Failed to copy to clipboard');
            });
        });
        
        // Add visual indication that it's clickable
        element.style.cursor = 'pointer';
        element.title = 'Click to copy';
    });
});

// Toast notification system
function showToast(message, type = 'success') {
    // Remove existing toasts
    const existingToasts = document.querySelectorAll('.toast');
    existingToasts.forEach(toast => toast.remove());
    
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    
    // Add toast styles
    Object.assign(toast.style, {
        position: 'fixed',
        top: '20px',
        right: '20px',
        padding: '12px 24px',
        borderRadius: '8px',
        color: 'white',
        fontWeight: '500',
        zIndex: '1000',
        transform: 'translateX(100%)',
        transition: 'transform 0.3s ease',
        backgroundColor: type === 'success' ? '#28a745' : '#dc3545'
    });
    
    document.body.appendChild(toast);
    
    // Slide in
    setTimeout(() => {
        toast.style.transform = 'translateX(0)';
    }, 10);
    
    // Slide out and remove
    setTimeout(() => {
        toast.style.transform = 'translateX(100%)';
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}