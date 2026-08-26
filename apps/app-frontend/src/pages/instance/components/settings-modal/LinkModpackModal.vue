<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	Avatar,
	Button,
	defineMessages,
	DropdownSelect,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import {
	CheckIcon,
	DownloadIcon,
	ExternalIcon,
	LinkIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { get_project, get_project_versions } from '@/helpers/cache'
import { edit } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { instanceKeys } from '../../query-options.ts'

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	(e: 'linked', payload: { project: any; version: any }): void
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const debug = useDebugLogger('LinkModpackModal')

const modal = ref<InstanceType<typeof NewModal>>()
const searchQuery = ref('')
const searchLoading = ref(false)
const searchResults = ref<any[]>([])
const selectedProject = ref<any | null>(null)
const versionsLoading = ref(false)
const versions = ref<Labrinth.Versions.v2.Version[]>([])
const selectedVersionId = ref<string | null>(null)
const linking = ref(false)

const messages = defineMessages({
	modalHeader: {
		id: 'installation-settings.link-modpack.modal-header',
		defaultMessage: 'Link to Modrinth modpack',
	},
	searchPlaceholder: {
		id: 'installation-settings.link-modpack.search-placeholder',
		defaultMessage: 'Search modpacks or paste Modrinth URL / slug...',
	},
	searchLabel: {
		id: 'installation-settings.link-modpack.search-label',
		defaultMessage: 'Modpack search',
	},
	searchResultsTitle: {
		id: 'installation-settings.link-modpack.search-results',
		defaultMessage: 'Select a modpack',
	},
	noResults: {
		id: 'installation-settings.link-modpack.no-results',
		defaultMessage: 'No modpacks found matching your search.',
	},
	selectedModpackTitle: {
		id: 'installation-settings.link-modpack.selected-modpack',
		defaultMessage: 'Selected modpack',
	},
	changeModpackButton: {
		id: 'installation-settings.link-modpack.change-modpack',
		defaultMessage: 'Change modpack',
	},
	selectVersionLabel: {
		id: 'installation-settings.link-modpack.select-version',
		defaultMessage: 'Target modpack version',
	},
	compatibleBadge: {
		id: 'installation-settings.link-modpack.compatible-badge',
		defaultMessage: 'Compatible',
	},
	cancelButton: {
		id: 'common.cancel',
		defaultMessage: 'Cancel',
	},
	linkButton: {
		id: 'installation-settings.link-modpack.confirm-button',
		defaultMessage: 'Link modpack',
	},
	linkingButton: {
		id: 'installation-settings.link-modpack.linking-button',
		defaultMessage: 'Linking...',
	},
	linkSuccessTitle: {
		id: 'installation-settings.link-modpack.success-title',
		defaultMessage: 'Instance linked',
	},
	linkSuccessMessage: {
		id: 'installation-settings.link-modpack.success-message',
		defaultMessage: 'Instance has been linked to {name} successfully.',
	},
})

function parseModpackQuery(input: string): string {
	const trimmed = input.trim()
	const urlMatch = trimmed.match(/modrinth\.com\/modpack\/([a-zA-Z0-9_-]+)/)
	if (urlMatch) return urlMatch[1]
	return trimmed
}

let searchTimer: ReturnType<typeof setTimeout> | null = null

watch(searchQuery, (newQuery) => {
	if (searchTimer) clearTimeout(searchTimer)
	if (!newQuery.trim()) {
		searchResults.value = []
		searchLoading.value = false
		return
	}

	searchTimer = setTimeout(() => {
		performSearch(newQuery)
	}, 300)
})

async function performSearch(query: string) {
	const parsed = parseModpackQuery(query)
	if (!parsed) return

	searchLoading.value = true
	try {
		// First try direct slug/ID match if it's a single word/slug
		let directProject: any = null
		if (!parsed.includes(' ')) {
			directProject = await get_project(parsed).catch(() => null)
			if (directProject && directProject.project_type !== 'modpack') {
				directProject = null
			}
		}

		const results = await client.labrinth.projects_v3.search({
			query: parsed,
			new_filters: 'project_types = "modpack"',
			limit: 8,
		})

		const hits = results.hits.map((hit) => ({
			id: hit.project_id,
			slug: hit.slug,
			title: hit.name,
			description: hit.description,
			icon_url: hit.icon_url,
			author: hit.author,
			downloads: hit.downloads,
		}))

		if (directProject) {
			const exists = hits.some((h) => h.id === directProject.id)
			if (!exists) {
				hits.unshift({
					id: directProject.id,
					slug: directProject.slug,
					title: directProject.title,
					description: directProject.description,
					icon_url: directProject.icon_url,
					author: directProject.author ?? directProject.team,
					downloads: directProject.downloads,
				})
			}
		}

		searchResults.value = hits
	} catch (err) {
		debug('performSearch: error', err)
		searchResults.value = []
	} finally {
		searchLoading.value = false
	}
}

async function selectProject(project: any) {
	selectedProject.value = project
	searchResults.value = []
	searchQuery.value = ''
	selectedVersionId.value = null
	versionsLoading.value = true

	try {
		const fetchedVersions = ((await get_project_versions(project.id).catch(handleError)) ??
			[]) as Labrinth.Versions.v2.Version[]

		versions.value = fetchedVersions

		// Try to find a compatible version matching game_version and loader
		const instanceLoader = props.instance.loader?.toLowerCase()
		const instanceGameVersion = props.instance.game_version

		const compatible = fetchedVersions.find((v) => {
			const matchesGame = v.game_versions.includes(instanceGameVersion)
			const matchesLoader = !instanceLoader || v.loaders.some((l) => l.toLowerCase() === instanceLoader)
			return matchesGame && matchesLoader
		})

		if (compatible) {
			selectedVersionId.value = compatible.id
		} else if (fetchedVersions.length > 0) {
			selectedVersionId.value = fetchedVersions[0].id
		}
	} catch (err) {
		handleError(err)
	} finally {
		versionsLoading.value = false
	}
}

function clearSelectedProject() {
	selectedProject.value = null
	versions.value = []
	selectedVersionId.value = null
}

const versionOptions = computed(() => {
	return versions.value.map((v) => {
		const instanceLoader = props.instance.loader?.toLowerCase()
		const instanceGameVersion = props.instance.game_version
		const isCompatible =
			v.game_versions.includes(instanceGameVersion) &&
			(!instanceLoader || v.loaders.some((l) => l.toLowerCase() === instanceLoader))

		return {
			value: v.id,
			label: `${v.name || v.version_number} (${v.game_versions.slice(0, 2).join(', ')}${v.game_versions.length > 2 ? '...' : ''})`,
			isCompatible,
			version: v,
		}
	})
})

const selectedVersionOption = computed(() => {
	return (
		versionOptions.value.find((opt) => opt.value === selectedVersionId.value)?.label ??
		selectedVersionId.value ??
		''
	)
})

async function handleConfirmLink() {
	if (!selectedProject.value || !selectedVersionId.value) return

	linking.value = true
	try {
		debug('handleConfirmLink: linking instance', {
			instanceId: props.instance.id,
			projectId: selectedProject.value.id,
			versionId: selectedVersionId.value,
		})

		await edit(props.instance.id, {
			link: {
				type: 'modrinth_modpack',
				project_id: selectedProject.value.id,
				version_id: selectedVersionId.value,
			} as any,
		})

		await queryClient.invalidateQueries({
			queryKey: ['linkedModpackInfo', props.instance.id],
		})
		await queryClient.invalidateQueries({
			queryKey: instanceKeys.instance(props.instance.id),
		})

		addNotification({
			type: 'success',
			title: formatMessage(messages.linkSuccessTitle),
			message: formatMessage(messages.linkSuccessMessage, {
				name: selectedProject.value.title,
			}),
		})

		emit('linked', {
			project: selectedProject.value,
			version: versions.value.find((v) => v.id === selectedVersionId.value),
		})

		modal.value?.hide()
	} catch (err) {
		handleError(err)
	} finally {
		linking.value = false
	}
}

function show() {
	clearSelectedProject()
	searchQuery.value = ''
	searchResults.value = []
	modal.value?.show()
}

defineExpose({
	show,
})
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.modalHeader)"
		max-width="580px"
		:closable="!linking"
	>
		<div class="flex flex-col gap-5">
			<!-- If no modpack is selected yet, show search input and list -->
			<div v-if="!selectedProject" class="flex flex-col gap-3">
				<div class="relative flex items-center">
					<Input
						v-model="searchQuery"
						:placeholder="formatMessage(messages.searchPlaceholder)"
						class="w-full"
						:aria-label="formatMessage(messages.searchLabel)"
					>
						<template #prefix>
							<SpinnerIcon v-if="searchLoading" class="size-5 animate-spin text-secondary" />
							<SearchIcon v-else class="size-5 text-secondary" />
						</template>
					</Input>
				</div>

				<!-- Search results -->
				<div
					v-if="searchResults.length > 0"
					class="flex max-h-80 flex-col gap-2 overflow-y-auto rounded-xl bg-surface-2 p-2"
				>
					<button
						v-for="project in searchResults"
						:key="project.id"
						type="button"
						class="flex items-center gap-3 rounded-lg p-2.5 text-left transition hover:bg-surface-3"
						@click="selectProject(project)"
					>
						<Avatar :src="project.icon_url" :alt="project.title" size="2.5rem" no-shadow raised />
						<div class="flex min-w-0 flex-1 flex-col">
							<div class="flex items-center gap-2">
								<span class="truncate font-semibold text-contrast">{{ project.title }}</span>
								<span v-if="project.author" class="text-xs text-secondary truncate">
									by {{ project.author }}
								</span>
							</div>
							<span class="truncate text-xs text-secondary">{{ project.description }}</span>
						</div>
					</button>
				</div>
				<div
					v-else-if="searchQuery.trim() && !searchLoading"
					class="rounded-xl bg-surface-2 p-4 text-center text-sm text-secondary"
				>
					{{ formatMessage(messages.noResults) }}
				</div>
			</div>

			<!-- Selected Modpack & Version Selector -->
			<div v-else class="flex flex-col gap-4">
				<div class="flex items-center justify-between gap-3 rounded-xl bg-surface-2 p-3.5">
					<div class="flex min-w-0 items-center gap-3">
						<Avatar
							:src="selectedProject.icon_url"
							:alt="selectedProject.title"
							size="3rem"
							no-shadow
							raised
						/>
						<div class="flex min-w-0 flex-col">
							<span class="truncate font-semibold text-contrast text-base">
								{{ selectedProject.title }}
							</span>
							<span class="truncate text-xs text-secondary">{{ selectedProject.description }}</span>
						</div>
					</div>
					<Button
						type="outlined"
						size="small"
						:disabled="linking || versionsLoading"
						@click="clearSelectedProject"
					>
						{{ formatMessage(messages.changeModpackButton) }}
					</Button>
				</div>

				<div class="flex flex-col gap-2">
					<label class="text-sm font-semibold text-contrast">
						{{ formatMessage(messages.selectVersionLabel) }}
					</label>

					<div v-if="versionsLoading" class="flex items-center gap-2 text-sm text-secondary py-2">
						<SpinnerIcon class="size-4 animate-spin" />
						<span>Loading modpack versions...</span>
					</div>

					<div v-else-if="versions.length > 0" class="flex flex-col gap-2">
						<DropdownSelect
							:options="versionOptions.map((opt) => opt.value)"
							:selected-option="selectedVersionOption"
							@select="selectedVersionId = $event"
						>
							<template #default="{ selected }">
								<span class="font-medium">{{ selected }}</span>
							</template>
							<template #option="{ option }">
								<div class="flex items-center justify-between gap-2 py-1">
									<span>
										{{ versionOptions.find((o) => o.value === option)?.label ?? option }}
									</span>
									<span
										v-if="versionOptions.find((o) => o.value === option)?.isCompatible"
										class="rounded bg-green/20 px-1.5 py-0.5 text-xs font-semibold text-green"
									>
										{{ formatMessage(messages.compatibleBadge) }}
									</span>
								</div>
							</template>
						</DropdownSelect>
					</div>
					<div v-else class="text-sm text-secondary">
						No versions available for this modpack.
					</div>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" :disabled="linking" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(messages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="!selectedProject || !selectedVersionId || linking || versionsLoading"
					@click="handleConfirmLink"
				>
					<SpinnerIcon v-if="linking" class="size-5 animate-spin" />
					<LinkIcon v-else class="size-5" />
					{{ formatMessage(linking ? messages.linkingButton : messages.linkButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
