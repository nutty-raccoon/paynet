import { locale, _, waitLocale } from 'svelte-i18n';
import { currentLanguage } from '../stores';

// Initialize i18n
import '../lib/i18n';

// Sync the currentLanguage store with svelte-i18n's locale
currentLanguage.subscribe((lang) => {
  if (lang) {
    locale.set(lang);
  }
});

// Sync svelte-i18n's locale changes back to our store
locale.subscribe((lang) => {
  if (lang && lang !== 'en' && lang !== 'es') {
    // If the locale is not supported, default to English
    currentLanguage.set('en');
  } else if (lang) {
    currentLanguage.set(lang);
  }
});

// Export the translation function and locale
export { _ as t, locale, waitLocale };