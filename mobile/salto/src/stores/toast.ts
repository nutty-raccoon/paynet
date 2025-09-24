import { writable, get } from 'svelte/store';
import { showErrorDetail } from '../stores';
import type { ToastMessage, ToastConfig, ToastType } from '../types/toast';

// Store for managing active toast messages
export const toasts = writable<ToastMessage[]>([]);

// Default durations for different toast types (in milliseconds)
const DEFAULT_DURATIONS = {
  error: 7000,
  success: 4000,
  warning: 5000,
  info: 4000
} as const;

// Generate unique ID for each toast
function generateToastId(): string {
  return `toast-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

// Add a new toast message
export function addToast(config: ToastConfig, error?: any): string {
  const id = generateToastId();
  const duration = config.duration ?? DEFAULT_DURATIONS[config.type];
  
  // Format message with error details if showErrorDetail is true and error is provided
  let finalMessage = config.message;
  if (error && get(showErrorDetail)) {
    const errorText = typeof error === 'string' ? error :
                     error?.message || error?.toString() || 'Unknown error';
    finalMessage = `${config.message}\n\nError details: ${errorText}`;
  }
  
  const toast: ToastMessage = {
    id,
    type: config.type,
    message: finalMessage,
    duration,
    timestamp: Date.now()
  };

  // Add toast to store
  toasts.update(currentToasts => [toast, ...currentToasts]);

  // Set up auto-dismiss if duration is positive
  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }

  return id;
}

// Remove a specific toast by ID
export function removeToast(id: string): void {
  toasts.update(currentToasts =>
    currentToasts.filter(toast => toast.id !== id)
  );
}

// Clear all toasts
export function clearAllToasts(): void {
  toasts.set([]);
}

// Convenience functions for different toast types
export function showErrorToast(message: string, error?: any, duration?: number): string {
  return addToast({
    type: 'error' as ToastType,
    message,
    duration
  }, error);
}

export function showSuccessToast(message: string, error?: any, duration?: number): string {
  return addToast({
    type: 'success' as ToastType,
    message,
    duration
  }, error);
}

export function showWarningToast(message: string, error?: any, duration?: number): string {
  return addToast({
    type: 'warning' as ToastType,
    message,
    duration
  }, error);
}

export function showInfoToast(message: string, error?: any, duration?: number): string {
  return addToast({
    type: 'info' as ToastType,
    message,
    duration
  }, error);
}