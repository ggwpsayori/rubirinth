<script setup lang="ts">
import {
	CheckIcon,
	ChevronRightIcon,
	FolderIcon,
	PlusIcon,
	RefreshCwIcon,
	SearchIcon,
	SpinnerIcon,
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	Checkbox,
	defineMessages,
	injectInstanceImport,
	injectNotificationManager,
	Input,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const importProvider = injectInstanceImport()
const debug = useDebugLogger('AdvancedSettings')

type LauncherInfo = {
	name: string
	path: string
	instances: string[]
}

const loading = ref(true)
const scanning = ref(false)
const importing = ref(false)
const searchQuery = ref('')
const launchers = ref<LauncherInfo[]>([])
const expandedLaunchers = ref<Set<string>>(new Set())
const selectedInstances = ref<Record<string, Set<string>>>({})
const newCustomPath = ref('')
const showAddCustomModal = ref(false)

const messages = defineMessages({
	advancedTitle: {
		id: 'app.settings.advanced.title',
		defaultMessage: 'Advanced settings',
	},
	importSectionTitle: {
		id: 'app.settings.advanced.import-section.title',
		defaultMessage: 'Import from other launchers',
	},
	importSectionDescription: {
		id: 'app.settings.advanced.import-section.description',
		defaultMessage:
			'Import your instances, mods, and worlds from other Minecraft launchers (MultiMC, Prism Launcher, GDLauncher, ATLauncher, CurseForge) or a custom directory.',
	},
	scanLaunchersButton: {
		id: 'app.settings.advanced.scan-launchers',
		defaultMessage: 'Scan launchers',
	},
	addCustomFolderButton: {
		id: 'app.settings.advanced.add-custom-folder',
		defaultMessage: 'Add custom folder...',
	},
	searchPlaceholder: {
		id: 'app.settings.advanced.search-instances',
		defaultMessage: 'Search instance name...',
	},
	noLaunchersDetected: {
		id: 'app.settings.advanced.no-launchers-detected',
		defaultMessage:
			'No instances were automatically detected from other launchers. You can specify a custom directory above.',
	},
	importSelectedButton: {
		id: 'app.settings.advanced.import-selected',
		defaultMessage: 'Import selected ({count})',
	},
	importingButton: {
		id: 'app.settings.advanced.importing',
		defaultMessage: 'Importing...',
	},
	importSuccessTitle: {
		id: 'app.settings.advanced.import-success.title',
		defaultMessage: 'Import completed',
	},
	importSuccessText: {
		id: 'app.settings.advanced.import-success.text',
		defaultMessage: 'Successfully imported {count} instance(s).',
	},
	clearSelection: {
		id: 'app.settings.advanced.clear-selection',
		defaultMessage: 'Clear selection',
	},
	instancesCount: {
		id: 'app.settings.advanced.instances-count',
		defaultMessage: '{count} instance(s)',
	},
	customLauncherName: {
		id: 'creation-flow.modal.import-instance.custom-launcher.name',
		defaultMessage: 'Custom ({pathName})',
	},
	noInstancesFound: {
		id: 'creation-flow.modal.import-instance.notification.no-instances-found.title',
		defaultMessage: 'No instances found',
	},
})

async function scanLaunchers() {
	scanning.value = true
	try {
		debug('scanLaunchers: detecting...')
		const detected = await importProvider.getDetectedLaunchers()
		launchers.value = detected.map((l) => ({
			name: l.name,
			path: l.path,
			instances: l.instances ?? [],
		}))

		// Auto expand all detected launchers that have instances
		const newExpanded = new Set<string>()
		for (const l of launchers.value) {
			if (l.instances.length > 0) {
				newExpanded.add(l.name)
			}
		}
		expandedLaunchers.value = newExpanded
	} catch (err) {
		handleError(err)
	} finally {
		loading.value = false
		scanning.value = false
	}
}

onMounted(() => {
	scanLaunchers()
})

function getLauncherDisplayName(name: string): string {
	switch (name) {
		case 'ModrinthApp':
			return 'Modrinth App'
		case 'AstralRinth':
			return 'AstralRinth'
		case 'PrismLauncher':
			return 'Prism Launcher'
		case 'Curseforge':
			return 'CurseForge'
		default:
			return name
	}
}

function filteredInstances(launcher: LauncherInfo): string[] {
	if (!searchQuery.value.trim()) return launcher.instances
	const q = searchQuery.value.toLowerCase()
	return launcher.instances.filter((name) => name.toLowerCase().includes(q))
}

const visibleLaunchers = computed(() => {
	return launchers.value.filter((l) => filteredInstances(l).length > 0)
})

function toggleExpanded(name: string) {
	if (expandedLaunchers.value.has(name)) {
		expandedLaunchers.value.delete(name)
	} else {
		expandedLaunchers.value.add(name)
	}
	expandedLaunchers.value = new Set(expandedLaunchers.value)
}

function isInstanceSelected(launcherName: string, instanceName: string): boolean {
	return selectedInstances.value[launcherName]?.has(instanceName) ?? false
}

function toggleInstance(launcherName: string, instanceName: string) {
	if (!selectedInstances.value[launcherName]) {
		selectedInstances.value[launcherName] = new Set()
	}
	if (selectedInstances.value[launcherName].has(instanceName)) {
		selectedInstances.value[launcherName].delete(instanceName)
	} else {
		selectedInstances.value[launcherName].add(instanceName)
	}
	selectedInstances.value = { ...selectedInstances.value }
}

function getLauncherCheckState(launcher: LauncherInfo): boolean {
	const set = selectedInstances.value[launcher.name]
	if (!set || set.size === 0) return false
	return launcher.instances.length > 0 && set.size === launcher.instances.length
}

function getLauncherIndeterminate(launcher: LauncherInfo): boolean {
	const set = selectedInstances.value[launcher.name]
	if (!set || set.size === 0) return false
	return set.size < launcher.instances.length
}

function toggleLauncherAll(launcher: LauncherInfo, checked: boolean) {
	if (!selectedInstances.value[launcher.name]) {
		selectedInstances.value[launcher.name] = new Set()
	}
	if (checked) {
		for (const inst of launcher.instances) {
			selectedInstances.value[launcher.name].add(inst)
		}
	} else {
		selectedInstances.value[launcher.name].clear()
	}
	selectedInstances.value = { ...selectedInstances.value }
}

const totalSelectedCount = computed(() => {
	let count = 0
	for (const set of Object.values(selectedInstances.value)) {
		count += set.size
	}
	return count
})

function clearSelection() {
	selectedInstances.value = {}
}

async function handleBrowseCustomFolder() {
	try {
		const path = await importProvider.selectDirectory()
		if (!path) return

		const instances = await importProvider.getImportableInstances('Custom', path)
		if (!instances || instances.length === 0) {
			addNotification({
				type: 'error',
				title: formatMessage(messages.noInstancesFound),
				message: 'No importable instances were found at the specified path.',
			})
			return
		}

		const pathName = path.split(/[\\/]/).pop() || path
		const customName = formatMessage(messages.customLauncherName, { pathName })

		launchers.value.push({
			name: customName,
			path,
			instances,
		})
		expandedLaunchers.value.add(customName)
		expandedLaunchers.value = new Set(expandedLaunchers.value)

		addNotification({
			type: 'success',
			title: 'Folder added',
			message: `Found ${instances.length} instance(s) in ${pathName}`,
		})
	} catch (err) {
		handleError(err)
	}
}

async function handleImport() {
	if (totalSelectedCount.value === 0 || importing.value) return

	importing.value = true
	try {
		const selections: { launcher: string; path: string; instanceNames: string[] }[] = []
		for (const launcher of launchers.value) {
			const set = selectedInstances.value[launcher.name]
			if (set && set.size > 0) {
				selections.push({
					launcher: launcher.name,
					path: launcher.path,
					instanceNames: Array.from(set),
				})
			}
		}

		const count = totalSelectedCount.value
		await importProvider.importInstances(selections)

		addNotification({
			type: 'success',
			title: formatMessage(messages.importSuccessTitle),
			message: formatMessage(messages.importSuccessText, { count }),
		})

		clearSelection()
	} catch (err) {
		handleError(err)
	} finally {
		importing.value = false
	}
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<!-- Import section -->
		<section class="flex flex-col gap-4">
			<div class="flex flex-col gap-1">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.importSectionTitle) }}
				</h2>
				<p class="m-0 text-secondary text-sm">
					{{ formatMessage(messages.importSectionDescription) }}
				</p>
			</div>

			<!-- Actions header -->
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex items-center gap-2">
					<Button
						type="outlined"
						size="small"
						:disabled="scanning || importing"
						@click="scanLaunchers"
					>
						<SpinnerIcon v-if="scanning" class="size-4 animate-spin" />
						<RefreshCwIcon v-else class="size-4" />
						{{ formatMessage(messages.scanLaunchersButton) }}
					</Button>
					<Button
						type="outlined"
						size="small"
						:disabled="scanning || importing"
						@click="handleBrowseCustomFolder"
					>
						<FolderIcon class="size-4" />
						{{ formatMessage(messages.addCustomFolderButton) }}
					</Button>
				</div>

				<Button
					v-if="totalSelectedCount > 0"
					type="quiet"
					size="small"
					class="text-secondary"
					@click="clearSelection"
				>
					{{ formatMessage(messages.clearSelection) }}
				</Button>
			</div>

			<!-- Search bar if any launchers found -->
			<div v-if="launchers.length > 0" class="w-full">
				<Input
					v-model="searchQuery"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					class="w-full"
				>
					<template #prefix>
						<SearchIcon class="size-4 text-secondary" />
					</template>
				</Input>
			</div>

			<!-- Loading state -->
			<div
				v-if="loading"
				class="flex items-center justify-center py-10 rounded-2xl bg-surface-2 text-secondary text-sm gap-2"
			>
				<SpinnerIcon class="size-5 animate-spin" />
				<span>Scanning for installed Minecraft launchers...</span>
			</div>

			<!-- Detected Launchers List -->
			<div v-else-if="visibleLaunchers.length > 0" class="flex flex-col gap-3">
				<div
					v-for="launcher in visibleLaunchers"
					:key="launcher.name"
					class="flex flex-col rounded-2xl border border-solid border-divider bg-surface-2 overflow-hidden shadow-sm"
				>
					<!-- Launcher Header Accordion -->
					<div
						class="flex items-center justify-between p-3.5 bg-surface-3 cursor-pointer select-none transition-colors hover:bg-surface-4"
						@click="toggleExpanded(launcher.name)"
					>
						<div class="flex items-center gap-3 min-w-0">
							<ChevronRightIcon
								class="size-5 shrink-0 text-secondary transition-transform duration-150"
								:class="{ 'rotate-90': expandedLaunchers.has(launcher.name) }"
							/>
							<Checkbox
								:model-value="getLauncherCheckState(launcher)"
								:indeterminate="getLauncherIndeterminate(launcher)"
								@update:model-value="toggleLauncherAll(launcher, $event)"
								@click.stop
							/>
							<div class="flex flex-col min-w-0">
								<span class="font-semibold text-contrast text-base truncate">
									{{ getLauncherDisplayName(launcher.name) }}
								</span>
								<span class="text-xs text-secondary truncate">{{ launcher.path }}</span>
							</div>
						</div>

						<span class="text-xs font-semibold text-secondary rounded-full bg-surface-2 px-2.5 py-1">
							{{ formatMessage(messages.instancesCount, { count: launcher.instances.length }) }}
						</span>
					</div>

					<!-- Instances List (Expanded) -->
					<div
						v-if="expandedLaunchers.has(launcher.name)"
						class="flex flex-col divide-y divide-solid divide-divider border-t border-divider max-h-60 overflow-y-auto"
					>
						<div
							v-for="instance in filteredInstances(launcher)"
							:key="instance"
							class="flex items-center gap-3 px-4 py-2.5 transition-colors hover:bg-surface-3 cursor-pointer"
							@click="toggleInstance(launcher.name, instance)"
						>
							<Checkbox
								:model-value="isInstanceSelected(launcher.name, instance)"
								@update:model-value="toggleInstance(launcher.name, instance)"
								@click.stop
							/>
							<span class="font-medium text-contrast text-sm truncate">{{ instance }}</span>
						</div>
					</div>
				</div>
			</div>

			<!-- No Launchers Empty State -->
			<div
				v-else
				class="flex flex-col items-center justify-center p-8 rounded-2xl bg-surface-2 text-center text-secondary text-sm gap-2"
			>
				<span>{{ formatMessage(messages.noLaunchersDetected) }}</span>
			</div>

			<!-- Import Action Button -->
			<div v-if="launchers.length > 0" class="mt-2 flex justify-end">
				<Button
					type="colored"
					color="brand"
					size="large"
					:disabled="totalSelectedCount === 0 || importing"
					@click="handleImport"
				>
					<SpinnerIcon v-if="importing" class="size-5 animate-spin" />
					<CheckIcon v-else class="size-5" />
					{{
						importing
							? formatMessage(messages.importingButton)
							: formatMessage(messages.importSelectedButton, { count: totalSelectedCount })
					}}
				</Button>
			</div>
		</section>
	</div>
</template>
