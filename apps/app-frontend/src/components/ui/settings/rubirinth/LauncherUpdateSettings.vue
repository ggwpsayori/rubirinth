<script setup lang="ts">
import { BadgeCheckIcon, DownloadIcon, GithubIcon, RefreshCwIcon, SpinnerIcon } from '@modrinth/assets'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { getVersion } from '@tauri-apps/api/app'
import { computed, onMounted, ref } from 'vue'

import RubirinthSettingsPage from '@/components/ui/settings/rubirinth/RubirinthSettingsPage.vue'
import {
	GITHUB_RELEASES_URL,
	isUpdateAvailable,
	isUpdateDownloading,
	latestRelease,
	latestReleaseInstaller,
	startUpdateInstallation,
	fetchLatestRelease,
} from '@/helpers/rubirinth/update'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'

const { formatMessage } = useVIntl()
const currentVersion = ref('')
const checkingUpdates = ref(false)

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()

onMounted(async () => {
	try {
		currentVersion.value = await getVersion()
	} catch (e) {
		console.error('Failed to get app version:', e)
	}
})

const changelogContent = computed(() => {
	const raw = latestRelease.value?.body || ''
	if (!raw) return ''

	// Strip out the installation instructions section from GitHub release notes
	const installHeaderRegex = /(?:(?:\r?\n)+\s*(?:---|___|\*\*\*)\s*)*(?:\r?\n)+#{1,4}\s*(?:Установка|Installation)[\s\S]*$/i
	const trimmed = raw.replace(installHeaderRegex, '').trim()
	return trimmed || raw
})

const messages = defineMessages({
	pageTitle: {
		id: 'app.settings.updates.title',
		defaultMessage: 'Updates',
	},
	pageDescription: {
		id: 'app.settings.updates.description',
		defaultMessage: 'Check for new versions and view update history for Rubirinth App.',
	},
	checkUpdates: {
		id: 'app.settings.updates.check-updates',
		defaultMessage: 'Check for updates',
	},
	checking: {
		id: 'app.settings.updates.checking',
		defaultMessage: 'Checking...',
	},
	upToDateTitle: {
		id: 'app.settings.updates.up-to-date',
		defaultMessage: "You're using the latest version",
	},
	upToDateDesc: {
		id: 'app.settings.updates.up-to-date-desc',
		defaultMessage: 'Rubirinth App {version} is up to date.',
	},
	updateAvailableTitle: {
		id: 'app.settings.updates.update-available-title',
		defaultMessage: 'A new version of Rubirinth is available!',
	},
	updateAvailableDesc: {
		id: 'app.settings.updates.update-available-desc',
		defaultMessage:
			'Version {version} is available. We recommend updating to get the latest features and improvements.',
	},
	currentVersion: {
		id: 'app.settings.updates.current-version',
		defaultMessage: 'Current version',
	},
	latestVersion: {
		id: 'app.settings.updates.latest-version',
		defaultMessage: 'New version',
	},
	installNow: {
		id: 'app.settings.updates.install-now',
		defaultMessage: 'Update now',
	},
	downloading: {
		id: 'app.settings.updates.downloading',
		defaultMessage: 'Downloading...',
	},
	changelogTitle: {
		id: 'app.settings.updates.changelog',
		defaultMessage: "What's new in this version:",
	},
	viewOnGithub: {
		id: 'app.settings.updates.view-on-github',
		defaultMessage: 'View release on GitHub',
	},
})

async function handleCheckUpdates() {
	if (checkingUpdates.value || isUpdateDownloading.value) return
	checkingUpdates.value = true
	try {
		await fetchLatestRelease()
	} finally {
		checkingUpdates.value = false
	}
}

async function handleInstallUpdate() {
	await startUpdateInstallation()
}
</script>

<template>
	<RubirinthSettingsPage
		:title="formatMessage(messages.pageTitle)"
		:description="formatMessage(messages.pageDescription)"
	>
		<template #actions>
			<Button
				type="outlined"
				:disabled="checkingUpdates || isUpdateDownloading"
				@click="handleCheckUpdates"
			>
				<RefreshCwIcon :class="{ 'animate-spin': checkingUpdates }" />
				{{ checkingUpdates ? formatMessage(messages.checking) : formatMessage(messages.checkUpdates) }}
			</Button>
		</template>

		<div class="flex flex-col gap-5">
			<!-- Update available card -->
			<div
				v-if="isUpdateAvailable"
				class="flex flex-col gap-4 rounded-2xl border border-solid border-brand bg-[rgba(70,127,197,0.08)] p-5 shadow-sm"
			>
				<div class="flex items-start justify-between gap-4">
					<div class="flex items-center gap-3">
						<div
							class="flex size-11 shrink-0 items-center justify-center rounded-xl bg-brand text-contrast shadow-sm"
						>
							<DownloadIcon class="size-6" />
						</div>
						<div>
							<h2 class="m-0 text-lg font-bold text-contrast">
								{{ formatMessage(messages.updateAvailableTitle) }}
							</h2>
							<p class="m-0 mt-0.5 text-sm text-secondary">
								{{
									formatMessage(messages.updateAvailableDesc, {
										version: latestRelease?.tag_name?.replace(/^v/, ''),
									})
								}}
							</p>
						</div>
					</div>

					<Button
						type="colored"
						color="brand"
						:disabled="isUpdateDownloading || !latestReleaseInstaller"
						@click="handleInstallUpdate"
					>
						<SpinnerIcon v-if="isUpdateDownloading" class="animate-spin" />
						<DownloadIcon v-else />
						{{
							isUpdateDownloading
								? formatMessage(messages.downloading)
								: formatMessage(messages.installNow)
						}}
					</Button>
				</div>

				<div class="grid grid-cols-2 gap-3 rounded-xl border border-solid border-divider bg-surface-2 p-3 text-sm">
					<div class="flex flex-col gap-0.5">
						<span class="text-xs text-secondary">{{ formatMessage(messages.currentVersion) }}</span>
						<span class="font-semibold text-contrast">v{{ currentVersion.replace(/^v/, '') }}</span>
					</div>
					<div class="flex flex-col gap-0.5">
						<span class="text-xs text-secondary">{{ formatMessage(messages.latestVersion) }}</span>
						<span class="font-bold text-brand">v{{ latestRelease?.tag_name?.replace(/^v/, '') }}</span>
					</div>
				</div>

				<div v-if="isUpdateDownloading && progress > 0" class="flex flex-col gap-1.5 mt-1">
					<div class="flex justify-between text-xs text-secondary">
						<span>{{ formatMessage(messages.downloading) }}</span>
						<span>{{ Math.round(progress * 100) }}%</span>
					</div>
					<div class="h-2 w-full overflow-hidden rounded-full bg-surface-3">
						<div
							class="h-full bg-brand transition-all duration-200"
							:style="{ width: `${progress * 100}%` }"
						/>
					</div>
				</div>
			</div>

			<!-- Up-to-date card -->
			<div
				v-else
				class="flex items-center gap-4 rounded-2xl border border-solid border-divider bg-surface-2 p-5"
			>
				<div
					class="flex size-12 shrink-0 items-center justify-center rounded-2xl border border-solid border-[rgba(27,217,106,0.35)] bg-[rgba(27,217,106,0.12)] text-brand"
				>
					<BadgeCheckIcon class="size-6" />
				</div>
				<div>
					<h2 class="m-0 text-base font-bold text-contrast">
						{{ formatMessage(messages.upToDateTitle) }}
					</h2>
					<p class="m-0 mt-0.5 text-sm text-secondary">
						{{
							formatMessage(messages.upToDateDesc, {
								version: `v${currentVersion.replace(/^v/, '')}`,
							})
						}}
					</p>
				</div>
			</div>

			<!-- Release changelog -->
			<div
				v-if="changelogContent"
				class="flex flex-col gap-3 rounded-2xl border border-solid border-divider bg-surface-2 p-5"
			>
				<div class="flex items-center justify-between">
					<h3 class="m-0 text-sm font-semibold uppercase tracking-wider text-secondary">
						{{ formatMessage(messages.changelogTitle) }}
					</h3>
					<a
						v-if="latestRelease?.html_url"
						class="inline-flex items-center gap-1.5 text-sm font-medium text-brand hover:underline"
						:href="latestRelease.html_url || GITHUB_RELEASES_URL"
						target="_blank"
						rel="noopener noreferrer"
					>
						<GithubIcon class="size-4" />
						{{ formatMessage(messages.viewOnGithub) }}
					</a>
				</div>

				<div
					class="markdown-body max-h-80 overflow-y-auto rounded-xl border border-solid border-divider bg-bg-secondary p-4 text-sm leading-relaxed text-primary"
					v-html="renderString(changelogContent)"
				/>
			</div>
		</div>
	</RubirinthSettingsPage>
</template>
