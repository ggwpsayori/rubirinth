import {
	buildLocaleMessages,
	createMessageCompiler,
	type CrowdinMessages,
	LOCALES,
} from '@modrinth/ui'
import { uiLocaleModulesEager } from '@modrinth/ui/src/locales.eager.ts'
import { createI18n } from 'vue-i18n'

const localeModules = import.meta.glob<{ default: CrowdinMessages }>('./locales/*/index.json', {
	eager: true,
})

const i18n = createI18n({
	legacy: false,
	locale: 'ru-RU',
	fallbackLocale: 'en-US',
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: buildLocaleMessages(localeModules, uiLocaleModulesEager),
})

export async function setLocale(requestedLocale: string): Promise<void> {
	const locale = LOCALES.some((candidate) => candidate.code === requestedLocale)
		? requestedLocale
		: 'en-US'
	i18n.global.locale.value = locale
}

export default i18n
