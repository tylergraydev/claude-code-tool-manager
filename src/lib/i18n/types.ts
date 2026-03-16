import type { en } from './locales/en';

export type TranslationKey = keyof typeof en;
export type Translations = Record<TranslationKey, string>;
export type Locale = 'en' | 'zh-TW';

export interface LocaleInfo {
	code: Locale;
	label: string;
}
