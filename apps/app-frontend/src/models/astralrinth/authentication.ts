import { ElyByIcon, ExternalIcon } from '@modrinth/assets'
import { invoke } from '@tauri-apps/api/core'
import type { Component } from 'vue'
import { ref } from 'vue'

export type ExternalAuthProvider = {
	id: string
	accountOptionId: string
	icon: Component
	name: string
	skinManagementUrl?: string
}

export type MinecraftCredential = {
	account_type?: string
	profile: {
		id: string
		name: string
	}
}

type ExternalAuthenticationOptions = {
	onAuthenticated: (credentials: MinecraftCredential) => Promise<void>
	onError: (error: unknown) => void
}

type ExternalAuthProviderMetadata = {
	id: string
	displayName: string
	icon: string
	skinManagementUrl?: string
}

const externalAuthProviderIcons: Record<string, Component> = {
	elyby: ElyByIcon,
}

export const externalAuthProviders = ref<ExternalAuthProvider[]>([])

let providerLoad: Promise<ExternalAuthProvider[]> | undefined

/** Loads provider metadata once and adapts it for account-selection controls. */
export async function loadExternalAuthProviders() {
	if (externalAuthProviders.value.length > 0) {
		return externalAuthProviders.value
	}

	const load =
		providerLoad ??
		invoke<ExternalAuthProviderMetadata[]>('plugin:auth|get_external_auth_providers').then(
			(providers) => {
				externalAuthProviders.value = providers.map((provider) => ({
					id: provider.id,
					accountOptionId: `add_external_${provider.id}_account`,
					icon: externalAuthProviderIcons[provider.icon] ?? ExternalIcon,
					name: provider.displayName,
					skinManagementUrl: provider.skinManagementUrl,
				}))

				return externalAuthProviders.value
			},
		)
	providerLoad = load

	try {
		return await load
	} finally {
		if (providerLoad === load) {
			providerLoad = undefined
		}
	}
}

/** Finds the UI metadata for a stored external account type. */
export function getExternalAuthProvider(accountType?: string) {
	return externalAuthProviders.value.find((provider) => provider.id === accountType)
}

/** Starts the native OAuth flow for one external authentication provider. */
export async function authenticateExternalProvider(
	provider: string,
): Promise<MinecraftCredential | null> {
	return await invoke('plugin:auth|authenticate_external_provider', { provider })
}

/** Coordinates an external sign-in attempt and exposes its shared loading state. */
export function useExternalAuthentication(options: ExternalAuthenticationOptions) {
	const disabled = ref(false)

	/** Authenticates the selected provider while preventing parallel sign-in windows. */
	async function authenticate(selectedProvider: ExternalAuthProvider) {
		if (disabled.value) {
			return
		}

		disabled.value = true

		try {
			const credentials = await authenticateExternalProvider(selectedProvider.id)
			if (credentials) {
				await options.onAuthenticated(credentials)
			}
		} catch (error) {
			options.onError(error)
		} finally {
			disabled.value = false
		}
	}

	return {
		authenticate,
		disabled,
	}
}
