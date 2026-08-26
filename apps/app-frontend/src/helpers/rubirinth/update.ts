import { getVersion } from '@tauri-apps/api/app'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { ref } from 'vue'

import { downloadAndInstallUpdate, getOS } from '@/helpers/utils.js'

export type GitHubReleaseAsset = {
	name: string
	browser_download_url: string
	size: number
}

export type GitHubRelease = {
	tag_name: string
	name: string
	body: string
	html_url: string
	assets: GitHubReleaseAsset[]
}

export const GITHUB_LATEST_RELEASE_API = 'https://api.github.com/repos/ggwpsayori/rubirinth/releases/latest'
export const GITHUB_RELEASES_URL = 'https://github.com/ggwpsayori/rubirinth/releases'

export const isUpdateAvailable = ref(false)
export const isUpdateDownloading = ref(false)
export const latestRelease = ref<GitHubRelease | null>(null)
export const latestReleaseInstaller = ref<GitHubReleaseAsset | null>(null)

export async function fetchLatestRelease(): Promise<void> {
	try {
		const os = (await getOS()).toLowerCase()
		if (os !== 'windows') {
			console.log('Skipping update check on non-windows platform:', os)
			return
		}

		const fetchFn = typeof tauriFetch === 'function' ? tauriFetch : fetch
		const response = await fetchFn(GITHUB_LATEST_RELEASE_API, {
			headers: {
				Accept: 'application/vnd.github+json',
				'User-Agent': 'Rubirinth-App',
			},
		})
		if (!response.ok) {
			console.log('Failed to fetch GitHub release:', response.status)
			return
		}

		const release: GitHubRelease = await response.json()
		latestRelease.value = release

		// Find Windows .exe installer asset
		const exeAsset = release.assets.find((asset) => asset.name.toLowerCase().endsWith('.exe'))
		latestReleaseInstaller.value = exeAsset ?? null

		const currentRaw = await getVersion()
		const currentVer = normalizeVersion(currentRaw)
		const remoteVer = normalizeVersion(release.tag_name)

		const isNewer = compareVersions(remoteVer, currentVer) > 0
		console.log('[Updater] Current:', currentVer, 'Remote:', remoteVer, 'IsNewer:', isNewer)
		isUpdateAvailable.value = isNewer
	} catch (error) {
		console.error('[Updater] Failed to check for updates:', error)
	}
}

export async function startUpdateInstallation(): Promise<boolean> {
	if (!latestReleaseInstaller.value) return false

	try {
		isUpdateDownloading.value = true
		await downloadAndInstallUpdate(
			latestReleaseInstaller.value.browser_download_url,
			latestReleaseInstaller.value.name,
		)
		return true
	} catch (error) {
		console.error('Failed to download & install update:', error)
		return false
	} finally {
		isUpdateDownloading.value = false
	}
}

function normalizeVersion(ver: string): string {
	return ver.trim().replace(/^v/i, '')
}

function compareVersions(left: string, right: string): number {
	const leftParts = left.split(/[.-]/).map((x) => parseInt(x, 10) || 0)
	const rightParts = right.split(/[.-]/).map((x) => parseInt(x, 10) || 0)
	const len = Math.max(leftParts.length, rightParts.length)

	for (let i = 0; i < len; i++) {
		const l = leftParts[i] ?? 0
		const r = rightParts[i] ?? 0
		if (l !== r) return l - r
	}
	return 0
}
