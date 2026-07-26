<script setup lang="ts">
import { ElyByIcon, ExternalIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, useVIntl } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

const props = defineProps<{
	accountType?: string
}>()

const messages = defineMessages({
	title: {
		id: 'astralrinth.app.skins.unsupported-account.title',
		defaultMessage: 'Microsoft account required',
	},
	description: {
		id: 'astralrinth.app.skins.unsupported-account.description',
		defaultMessage:
			'Skins can only be managed with a Microsoft account. Return home and select or add a Microsoft account to continue.',
	},
	returnHome: {
		id: 'astralrinth.app.skins.unsupported-account.return-home',
		defaultMessage: 'Return home',
	},
	elyByTitle: {
		id: 'astralrinth.app.skins.unsupported-account.elyby.title',
		defaultMessage: 'Manage your Ely.by skin',
	},
	elyByDescription: {
		id: 'astralrinth.app.skins.unsupported-account.elyby.description',
		defaultMessage:
			"You're signed in with an Ely.by account. You can upload, select, and manage your skin directly on the Ely.by website.",
	},
	elyByAction: {
		id: 'astralrinth.app.skins.unsupported-account.elyby.action',
		defaultMessage: 'Manage skin on Ely.by',
	},
})

const router = useRouter()
const { formatMessage } = useVIntl()
const isElyByAccount = computed(() => props.accountType === 'elyby')

function returnHome() {
	void router.push({ path: '/' })
}

function openElyBySkins() {
	void openUrl('https://account.ely.by/login')
}
</script>

<template>
	<div
		class="box-border flex min-h-full cursor-pointer items-center justify-center p-4 sm:p-8"
		role="alert"
		tabindex="0"
		@click="returnHome"
		@keydown="returnHome"
	>
		<div
			class="grid w-full max-w-3xl overflow-hidden rounded-2xl border border-solid border-surface-5 bg-bg-raised shadow-xl"
			:class="{ 'md:grid-cols-[minmax(220px,0.8fr)_minmax(0,1.5fr)]': isElyByAccount }"
		>
			<div
				v-if="isElyByAccount"
				class="relative flex min-h-48 items-center justify-center overflow-hidden bg-surface-2 p-8 md:min-h-full"
			>
				<div class="absolute inset-0 bg-gradient-to-br from-bg-blue via-surface-2 to-bg-raised" />
				<div
					class="relative flex size-28 items-center justify-center rounded-3xl border border-solid border-surface-5 bg-bg-raised shadow-lg"
				>
					<ElyByIcon class="size-20" aria-hidden="true" />
				</div>
			</div>

			<div class="flex min-w-0 flex-col gap-5 p-7 sm:p-9">
				<h1 class="m-0 text-3xl font-extrabold leading-tight">
					{{
						formatMessage(isElyByAccount ? messages.elyByTitle : messages.title)
					}}
				</h1>
				<p class="m-0 text-lg leading-relaxed text-secondary">
					{{
						formatMessage(isElyByAccount ? messages.elyByDescription : messages.description)
					}}
				</p>
				<div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap">
					<ButtonStyled v-if="isElyByAccount" color="brand">
						<button @click.stop="openElyBySkins" @keydown.stop>
							{{ formatMessage(messages.elyByAction) }}
							<ExternalIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
					<ButtonStyled :type="isElyByAccount ? 'outlined' : 'standard'" color="brand">
						<button @click.stop="returnHome">
							{{ formatMessage(messages.returnHome) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>
