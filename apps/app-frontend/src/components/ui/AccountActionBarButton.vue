<script setup lang="ts">
import {
	DropdownIcon,
	MicrosoftIcon,
	OfflineIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	IconButton,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { Dropdown } from 'floating-vue'
import type { Ref } from 'vue'
import { computed, onMounted, ref } from 'vue'

import AccountsErrorModals from '@/components/ui/astralrinth/accounts/error/AccountsErrorModals.vue'
import AccountsInputModals from '@/components/ui/astralrinth/accounts/input/AccountsInputModals.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	login as login_flow,
	offline_login,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer.ts'
import type { Skin } from '@/helpers/skins'
import { get_available_skins } from '@/helpers/skins'
import {
	externalAuthProviders,
	getExternalAuthProvider,
	loadExternalAuthProviders,
	type MinecraftCredential,
	useExternalAuthentication,
} from '@/models/astralrinth/authentication'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const emit = defineEmits<{
	change: []
}>()

type AccountsInputModalsHandle = {
	hideAuth: () => void
	hideOffline: () => void
	showAuth: () => void
	showOffline: () => void
}

type AccountsErrorModalsHandle = {
	hideInputOfflineError: () => void
	showInputOfflineError: () => void
	showUnexpectedError: () => void
}

const offlineLoginCooldownMs = 1000
const minOfflinePlayerNameLength = 3
const maxOfflinePlayerNameLength = 20
const nameExp = 'a-zA-Z0-9_'
const nameRegex = new RegExp('^[' + nameExp + ']+$')

const accounts: Ref<MinecraftCredential[]> = ref([])
const loginDisabled = ref(false)
const offlineLoginDisabled = ref(false)
const defaultUser = ref<string | undefined>()
const equippedSkin = ref<Skin | null>(null)
const headUrlCache = ref(new Map<string, string>())
const showDropdown = ref(false)

const accountsInputModals = ref<AccountsInputModalsHandle | null>(null)
const accountsErrorModals = ref<AccountsErrorModalsHandle | null>(null)

const offlinePlayerName = ref('')
const { authenticate: addExternalProfile, disabled: externalAuthDisabled } =
	useExternalAuthentication({
		onAuthenticated: async (credentials) => {
			await setAccount(credentials)
			accountsInputModals.value?.hideAuth()
		},
		onError: (error) => {
			handleError(error)
			accountsErrorModals.value?.showUnexpectedError()
		},
	})

function getAccountType(account?: MinecraftCredential) {
	switch (account?.account_type) {
		case 'microsoft':
			return MicrosoftIcon
		case 'offline':
			return OfflineIcon
		default:
			return getExternalAuthProvider(account?.account_type)?.icon ?? null
	}
}

function showOfflineLoginModal() {
	accountsInputModals.value?.showOffline()
}

function showAccountLoginModal() {
	accountsInputModals.value?.showAuth()
}

function retryAddOfflineProfile() {
	accountsErrorModals.value?.hideInputOfflineError()
	offlineLoginDisabled.value = false
	clearOfflineFields()
	showOfflineLoginModal()
}

function clearOfflineFields() {
	offlinePlayerName.value = ''
}

async function addOfflineProfile() {
	if (offlineLoginDisabled.value) {
		return
	}

	offlineLoginDisabled.value = true

	const name = offlinePlayerName.value.trim()
	const isValidName =
		nameRegex.test(name) &&
		name.length >= minOfflinePlayerNameLength &&
		name.length <= maxOfflinePlayerNameLength

	if (!isValidName) {
		accountsInputModals.value?.hideOffline()
		accountsErrorModals.value?.showInputOfflineError()
		offlineLoginDisabled.value = false
		clearOfflineFields()
		return
	}

	try {
		const result = await offline_login(name)
		accountsInputModals.value?.hideOffline()

		if (result) {
			await setAccount(result)
			await refreshValues()
		} else {
			accountsErrorModals.value?.showUnexpectedError()
		}
	} catch (error) {
		handleError(error)
		accountsErrorModals.value?.showUnexpectedError()
	} finally {
		clearOfflineFields()
		window.setTimeout(() => {
			offlineLoginDisabled.value = false
		}, offlineLoginCooldownMs)
	}
}

async function refreshValues() {
	try {
		defaultUser.value = await get_default_user().catch(() => undefined)
		const userList = await users().catch(() => [])
		accounts.value = Array.isArray(userList) ? [...userList] : []
		accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))

		if (selectedAccount.value && (selectedAccount.value.account_type === 'microsoft' || !selectedAccount.value.account_type)) {
			try {
				const skins = await get_available_skins()
				equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null

				if (equippedSkin.value) {
					try {
						const headUrl = await getPlayerHeadUrl(equippedSkin.value)
						headUrlCache.value = new Map(headUrlCache.value).set(
							equippedSkin.value.texture_key,
							headUrl,
						)
					} catch (error) {
						console.warn('Failed to get head render for equipped skin:', error)
					}
				}
			} catch {
				equippedSkin.value = null
			}
		} else {
			equippedSkin.value = null
		}
	} catch (e) {
		console.error('Failed to refresh accounts:', e)
	}
}

async function setEquippedSkin(skin: Skin) {
	equippedSkin.value = skin

	try {
		const headUrl = await getPlayerHeadUrl(skin)
		headUrlCache.value = new Map(headUrlCache.value).set(skin.texture_key, headUrl)
	} catch (error) {
		console.warn('Failed to get head render for equipped skin:', error)
	}
}

defineExpose({
	refreshValues,
	setEquippedSkin,
	showAccountLoginModal,
})

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

const avatarUrl = computed(() => {
	if (equippedSkin.value?.texture_key) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}

		return 'https://mc-heads.net/avatar/' + equippedSkin.value.texture_key + '/128'
	}

	if (selectedAccount.value?.profile?.id) {
		return 'https://mc-heads.net/avatar/' + selectedAccount.value.profile.id + '/128'
	}

	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function getAccountAvatarUrl(account: MinecraftCredential) {
	if (
		account.profile.id === selectedAccount.value?.profile?.id &&
		equippedSkin.value?.texture_key
	) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
	}

	return 'https://mc-heads.net/avatar/' + account.profile.id + '/128'
}

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
		accountsInputModals.value?.hideAuth()
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

async function logout(id: string) {
	await remove_user(id).catch(handleError)
	await refreshValues()

	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	} else {
		emit('change')
	}

	trackEvent('AccountLogOut')
}

useAppEvent('process', async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

onMounted(() => {
	void refreshValues()
	void loadExternalAuthProviders().catch(handleError)
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
	},
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	selectAccount: {
		id: 'minecraft-account.select-account',
		defaultMessage: 'Select account',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
})
</script>

<template>
	<div class="relative flex items-center">
		<!-- When no accounts: button in frame to add account -->
		<button
			v-if="accounts.length === 0"
			type="button"
			class="flex border-solid border-surface-5 text-sm items-center gap-1.5 py-1.5 px-3 rounded-xl border bg-transparent hover:bg-surface-2 cursor-pointer transition-colors text-primary font-medium select-none"
			:disabled="loginDisabled || offlineLoginDisabled || externalAuthDisabled"
			@click="showAccountLoginModal"
		>
			<PlusIcon class="size-4 text-brand shrink-0" />
			<span>{{ formatMessage(messages.addAccount) }}</span>
		</button>

		<!-- When accounts exist: Dropdown in a frame -->
		<Dropdown
			v-else
			placement="bottom-end"
			:triggers="['click']"
			:hide-triggers="['click']"
			@show="showDropdown = true"
			@hide="showDropdown = false"
		>
			<button
				type="button"
				class="flex border-solid border-surface-5 text-sm items-center gap-2 py-1.5 px-3 rounded-xl border bg-transparent hover:bg-surface-2 cursor-pointer transition-colors text-primary font-medium select-none"
			>
				<Avatar
					:src="selectedAccount ? avatarUrl : 'https://launcher-files.modrinth.com/assets/steve_head.png'"
					size="20px"
					disable-conditional-icon-padding
				/>
				<component
					:is="getAccountType(selectedAccount)"
					v-if="selectedAccount && getAccountType(selectedAccount)"
					class="w-3.5 h-3.5 shrink-0 opacity-80"
				/>
				<span class="max-w-[120px] truncate text-contrast font-medium">
					{{ selectedAccount ? selectedAccount.profile.name : formatMessage(messages.selectAccount) }}
				</span>
				<DropdownIcon class="size-3 text-secondary transition-transform shrink-0" :class="{ 'rotate-180': showDropdown }" />
			</button>

			<template #popper>
				<div class="flex w-[20rem] max-h-[24rem] flex-col gap-1 p-2 bg-bg-raised border border-solid border-surface-5 rounded-xl shadow-xl overflow-auto">
					<div
						v-for="account in accounts"
						:key="account.profile.id"
						class="flex w-full items-center gap-1.5 rounded-lg p-1.5 transition-colors hover:bg-surface-3"
						:class="{ 'bg-surface-3': selectedAccount && selectedAccount.profile.id === account.profile.id }"
					>
						<button
							type="button"
							class="flex flex-grow items-center gap-2 bg-transparent border-0 cursor-pointer min-w-0 p-1 text-left"
							@click="setAccount(account)"
						>
							<RadioButtonCheckedIcon
								v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
								class="w-4 h-4 text-brand shrink-0"
							/>
							<RadioButtonIcon v-else class="w-4 h-4 text-secondary shrink-0" />
							<Avatar
								:src="getAccountAvatarUrl(account)"
								size="24px"
								disable-conditional-icon-padding
							/>
							<component
								:is="getAccountType(account)"
								v-if="getAccountType(account)"
								class="w-3.5 h-3.5 shrink-0 opacity-80"
							/>
							<span
								class="truncate text-sm"
								:class="selectedAccount && selectedAccount.profile.id === account.profile.id ? 'text-contrast font-semibold' : 'text-primary'"
							>
								{{ account.profile.name }}
							</span>
						</button>
						<IconButton
							v-tooltip="formatMessage(messages.removeAccount)"
							type="quiet"
							color="red"
							size="xs"
							:label="formatMessage(messages.removeAccount)"
							class="!size-7 hover:!bg-red hover:!text-[var(--color-accent-contrast)] shrink-0"
							@click.stop="logout(account.profile.id)"
						>
							<TrashIcon class="size-3.5" />
						</IconButton>
					</div>

					<div class="pt-2 mt-1 border-0 border-t border-solid border-surface-4">
						<Button
							type="colored"
							color="brand"
							class="w-full !justify-start !text-xs !py-1.5"
							@click="showAccountLoginModal"
						>
							<PlusIcon class="size-3.5" />
							{{ formatMessage(messages.addAccount) }}
						</Button>
					</div>
				</div>
			</template>
		</Dropdown>

		<AccountsInputModals
			ref="accountsInputModals"
			:offline-login-disabled="offlineLoginDisabled"
			:offline-player-name="offlinePlayerName"
			:login-disabled="loginDisabled"
			:external-auth-disabled="externalAuthDisabled"
			:external-auth-providers="externalAuthProviders"
			@login-microsoft="login"
			@login-external="addExternalProfile"
			@submit-offline="addOfflineProfile"
			@update:offline-player-name="offlinePlayerName = $event"
		/>
		<AccountsErrorModals
			ref="accountsErrorModals"
			:max-offline-player-name-length="maxOfflinePlayerNameLength"
			:min-offline-player-name-length="minOfflinePlayerNameLength"
			:name-exp="nameExp"
			@retry-offline="retryAddOfflineProfile"
		/>
	</div>
</template>
