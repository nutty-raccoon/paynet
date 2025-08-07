// wallet-button.js - Web Component for wallet buttons

class WalletButton extends HTMLElement {
    // Static event name constants
    static get EVENTS() {
        return {
            CLICK: 'wallet-click'
        };
    }

    constructor() {
        super();
        this.attachShadow({ mode: 'open' });
        this.render();
    }

    static get observedAttributes() {
        return ['variant', 'disabled', 'loading'];
    }

    attributeChangedCallback(name, oldValue, newValue) {
        if (oldValue !== newValue) {
            this.render();
        }
    }

    get variant() {
        return this.getAttribute('variant') || 'primary';
    }

    get disabled() {
        return this.hasAttribute('disabled');
    }

    get loading() {
        return this.hasAttribute('loading');
    }

    render() {
        const styles = `
            <style>
                :host {
                    display: inline-block;
                }
                
                .wallet-btn {
                    margin-left: 10px;
                    padding: 8px 16px;
                    border: 0;
                    border-radius: 5px;
                    cursor: pointer;
                    font-size: 14px;
                    font-family: inherit;
                    font-weight: normal;
                    text-decoration: none;
                    outline: none;
                    box-shadow: none;
                    appearance: none;
                    display: inline-block;
                    vertical-align: baseline;
                    line-height: normal;
                    transition: background-color 0.2s ease;
                    min-width: 120px;
                    position: relative;
                    overflow: hidden;
                }

                .wallet-btn.primary {
                    background-color: #48bb78;
                    color: white;
                }

                .wallet-btn.primary:hover:not(:disabled) {
                    background-color: #38a169;
                }

                .wallet-btn.danger {
                    background-color: #dc3545;
                    color: white;
                }

                .wallet-btn.danger:hover:not(:disabled) {
                    background-color: #c82333;
                }

                .wallet-btn:disabled {
                    background-color: #a0aec0;
                    cursor: not-allowed;
                    opacity: 0.7;
                }

                .loading-spinner {
                    display: inline-block;
                    width: 14px;
                    height: 14px;
                    border: 2px solid transparent;
                    border-top: 2px solid currentColor;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                    margin-right: 8px;
                }

                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }

                .btn-content {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }

                :host([hidden]) {
                    display: none !important;
                }
            </style>
        `;

        const content = `
            <button class="wallet-btn ${this.variant}" ${this.disabled ? 'disabled' : ''}>
                <div class="btn-content">
                    ${this.loading ? '<span class="loading-spinner"></span>' : ''}
                    <slot></slot>
                </div>
            </button>
        `;

        this.shadowRoot.innerHTML = styles + content;

        // Forward click events
        const button = this.shadowRoot.querySelector('button');
        button.addEventListener('click', (e) => {
            if (!this.disabled && !this.loading) {
                this.dispatchEvent(new CustomEvent(WalletButton.EVENTS.CLICK, {
                    bubbles: true,
                    detail: { originalEvent: e }
                }));
            }
        });
    }

    // Public methods for external control
    setLoading(loading) {
        if (loading) {
            this.setAttribute('loading', '');
        } else {
            this.removeAttribute('loading');
        }
    }

    setDisabled(disabled) {
        if (disabled) {
            this.setAttribute('disabled', '');
        } else {
            this.removeAttribute('disabled');
        }
    }

    setVariant(variant) {
        this.setAttribute('variant', variant);
    }

    // Helper methods to maintain compatibility with existing code
    set textContent(text) {
        // Update the slot content
        this.innerHTML = text;
    }

    get textContent() {
        return this.innerHTML;
    }

    set disabled(value) {
        this.setDisabled(value);
    }

    set className(value) {
        // Map old className usage to variant
        if (value.includes('connected')) {
            this.setVariant('danger');
        } else {
            this.setVariant('primary');
        }
    }

    get className() {
        return `wallet-btn ${this.variant}`;
    }
}

// Register the custom element
customElements.define('wallet-button', WalletButton);

export { WalletButton };
