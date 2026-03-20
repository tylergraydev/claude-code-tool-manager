import { en } from './locales/en';
import { zhTW } from './locales/zh-TW';
import type { Locale, LocaleInfo, TranslationKey, Translations } from './types';

const locales: Record<Locale, Translations> = {
	en,
	'zh-TW': zhTW
};

const localeInfos: LocaleInfo[] = [
	{ code: 'en', label: 'EN' },
	{ code: 'zh-TW', label: '中文' }
];

function detectLocale(): Locale {
	if (typeof navigator === 'undefined') return 'en';
	for (const lang of navigator.languages ?? [navigator.language]) {
		if (lang in locales) return lang as Locale;
		const prefix = lang.split('-')[0];
		for (const code of Object.keys(locales)) {
			if (code.split('-')[0] === prefix) return code as Locale;
		}
	}
	return 'en';
}

function getInitialLocale(): Locale {
	try {
		const stored = localStorage.getItem('locale');
		if (stored && stored in locales) {
			return stored as Locale;
		}
	} catch {
		// localStorage unavailable
	}
	return detectLocale();
}

class I18nState {
	locale = $state<Locale>(getInitialLocale());

	constructor() {
		if (typeof document !== 'undefined') {
			document.documentElement.lang = this.locale;
		}
	}

	t(key: TranslationKey, params?: Record<string, string | number>): string {
		let value = locales[this.locale]?.[key] ?? locales.en[key] ?? key;
		if (params) {
			for (const [k, v] of Object.entries(params)) {
				value = value.replaceAll(`{${k}}`, String(v));
			}
		}
		return value;
	}

	setLocale(locale: Locale) {
		this.locale = locale;
		try {
			localStorage.setItem('locale', locale);
		} catch {
			// localStorage unavailable
		}
		if (typeof document !== 'undefined') {
			document.documentElement.lang = locale;
		}
	}

	get availableLocales(): LocaleInfo[] {
		return localeInfos;
	}

	get currentLabel(): string {
		return localeInfos.find((l) => l.code === this.locale)?.label ?? 'EN';
	}

	get nextLocale(): Locale {
		const idx = localeInfos.findIndex((l) => l.code === this.locale);
		return localeInfos[(idx + 1) % localeInfos.length].code;
	}
}

export const i18n = new I18nState();
