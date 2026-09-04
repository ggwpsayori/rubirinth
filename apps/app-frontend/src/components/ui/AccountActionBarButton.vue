<script setup lang="ts">
import {
	DropdownIcon,
	GripVerticalIcon,
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
import type { Ref } from 'vue'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import Draggable from 'vuedraggable'

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
import { elybyHeadCache, loadElyByHead } from '@/helpers/elyby-skin'
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
const ACCOUNTS_ORDER_STORAGE_KEY = 'rubirinth_accounts_order'

function sortAccountsBySavedOrder(loadedAccounts: MinecraftCredential[]): MinecraftCredential[] {
	try {
		const rawOrder = localStorage.getItem(ACCOUNTS_ORDER_STORAGE_KEY)
		if (!rawOrder) return loadedAccounts
		const order: string[] = JSON.parse(rawOrder)
		if (!Array.isArray(order)) return loadedAccounts

		return [...loadedAccounts].sort((a, b) => {
			const indexA = order.indexOf(a.profile.id)
			const indexB = order.indexOf(b.profile.id)
			if (indexA === -1 && indexB === -1) return 0
			if (indexA === -1) return 1
			if (indexB === -1) return -1
			return indexA - indexB
		})
	} catch {
		return loadedAccounts
	}
}

function onAccountsReordered() {
	try {
		const order = accounts.value.map((a) => a.profile.id)
		localStorage.setItem(ACCOUNTS_ORDER_STORAGE_KEY, JSON.stringify(order))
		emit('change')
	} catch (e) {
		console.error('Failed to save accounts order', e)
	}
}

const loginDisabled = ref(false)
const offlineLoginDisabled = ref(false)
const defaultUser = ref<string | undefined>()
const equippedSkin = ref<Skin | null>(null)
const headUrlCache = ref(new Map<string, string>())
const isOpen = ref(false)

function toggleOpen() {
	if (accounts.value.length === 0) {
		showAccountLoginModal()
		return
	}
	isOpen.value = !isOpen.value
}

function close() {
	isOpen.value = false
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape' && isOpen.value) {
		close()
	}
}

const accountsInputModals = ref<AccountsInputModalsHandle | null>(null)
const accountsErrorModals = ref<AccountsErrorModalsHandle | null>(null)

const offlinePlayerName = ref('')
const { authenticate: addExternalProfile, disabled: externalAuthDisabled } =
	useExternalAuthentication({
		onAuthenticated: async (credentials) => {
			await setAccount(credentials)
			accountsInputModals.value?.hideAuth()
			close()
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
	close()
	accountsInputModals.value?.showOffline()
}

function showAccountLoginModal() {
	close()
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

function startOfflineCooldown() {
	offlineLoginDisabled.value = true
	setTimeout(() => {
		offlineLoginDisabled.value = false
	}, offlineLoginCooldownMs)
}

function isOfflinePlayerNameValid(name: string) {
	return (
		name.length >= minOfflinePlayerNameLength &&
		name.length <= maxOfflinePlayerNameLength &&
		nameRegex.test(name)
	)
}

async function addOfflineProfile() {
	if (offlineLoginDisabled.value) {
		return
	}

	startOfflineCooldown()
	const name = offlinePlayerName.value.trim()

	if (!isOfflinePlayerNameValid(name)) {
		accountsErrorModals.value?.showInputOfflineError()
		return
	}

	try {
		const user = await offline_login(name)
		await setAccount(user)
		clearOfflineFields()
		accountsInputModals.value?.hideOffline()
		close()
	} catch (error) {
		handleError(error)
		accountsErrorModals.value?.showUnexpectedError()
	}
}

async function refreshValues() {
	try {
		const loaded = await users().catch(handleError) ?? []
		accounts.value = sortAccountsBySavedOrder(loaded)
		defaultUser.value = await get_default_user().catch(handleError)

		for (const acc of accounts.value) {
			if (acc.account_type === 'elyby' || acc.account_type === 'offline') {
				void loadElyByHead(acc.profile.name)
			}
		}

		if (selectedAccount.value) {
			try {
				const availableSkins = await get_available_skins(selectedAccount.value.profile.id)
				equippedSkin.value = availableSkins.find((s) => s.state === 'Equipped') ?? null

				if (equippedSkin.value?.texture_key) {
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
	if (selectedAccount.value) {
		const account = selectedAccount.value
		if (account.account_type === 'elyby' || account.account_type === 'offline') {
			const elyHead = elybyHeadCache.value.get(account.profile.name.toLowerCase())
			if (elyHead) {
				return elyHead
			}
			void loadElyByHead(account.profile.name)
			return 'https://launcher-files.modrinth.com/assets/steve_head.png'
		}
	}

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

	if (account.account_type === 'elyby' || account.account_type === 'offline') {
		const elyHead = elybyHeadCache.value.get(account.profile.name.toLowerCase())
		if (elyHead) {
			return elyHead
		}
		void loadElyByHead(account.profile.name)
		return 'https://launcher-files.modrinth.com/assets/steve_head.png'
	}

	if (account.profile?.id) {
		return 'https://mc-heads.net/avatar/' + account.profile.id + '/128'
	}

	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
}

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
	close()
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
		accountsInputModals.value?.hideAuth()
		close()
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

function onAccountsUpdated() {
	void refreshValues()
}

onMounted(() => {
	void refreshValues()
	void loadExternalAuthProviders().catch(handleError)
	window.addEventListener('keydown', handleKeydown)
	window.addEventListener('rubirinth-accounts-updated', onAccountsUpdated)
})

onUnmounted(() => {
	window.removeEventListener('keydown', handleKeydown)
	window.removeEventListener('rubirinth-accounts-updated', onAccountsUpdated)
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
		<!-- Main Action Bar Trigger Button -->
		<button
			type="button"
			class="flex border-solid border-surface-5 text-sm items-center gap-2 py-1.5 px-3 rounded-xl border bg-transparent hover:bg-surface-2 cursor-pointer transition-colors text-primary font-medium select-none"
			:disabled="loginDisabled || offlineLoginDisabled || externalAuthDisabled"
			@click="toggleOpen"
		>
			<template v-if="accounts.length === 0">
				<PlusIcon class="size-4 text-brand shrink-0" />
				<span class="text-contrast font-medium">{{ formatMessage(messages.addAccount) }}</span>
			</template>
			<template v-else>
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
				<DropdownIcon
					class="size-3 text-secondary transition-transform shrink-0"
					:class="{ 'rotate-180': isOpen }"
				/>
			</template>
		</button>

		<!-- Backdrop to close on outside click -->
		<div
			v-if="isOpen"
			class="fixed inset-0 z-40 bg-transparent"
			@click="close"
		/>

		<!-- Dropdown popup matching Friends popover style -->
		<Transition
			enter-active-class="transition duration-150 ease-out"
			enter-from-class="transform scale-95 opacity-0"
			enter-to-class="transform scale-100 opacity-100"
			leave-active-class="transition duration-100 ease-in"
			leave-from-class="transform scale-100 opacity-100"
			leave-to-class="transform scale-95 opacity-0"
		>
			<div
				v-if="isOpen"
				class="absolute right-0 top-full mt-2 z-50 flex w-[22rem] max-h-[30rem] flex-col p-3 bg-bg-raised border border-solid border-surface-5 rounded-2xl shadow-2xl overflow-y-auto"
				@click.stop
			>
				<!-- Popover Header -->
				<div class="flex items-center mb-3 pb-2 border-0 border-b border-solid border-surface-4">
					<h3 class="text-base text-primary font-medium m-0">
						{{ formatMessage(messages.minecraftAccount) }}
					</h3>
				</div>

				<!-- Accounts list -->
				<div class="flex flex-col gap-1.5 overflow-y-auto">
					<Draggable
						:list="accounts"
						:item-key="(account) => account.profile.id" :force-fallback="true" :fallback-on-body="true" :fallback-tolerance="2"
						handle=".account-drag-handle"
						:animation="200"
						ghost-class="opacity-40"
						class="flex flex-col gap-1.5"
						@end="onAccountsReordered"
					>
						<template #item="{ element: account }">
							<div
								class="group/account flex w-full items-center gap-2 rounded-xl p-2 transition-colors hover:bg-surface-3 cursor-pointer select-none"
								:class="{ 'bg-surface-3': selectedAccount && selectedAccount.profile.id === account.profile.id }"
								@click="setAccount(account)"
							>
								<div
									class="account-drag-handle select-none touch-none flex items-center justify-center p-0.5 -ml-1 text-secondary opacity-35 group-hover/account:opacity-100 hover:!opacity-100 transition-opacity cursor-grab active:cursor-grabbing hover:text-contrast shrink-0"
									@click.stop
								>
									<GripVerticalIcon class="w-3.5 h-3.5" />
								</div>

								<RadioButtonCheckedIcon
									v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
									class="w-4 h-4 text-brand shrink-0"
								/>
								<RadioButtonIcon v-else class="w-4 h-4 text-secondary shrink-0" />
								<Avatar
									:src="getAccountAvatarUrl(account)"
									size="28px"
									disable-conditional-icon-padding
								/>
								<div class="flex flex-col min-w-0 flex-1">
									<div class="flex items-center gap-1.5">
										<span
											class="truncate text-sm"
											:class="selectedAccount && selectedAccount.profile.id === account.profile.id ? 'text-contrast font-semibold' : 'text-primary'"
										>
											{{ account.profile.name }}
										</span>
										<component
											:is="getAccountType(account)"
											v-if="getAccountType(account)"
											class="w-3.5 h-3.5 shrink-0 opacity-80"
										/>
									</div>
								</div>
								<IconButton
									v-tooltip="formatMessage(messages.removeAccount)"
									type="quiet"
									color="red"
									size="xs"
									:label="formatMessage(messages.removeAccount)"
									class="!size-7 hover:!bg-red hover:!text-[var(--color-accent-contrast)] shrink-0 opacity-0 group-hover/account:opacity-100 transition-opacity"
									@click.stop="logout(account.profile.id)"
								>
									<TrashIcon class="size-3.5" />
								</IconButton>
							</div>
						</template>
					</Draggable>

					<div v-if="accounts.length === 0" class="p-3 text-center text-sm text-secondary">
						{{ formatMessage(messages.notSignedIn) }}
					</div>
				</div>

				<!-- Add Account Button footer -->
				<div class="pt-3 mt-2 border-0 border-t border-solid border-surface-4">
					<Button
						type="colored"
						color="brand"
						class="w-full !justify-center !text-sm !py-2"
						@click="showAccountLoginModal"
					>
						<PlusIcon class="size-4" />
						{{ formatMessage(messages.addAccount) }}
					</Button>
				</div>
			</div>
		</Transition>

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
