export interface NavItem {
  key: string;
  label: string;
  icon: string;
  badge?: number;
}

export type ToastType = 'success' | 'error' | 'warn' | 'info';

export interface ToastMessage {
  text: string;
  type: ToastType;
}
