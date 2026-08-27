<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="544px" no-padding>
		<div class="grid grid-cols-[1fr_auto] gap-2.5 h-[154px] px-7 pt-4 pb-1 pr-9">
			<div class="flex flex-col gap-2.5 items-start justify-center h-min mt-5">
				<div class="font-semibold text-xl text-contrast">
					{{ formatMessage(messages.descriptionHeader) }}
				</div>
				<div class="text-secondary leading-6">
					{{ formatMessage(messages.description) }}
				</div>
			</div>
			<div class="relative h-full w-[96px] overflow-hidden mx-3">
				<div class="absolute top-0 left-0 z-0 w-full flex grow-0 flex-col items-end p-0">
					<img :src="steveImage" alt="" class="self-stretch" />
				</div>
				<div
					class="absolute left-0 bottom-0 z-10 order-1 h-6 w-[120px] shrink-0 grow-0 bg-[linear-gradient(180deg,rgba(39,41,46,0)_0%,#27292E_80%,#27292E_100%)]"
				></div>
			</div>
		</div>

		<div class="flex flex-col gap-6 px-6 pb-6">
			<div class="flex justify-end gap-2">
				<ButtonLink class="min-w-0 flex-1" href="https://support.modrinth.com" @click="modal?.hide()">
					<MessagesSquareIcon />
					{{ formatMessage(messages.getSupport) }}
				</ButtonLink>
				<Button
					type="colored"
					color="brand"
					class="min-w-0 flex-1"
					:disabled="loadingSignIn"
					@click="showAccountLoginModal"
				>
					<LogInIcon />
					{{ formatMessage(messages.otherSignInMethods) }}
				</Button>
			</div>
			<p class="m-0 text-center text-sm text-secondary">
				{{ formatMessage(messages.dontHaveAccount) }}
				<a
					class="text-blue font-medium hover:underline"
					href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc"
				>
					{{ formatMessage(messages.getMinecraft) }}
				</a>
			</p>
		</div>
	</NewModal>

	<AccountsInputModals
		ref="accountsInputModals"
		v-model:offline-player-name="offlinePlayerName"
		:offline-login-disabled="offlineLoginDisabled"
		:login-disabled="loginDisabled"
		:external-auth-disabled="externalAuthDisabled"
		:external-auth-providers="externalAuthProviders"
		@submit-offline="addOfflineProfile"
		@login-microsoft="login"
		@login-external="addExternalProfile"
	/>

	<AccountsErrorModals
		ref="accountsErrorModals"
		@retry-add-offline="retryAddOfflineProfile"
	/>
</template>

<script setup lang="ts">
import { LogInIcon, MessagesSquareIcon } from '@modrinth/assets'
import { Button, ButtonLink, defineMessages, injectNotificationManager, NewModal, useVIntl } from '@modrinth/ui'
import { onMounted, ref } from 'vue'

import steveImage from '@/assets/steve-look-up-left.webp'
import AccountsErrorModals from '@/components/ui/astralrinth/accounts/error/AccountsErrorModals.vue'
import AccountsInputModals from '@/components/ui/astralrinth/accounts/input/AccountsInputModals.vue'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { login as login_flow, offline_login, set_default_user } from '@/helpers/auth'
import {
	externalAuthProviders,
	loadExternalAuthProviders,
	type MinecraftCredential,
	useExternalAuthentication,
} from '@/models/astralrinth/authentication'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const messages = defineMessages({
	header: {
		id: 'minecraft-required.header',
		defaultMessage: 'Minecraft required',
	},
	descriptionHeader: {
		id: 'minecraft-required.description-header',
		defaultMessage: 'Sign in to a Minecraft account',
	},
	description: {
		id: 'minecraft-required.description',
		defaultMessage:
			'You need a Minecraft account before you can launch and play.',
	},
	getSupport: {
		id: 'minecraft-required.get-support',
		defaultMessage: 'Get support',
	},
	otherSignInMethods: {
		id: 'minecraft-required.other-sign-in-methods',
		defaultMessage: 'Sign-in methods',
	},
	dontHaveAccount: {
		id: 'minecraft-required.dont-have-account',
		defaultMessage: 'Don\'t have an account?',
	},
	getMinecraft: {
		id: 'minecraft-required.get-minecraft',
		defaultMessage: 'Get Minecraft',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const accountsInputModals = ref<InstanceType<typeof AccountsInputModals> | null>(null)
const accountsErrorModals = ref<InstanceType<typeof AccountsErrorModals> | null>(null)

const loadingSignIn = ref(false)
const loginDisabled = ref(false)
const offlineLoginDisabled = ref(false)
const offlinePlayerName = ref('')

const offlineLoginCooldownMs = 1000
const minOfflinePlayerNameLength = 3
const maxOfflinePlayerNameLength = 20
const nameExp = 'a-zA-Z0-9_'
const nameRegex = new RegExp('^[' + nameExp + ']+$')

const { authenticate: addExternalProfile, disabled: externalAuthDisabled } =
	useExternalAuthentication({
		onAuthenticated: async (credentials) => {
			await onAccountAdded(credentials)
			accountsInputModals.value?.hideAuth()
		},
		onError: (error) => {
			handleError(error)
			accountsErrorModals.value?.showUnexpectedError()
		},
	})

function show() {
	modal.value?.show()
}

function showAccountLoginModal() {
	modal.value?.hide()
	accountsInputModals.value?.showAuth()
}

function showOfflineLoginModal() {
	accountsInputModals.value?.showOffline()
}

function retryAddOfflineProfile() {
	accountsErrorModals.value?.hideInputOfflineError()
	offlineLoginDisabled.value = false
	offlinePlayerName.value = ''
	showOfflineLoginModal()
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

async function onAccountAdded(account: MinecraftCredential) {
	try {
		await set_default_user(account.profile.id)
	} catch (e) {
		handleError(e)
	}
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
		await onAccountAdded(user)
		offlinePlayerName.value = ''
		accountsInputModals.value?.hideOffline()
	} catch (error) {
		handleError(error)
		accountsErrorModals.value?.showUnexpectedError()
	}
}

async function login() {
	loginDisabled.value = true
	try {
		const loggedIn = await login_flow().catch(handleSevereError)
		if (loggedIn) {
			await onAccountAdded(loggedIn)
			accountsInputModals.value?.hideAuth()
		}
		trackEvent('AccountLogIn')
	} finally {
		loginDisabled.value = false
	}
}

onMounted(() => {
	void loadExternalAuthProviders().catch(handleError)
})

defineExpose({
	show,
})
</script>
