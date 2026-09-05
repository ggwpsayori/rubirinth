<script setup lang="ts">
import {
	CoffeeIcon,
	DownloadIcon,
	GaugeIcon,
	HeartHandshakeIcon,
	KeyIcon,
	LanguagesIcon,
	ModrinthIcon,
	PaintbrushIcon,
	RefreshCwIcon,
	Settings2Icon,
	ShieldIcon,
	ToggleRightIcon,
	UserIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	ProgressBar,
	TabbedModal,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { computed, provide, ref, watch } from 'vue'

import PrivacySettings from '@/components/ui/settings/account/PrivacySettings.vue'
import ProfileSettings from '@/components/ui/settings/account/ProfileSettings.vue'
import SocialSettings from '@/components/ui/settings/account/SocialSettings.vue'
import AppearanceSettings from '@/components/ui/settings/display/AppearanceSettings.vue'
import BehaviorSettings from '@/components/ui/settings/display/BehaviorSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/display/FeatureFlagSettings.vue'
import LanguageSettings from '@/components/ui/settings/display/LanguageSettings.vue'
import InstancesSyncedSettings from '@/components/ui/settings/instances/InstancesSyncedSettings.vue'
import JavaSettings from '@/components/ui/settings/instances/JavaSettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/instances/ResourceManagementSettings.vue'
import AdvancedSettings from '@/components/ui/settings/rubirinth/AdvancedSettings.vue'
import ExternalAuthLibrarySettings from '@/components/ui/settings/rubirinth/ExternalAuthLibrarySettings.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get, set } from '@/helpers/settings.ts'
import {
	appSettingsModalContextKey,
	type UnsavedChangesController,
} from '@/providers/app-settings-modal'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import LauncherUpdateModal from '@/components/ui/rubirinth/LauncherUpdateModal.vue'
import { isUpdateAvailable, latestRelease } from '@/helpers/rubirinth/update'

// TODO: Apply COMPONENT_STRUCTURE.md here and extract out common setting option components
const appSettings = useAppSettings()

const { formatMessage } = useVIntl()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabCategories = defineMessages({
	display: {
		id: 'settings.sidebar.label.display',
		defaultMessage: 'Display',
	},
	account: {
		id: 'settings.sidebar.label.account',
		defaultMessage: 'Account',
	},
	instances: {
		id: 'app.settings.sidebar.label.instances',
		defaultMessage: 'Instances',
	},
	rubirinth: {
		id: 'app.settings.sidebar.label.rubirinth',
		defaultMessage: 'Rubirinth',
	},
	advanced: {
		id: 'app.settings.sidebar.label.advanced',
		defaultMessage: 'Advanced',
	},
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		category: tabCategories.display,
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.behavior',
			defaultMessage: 'Behavior',
		}),
		category: tabCategories.display,
		icon: Settings2Icon,
		content: BehaviorSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Language',
		}),
		category: tabCategories.display,
		icon: LanguagesIcon,
		content: LanguageSettings,
	},
	{
		name: commonSettingsMessages.featureFlags,
		category: tabCategories.display,
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
	{
		name: commonSettingsMessages.profile,
		category: tabCategories.account,
		icon: UserIcon,
		content: ProfileSettings,
	},
	{
		name: commonSettingsMessages.social,
		category: tabCategories.account,
		icon: HeartHandshakeIcon,
		content: SocialSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		category: tabCategories.account,
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.synced-options',
			defaultMessage: 'Synced settings',
		}),
		category: tabCategories.instances,
		icon: RefreshCwIcon,
		content: InstancesSyncedSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java installations',
		}),
		category: tabCategories.instances,
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Resource management',
		}),
		category: tabCategories.instances,
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.external-auth-libraries',
			defaultMessage: 'Authentication libraries',
		}),
		category: tabCategories.rubirinth,
		icon: KeyIcon,
		content: ExternalAuthLibrarySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.import',
			defaultMessage: 'Import',
		}),
		category: tabCategories.advanced,
		icon: DownloadIcon,
		content: AdvancedSettings,
	},
]

const availableTabs = computed(() =>
	tabs.filter((tab) => !tab.developerOnly || appSettings.devMode),
)

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)
const unsavedChangesPopup = ref<{ nudge: () => void } | null>(null)
const unsavedChangesController = ref<UnsavedChangesController | null>(null)
const emptyUnsavedChangesState: Record<string, unknown> = {}
const originalUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getOriginal() ?? emptyUnsavedChangesState,
)
const modifiedUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getModified() ?? emptyUnsavedChangesState,
)
const savingUnsavedChanges = computed(() => unsavedChangesController.value?.isSaving() ?? false)
const hasUnsavedChanges = computed(
	() =>
		(unsavedChangesController.value?.hasChanges() ?? false) ||
		(unsavedChangesController.value?.isSaving() ?? false),
)

function canLeaveCurrentTab(): boolean {
	if (
		!unsavedChangesController.value?.hasChanges() &&
		!unsavedChangesController.value?.isSaving()
	) {
		return true
	}
	unsavedChangesPopup.value?.nudge()
	return false
}

function close(): boolean {
	return modal.value?.hide() ?? false
}

function registerUnsavedChangesController(controller: UnsavedChangesController | null): void {
	unsavedChangesController.value = controller
}

provide(appSettingsModalContextKey, {
	close,
	registerUnsavedChangesController,
})

function resetUnsavedChanges(): void {
	unsavedChangesController.value?.reset()
}

function saveUnsavedChanges(): void {
	void unsavedChangesController.value?.save()
}

function show() {
	modal.value?.show()
}

function showProfile(): void {
	const profileTabIndex = availableTabs.value.findIndex((tab) => tab.content === ProfileSettings)
	if (profileTabIndex >= 0) {
		modal.value?.setTab(profileTabIndex)
	}
	modal.value?.show()
}

function showFeatureFlags(): void {
	const featureFlagsTabIndex = availableTabs.value.findIndex(
		(tab) => tab.content === FeatureFlagSettings,
	)
	if (featureFlagsTabIndex >= 0) {
		modal.value?.setTab(featureFlagsTabIndex)
	}
	modal.value?.show()
}

function showSyncedOptions(): void {
	const syncedOptionsTabIndex = availableTabs.value.findIndex(
		(tab) => tab.content === InstancesSyncedSettings,
	)
	if (syncedOptionsTabIndex >= 0) {
		modal.value?.setTab(syncedOptionsTabIndex)
	}
	modal.value?.show()
}

function showExternalAuthLibraries(): void {
	const authLibrariesTabIndex = availableTabs.value.findIndex(
		(tab) => tab.content === ExternalAuthLibrarySettings,
	)
	if (authLibrariesTabIndex >= 0) {
		modal.value?.setTab(authLibrariesTabIndex)
	}
	modal.value?.show()
}

defineExpose({ show, showProfile, showFeatureFlags, showSyncedOptions, showExternalAuthLibraries })

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()
const launcherUpdateModal = ref<InstanceType<typeof LauncherUpdateModal> | null>(null)

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		const selectedTab = modal.value ? availableTabs.value[modal.value.selectedTab] : undefined

		appSettings.devMode = !appSettings.devMode
		settings.value.developer_mode = !!appSettings.devMode
		devModeCounter.value = 0

		if (modal.value) {
			const selectedTabIndex = selectedTab ? availableTabs.value.indexOf(selectedTab) : -1
			modal.value.setTab(selectedTabIndex >= 0 ? selectedTabIndex : 0)
		}
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	appVersion: {
		id: 'app.settings.app-version',
		defaultMessage: 'Rubirinth App {version}',
	},
	macos: {
		id: 'app.settings.operating-system.macos',
		defaultMessage: 'macOS',
	},
	developerModeButtonLabel: {
		id: 'app.settings.developer-mode-button.label',
		defaultMessage: 'Toggle developer mode',
	},
	updateAvailableBtn: {
		id: 'app.settings.update-available-button',
		defaultMessage: 'Update to {version}',
	},
})
</script>
<template>
	<TabbedModal
		ref="modal"
		:tabs="availableTabs"
		:width="'min(928px, calc(95vw - 10rem))'"
		:before-hide="canLeaveCurrentTab"
		:before-tab-change="canLeaveCurrentTab"
		:floating-action-bar-shown="hasUnsavedChanges"
	>
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{ formatMessage(commonMessages.settingsLabel) }}
			</span>
		</template>
		<template #floating-action-bar>
			<UnsavedChangesPopup
				ref="unsavedChangesPopup"
				:original="originalUnsavedChangesState"
				:modified="modifiedUnsavedChangesState"
				:saving="savingUnsavedChanges"
				inline
				@reset="resetUnsavedChanges"
				@save="saveUnsavedChanges"
			/>
		</template>
		<template #footer>
			<div class="mt-auto text-secondary text-sm">
				<div class="mb-3">
					<template v-if="progress > 0 && progress < 1">
						<p class="m-0 mb-2">
							{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
						</p>
						<ProgressBar :progress="progress" />
					</template>
				</div>
				<p v-if="appSettings.devMode" class="text-brand font-semibold m-0 mb-2">
					{{ formatMessage(developerModeEnabled) }}
				</p>
				<div class="flex items-center gap-3">
					<button
						:aria-label="formatMessage(messages.developerModeButtonLabel)"
						class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation flex items-center justify-center"
						:class="{
							'text-brand': appSettings.devMode,
							'text-secondary': !appSettings.devMode,
						}"
						@click="devModeCount"
					>
						<svg
							viewBox="0 0 424 419"
							fill="currentColor"
							xmlns="http://www.w3.org/2000/svg"
							class="w-6 h-6"
							aria-hidden="true"
						>
							<path d="M0 0L423.407 75.8519L307.852 170.074H258.519L423.407 418.963L99.4074 123.407L220.296 132.741L222.074 111.407L84.7407 54.5185L56.2963 71.8519L0 15.4074V0Z" />
							<path d="M91.5658 265.778L187.852 337.481H31.3467L91.5658 265.778Z" />
						</svg>
					</button>
					<div class="max-w-[200px]">
						<p class="m-0">
							{{ formatMessage(messages.appVersion, { version }) }}
						</p>
						<p class="m-0">
							<span v-if="osPlatform === 'macos'">{{ formatMessage(messages.macos) }}</span>
							<span v-else class="capitalize">{{ osPlatform }}</span>
							{{ osVersion }}
						</p>
					</div>
				</div>
				<div v-if="isUpdateAvailable" class="mt-3">
					<Button
						type="colored"
						color="brand"
						class="w-full justify-center"
						@click="launcherUpdateModal?.show()"
					>
						<DownloadIcon class="w-4 h-4 mr-1.5" />
						{{ formatMessage(messages.updateAvailableBtn, { version: latestRelease?.tag_name }) }}
					</Button>
				</div>
			</div>
		</template>
	</TabbedModal>
	<LauncherUpdateModal ref="launcherUpdateModal" :version="version" />
</template>
