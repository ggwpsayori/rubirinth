<script setup lang="ts">
import { DownloadIcon, GithubIcon, SpinnerIcon } from '@modrinth/assets'
import { Button, ButtonLink, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { computed, ref } from 'vue'

import {
	GITHUB_RELEASES_URL,
	isUpdateDownloading,
	latestRelease,
	latestReleaseInstaller,
	startUpdateInstallation,
} from '@/helpers/rubirinth/update'

const props = defineProps<{
	version: string
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)

const changelogContent = computed(() => {
	const raw = latestRelease.value?.body || ''
	if (!raw) return ''

	// Strip out the installation instructions section from GitHub release notes
	const installHeaderRegex = /(?:(?:\r?\n)+\s*(?:---|___|\*\*\*)\s*)*(?:\r?\n)+#{1,4}\s*(?:Установка|Installation)[\s\S]*$/i
	const trimmed = raw.replace(installHeaderRegex, '').trim()
	return trimmed || raw
})

const messages = defineMessages({
	header: {
		id: 'rubirinth.app.launcher-update-modal.header',
		defaultMessage: 'Update available',
	},
	title: {
		id: 'rubirinth.app.launcher-update-modal.title',
		defaultMessage: 'A new version of Rubirinth is available!',
	},
	description: {
		id: 'rubirinth.app.launcher-update-modal.description',
		defaultMessage:
			'Update to get the latest features, improvements, and bug fixes directly from the official repository.',
	},
	currentVersion: {
		id: 'rubirinth.app.launcher-update-modal.current-version',
		defaultMessage: 'Current version:',
	},
	changelogTitle: {
		id: 'rubirinth.app.launcher-update-modal.changelog-title',
		defaultMessage: "What's new:",
	},
	latestVersion: {
		id: 'rubirinth.app.launcher-update-modal.latest-version',
		defaultMessage: 'New version:',
	},
	changelog: {
		id: 'rubirinth.app.launcher-update-modal.changelog',
		defaultMessage: 'View changelog on GitHub',
	},
	downloadAndInstall: {
		id: 'rubirinth.app.launcher-update-modal.download-and-install',
		defaultMessage: 'Update now',
	},
	downloading: {
		id: 'rubirinth.app.launcher-update-modal.downloading',
		defaultMessage: 'Downloading...',
	},
	close: {
		id: 'rubirinth.app.launcher-update-modal.close',
		defaultMessage: 'Later',
	},
})

async function onUpdateClick() {
	const success = await startUpdateInstallation()
	if (success) {
		modal.value?.hide()
	}
}

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="560px">
		<div class="flex flex-col gap-4">
			<div>
				<h3 class="m-0 text-lg font-bold text-contrast">
					{{ formatMessage(messages.title) }}
				</h3>
				<p class="m-0 mt-1 text-sm text-secondary">
					{{ formatMessage(messages.description) }}
				</p>
			</div>

			<div class="rounded-xl border border-solid border-divider bg-bg-secondary p-3 text-sm flex flex-col gap-1.5">
				<div class="flex justify-between">
					<span class="text-secondary">{{ formatMessage(messages.currentVersion) }}</span>
					<span class="font-semibold text-contrast">v{{ props.version }}</span>
				</div>
				<div class="flex justify-between">
					<span class="text-secondary">{{ formatMessage(messages.latestVersion) }}</span>
					<span class="font-bold text-brand">{{ latestRelease?.tag_name }}</span>
				</div>
			</div>

			<!-- Release changelog from GitHub -->
			<div
				v-if="changelogContent"
				class="flex flex-col gap-1.5 rounded-xl border border-solid border-divider bg-bg-secondary p-3.5 max-h-60 overflow-y-auto"
			>
				<span class="text-xs font-semibold uppercase tracking-wider text-secondary">
					{{ formatMessage(messages.changelogTitle) }}
				</span>
				<div
					class="markdown-body text-sm text-primary leading-relaxed"
					v-html="renderString(changelogContent)"
				/>
			</div>

			<div v-if="latestRelease?.html_url" class="text-sm">
				<a
					class="text-brand font-medium hover:underline inline-flex items-center gap-1.5"
					:href="latestRelease.html_url || GITHUB_RELEASES_URL"
					target="_blank"
					rel="noopener noreferrer"
				>
					<GithubIcon class="w-4 h-4" />
					{{ formatMessage(messages.changelog) }}
				</a>
			</div>

			<div class="flex justify-end gap-2 pt-2">
				<Button class="bordered" :disabled="isUpdateDownloading" @click="hide">
					{{ formatMessage(messages.close) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="isUpdateDownloading || !latestReleaseInstaller"
					@click="onUpdateClick"
				>
					<SpinnerIcon v-if="isUpdateDownloading" class="animate-spin" />
					<DownloadIcon v-else />
					{{
						isUpdateDownloading
							? formatMessage(messages.downloading)
							: formatMessage(messages.downloadAndInstall)
					}}
				</Button>
			</div>
		</div>
	</NewModal>
</template>
