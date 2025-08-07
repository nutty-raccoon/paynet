// Import Web Components
import { WalletButton } from './wallet-button.js';

// Core functionality - always loaded
class ToastManager {
    static show(message, type = 'success') {
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
}

// Clipboard functionality - always available
function initializeClipboard() {
    const codeElements = document.querySelectorAll('code, pre');
    codeElements.forEach(function(element) {
        element.addEventListener('click', function() {
            navigator.clipboard.writeText(element.textContent).then(function() {
                ToastManager.show('Copied to clipboard!', 'success');
            }).catch(function() {
                console.log('Failed to copy to clipboard');
                ToastManager.show('Failed to copy to clipboard', 'error');
            });
        });
        
        // Add visual indication that it's clickable
        element.style.cursor = 'pointer';
        element.title = 'Click to copy';
    });
}

// Starknet functionality - loaded on demand
let starknetModule = null;
let starknetCore = null;

async function loadStarknetWallet() {
    if (starknetModule) {
        return starknetModule;
    }
    
    try {
        console.log('Loading Starknet module...');
        starknetModule = await import(/* webpackChunkName: "starknet" */ '@starknet-io/get-starknet');
        console.log('Starknet module loaded successfully');
        return starknetModule;
    } catch (error) {
        console.error('Failed to load Starknet module:', error);
        throw new Error(`Failed to load Starknet library: ${error.message}`);
    }
}

async function loadStarknetCore() {
    if (starknetCore) {
        return starknetCore;
    }
    
    try {
        console.log('Loading Starknet core module...');
        starknetCore = await import(/* webpackChunkName: "starknet-core" */ 'starknet');
        console.log('Starknet core module loaded successfully');
        return starknetCore;
    } catch (error) {
        console.error('Failed to load Starknet core module:', error);
        throw new Error(`Failed to load Starknet core library: ${error.message}`);
    }
}

let addSelfDestructingEventListener = (element, eventType, callback) => {
    let handler = () => {
        callback();
        element.removeEventListener(eventType, handler);
    };
    element.addEventListener(eventType, handler);
};

async function initializeStarknetFeatures() {
    const walletStatus = document.getElementById('wallet-status');
    const connectButton = document.getElementById('wallet-connect-btn');
    const depositButton = document.getElementById('deposit-btn');
    
    // Get deposit data from embedded JSON script tag
    const depositDataScript = document.getElementById('deposit-data');
    if (!depositDataScript) {
        console.error('No deposit data found - missing script tag');
        walletStatus.textContent = 'Error: Missing deposit configuration';
        return;
    }
    
    let depositData;
    try {
        depositData = JSON.parse(depositDataScript.textContent);
        console.log('Loaded deposit data:', depositData);
    } catch (error) {
        console.error('Failed to parse deposit data:', error);
        walletStatus.textContent = 'Error: Invalid deposit configuration';
        return;
    }
    
    if (!walletStatus || !connectButton || !depositButton) {
        console.log('No wallet elements found - skipping Starknet initialization');
        return;
    }
    
    try {
        walletStatus.textContent = 'Loading Starknet library...';
        
        const { connect, disconnect } = await loadStarknetWallet();
        const { WalletAccount, Contract } = await loadStarknetCore();
        
        walletStatus.textContent = 'Ready to connect wallet';
        connectButton.setDisabled(false);
        const handleConnect = async () => {
            try {
                connectButton.setLoading(true);
                connectButton.textContent = 'Connecting...';
                
                const selectedWallet = await connect({ modalMode: 'alwaysAsk' });
                const myFrontendProviderUrl = depositData.provider_url;
                const myWalletAccount = await WalletAccount.connect(
                  { nodeUrl: myFrontendProviderUrl },
                  selectedWallet
                );
                // connect the contracts
                const assetContract = new Contract(depositData.asset_contract.abi, depositData.asset_contract.address, myWalletAccount.walletProvider);
                const invoiceContract = new Contract(depositData.invoice_contract.abi, depositData.invoice_contract.address, myWalletAccount.walletProvider);
                // generate the calldatas
                const claimCall = assetContract.populate('approve', {
                  spender: depositData.invoice_contract.address,
                  amount: {
                      low: depositData.amount_low,
                      high: depositData.amount_high,
                    }
                });
                const payInvoiceCall = invoiceContract.populate('pay_invoice', {
                    quote_id_hash: depositData.quote_id_hash,
                    expiry: depositData.expiry, 
                    asset: depositData.asset_contract.address, 
                      amount: {
                      low: depositData.amount_low,
                      high: depositData.amount_high,
                    },
                    payee: depositData.payee
                });

                
                walletStatus.textContent = `Connected to ${selectedWallet.name || 'wallet'}`;
                connectButton.setLoading(false);
                connectButton.textContent = 'Disconnect';
                connectButton.setVariant('danger');
                // Show deposit button
                depositButton.hidden = false;
                
                ToastManager.show('Wallet connected successfully!', 'success');
                
                const handleDeposit = async () => {
                    try {
                        depositButton.setLoading(true);
                        depositButton.textContent = 'Depositing...';
                        
                        const calls = [claimCall, payInvoiceCall];
                        console.log('Executing deposit with payload:', calls);
                        const resp = await myWalletAccount.execute(calls);
                        console.log('Deposit response:', resp);
                        
                        ToastManager.show('Deposit transaction submitted!', 'success');
                        depositButton.setLoading(false);
                        depositButton.textContent = 'Deposit';
                        
                    } catch (error) {
                        console.error('Deposit error:', error);
                        ToastManager.show(`Deposit failed: ${error.message}`, 'error');
                        depositButton.setLoading(false);
                        depositButton.textContent = 'Deposit';
                    }
                }
                // Set up deposit button functionality
                depositButton.addEventListener(WalletButton.EVENTS.CLICK, handleDeposit);
                
                // Change button to disconnect
                addSelfDestructingEventListener(connectButton, WalletButton.EVENTS.CLICK, async () => {
                    try {
                        console.log("in disconnect");
                        await disconnect();
                        depositButton.hidden = true;
                        depositButton.removeEventListener(WalletButton.EVENTS.CLICK, handleDeposit);
                        walletStatus.textContent = 'Disconnected';
                        connectButton.textContent = 'Connect Wallet';
                        connectButton.setVariant('primary');
                        ToastManager.show('Wallet disconnected', 'success');
                        addSelfDestructingEventListener(connectButton, WalletButton.EVENTS.CLICK, handleConnect);
                    } catch (error) {
                        console.error('Disconnect error:', error);
                        ToastManager.show('Failed to disconnect wallet', 'error');
                    }
                });
            } catch (error) {
                console.error('Wallet connection error:', error);
                connectButton.setLoading(false);
                connectButton.textContent = 'Connect Wallet';
                connectButton.setVariant('primary');
                ToastManager.show('Failed to connect wallet', 'error');
            }
        };

        addSelfDestructingEventListener(connectButton, WalletButton.EVENTS.CLICK, handleConnect);
        
    } catch (error) {
        console.error('Starknet initialization error:', error);
        walletStatus.textContent = `Error: ${error.message}`;
        connectButton.setDisabled(true);
        ToastManager.show('Failed to initialize Starknet features', 'error');
    }
}

// Global exports for HTML pages to use
window.loadStarknetWallet = loadStarknetWallet;
window.loadStarknetCore = loadStarknetCore;
window.initializeStarknetFeatures = initializeStarknetFeatures;
window.ToastManager = ToastManager;

// Initialize core functionality on DOM ready
document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 Paynet frontend loaded!');
    initializeClipboard();
    
    // Auto-initialize Starknet features if we're on a Starknet page
    if (document.getElementById('wallet-status')) {
        initializeStarknetFeatures();
    }
});
