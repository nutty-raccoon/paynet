export enum ToastType {
  ERROR = 'error',
  SUCCESS = 'success',
  WARNING = 'warning',
  INFO = 'info'
}

export interface ToastMessage {
  id: string;
  type: ToastType;
  message: string;
  duration?: number; // milliseconds
  timestamp: number;
}

export interface ToastConfig {
  type: ToastType;
  message: string;
  duration?: number;
}