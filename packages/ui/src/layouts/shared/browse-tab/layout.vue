<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { LayoutGridIcon, ListIcon, RotateCounterClockwiseIcon, SearchIcon } from '@modrinth/assets'
import { computed, ref, toValue } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button, IconButton } from '#ui/components/base/buttons'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import LoadingIndicator from '#ui/components/base/LoadingIndicator.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import Pagination from '#ui/components/base/Pagination.vue'
import ProjectCard from '#ui/components/project/card/ProjectCard.vue'
import ProjectCardList from '#ui/components/project/ProjectCardList.vue'
import SearchFilterControl from '#ui/components/search/SearchFilterControl.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { commonMessages, formatProjectTypeSentence, getProjectTypeCategoryMessage } from '#ui/utils/common-messages'
import type { SortType } from '#ui/utils/search'

import SelectedProjectsFloatingBar from './components/SelectedProjectsFloatingBar.vue'
import BrowseInstallHeader from './header.vue'
import { injectBrowseManager } from './providers/browse-manager'
import type { CardAction } from './types'

const ctx = injectBrowseManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	sortRelevance: {
		id: 'browse.sort.relevance',
		defaultMessage: 'Relevance',
	},
	sortDownloads: {
		id: 'browse.sort.downloads',
		defaultMessage: 'Downloads',
	},
	sortFollows: {
		id: 'browse.sort.follows',
		defaultMessage: 'Followers',
	},
	sortDatePublished: {
		id: 'browse.sort.date-published',
		defaultMessage: 'Date published',
	},
	sortDateUpdated: {
		id: 'browse.sort.date-updated',
		defaultMessage: 'Date updated',
	},
	sortVerifiedPlays: {
		id: 'browse.sort.verified-plays',
		defaultMessage: 'Verified Plays',
	},
	sortPlayers: {
		id: 'browse.sort.players',
		defaultMessage: 'Players',
	},
	listView: {
		id: 'browse.view-mode.list',
		defaultMessage: 'List',
	},
	gridView: {
		id: 'browse.view-mode.grid',
		defaultMessage: 'Cards',
	},
	searchPlaceholder: {
		id: 'browse.search.placeholder',
		defaultMessage: 'Search {projectType}...',
	},
	viewPrefix: {
		id: 'browse.view-prefix',
		defaultMessage: 'View:',
	},
	filterResults: {
		id: 'browse.filter-results',
		defaultMessage: 'Filter results...',
	},
	offline: {
		id: 'browse.offline',
		defaultMessage: 'You are currently offline. Connect to the internet to browse Rubirinth!',
	},
	noResults: {
		id: 'browse.no-results',
		defaultMessage: 'No results found for your query!',
	},
	linkOverridingPreferences: {
		id: 'browse.advanced-filters.link-overriding-preferences',
		defaultMessage: "This link's filters differ from your saved advanced exclusions",
	},
	applySavedPreferences: {
		id: 'browse.advanced-filters.apply-saved-preferences',
		defaultMessage: 'Apply saved preferences',
	},
})
const lockedMessages = computed(() => toValue(ctx.lockedFilterMessages))
const stickyInstallHeaderRef = ref<HTMLElement | null>(null)
const { isStuck: isInstallHeaderStuck } = useStickyObserver(
	stickyInstallHeaderRef,
	'BrowseInstallHeader',
)

function toggleDisplayMode() {
	if (ctx.displayMode) {
		ctx.displayMode.value = ctx.effectiveLayout.value === 'list' ? 'grid' : 'list'
	} else if (ctx.cycleDisplayMode) {
		ctx.cycleDisplayMode()
	}
}

function getSortLabel(st?: SortType | null): string {
	if (!st) return ''
	const name = st.name || ''
	const display = st.display || ''

	if (name === 'relevance' || display === 'Relevance') return formatMessage(messages.sortRelevance)
	if (name === 'downloads' || display === 'Downloads') return formatMessage(messages.sortDownloads)
	if (name === 'follows' || display === 'Followers') return formatMessage(messages.sortFollows)
	if (name === 'newest' || name === 'date_created' || display === 'Date Published' || display === 'Date published') return formatMessage(messages.sortDatePublished)
	if (name === 'updated' || name === 'date_modified' || display === 'Date Updated' || display === 'Date updated') return formatMessage(messages.sortDateUpdated)
	if (name === 'minecraft_java_server.verified_plays_2w' || display === 'Verified Plays') return formatMessage(messages.sortVerifiedPlays)
	if (name === 'minecraft_java_server.ping.data.players_online' || name === 'players' || display === 'Players' || display === 'Online players') return formatMessage(messages.sortPlayers)

	return display || name
}

const sortOptions = computed<ComboboxOption<SortType>[]>(() =>
	(ctx.effectiveSortTypes?.value ?? []).map((st) => ({
		value: st,
		label: getSortLabel(st),
	})),
)

const maxResultsOptions = computed<ComboboxOption<number>[]>(() =>
	(ctx.maxResultsOptions?.value ?? [5, 10, 15, 20, 50, 100]).map((n) => ({
		value: n,
		label: String(n),
	})),
)



function cardActionType(action: CardAction) {
	if (action.type === 'transparent') return 'quiet'
	if (action.type === 'outlined') return 'outlined'
	return action.color && action.color !== 'standard' ? 'colored' : 'base'
}

function cardActionColor(action: CardAction) {
	const type = cardActionType(action)
	return type === 'colored' || type === 'quiet' ? action.color : undefined
}

function cardActionClass(action: CardAction) {
	if (action.type !== 'outlined' || !action.color || action.color === 'standard') return undefined

	return {
		brand: '!text-brand [&>svg]:!text-brand !shadow-[inset_0_0_0_1px_var(--color-brand)]',
		red: '!text-red [&>svg]:!text-red !shadow-[inset_0_0_0_1px_var(--color-red)]',
		green: '!text-green [&>svg]:!text-green !shadow-[inset_0_0_0_1px_var(--color-green)]',
	}[action.color]
}

function getLoaderFieldValues(
	result: Labrinth.Search.v3.ResultSearchProject,
	field: string,
): string[] {
	return (result.project_loader_fields?.[field] ?? []).filter(
		(value): value is string => typeof value === 'string',
	)
}

function getProjectCardTags(result: Labrinth.Search.v3.ResultSearchProject, displayOnly: boolean) {
	const tags = new Set(displayOnly ? result.display_categories : result.categories)

	for (const loader of result.loaders) {
		if (loader !== 'mrpack') {
			tags.add(loader)
		}
	}

	if (result.loaders.includes('mrpack')) {
		for (const loader of getLoaderFieldValues(result, 'mrpack_loaders')) {
			tags.add(loader)
		}
	}

	return Array.from(tags)
}
</script>

<template>
	<template v-if="ctx.installContext?.value && ctx.variant !== 'web'">
		<div
			ref="stickyInstallHeaderRef"
			class="sticky top-0 z-20 -mx-6 -mt-6 rounded-tl-[--radius-xl] border-0 border-b border-solid bg-surface-1 px-6 py-4 border-surface-5"
			:class="[isInstallHeaderStuck ? 'border-t' : '']"
		>
			<BrowseInstallHeader />
		</div>
	</template>
	<SelectedProjectsFloatingBar v-if="ctx.installContext?.value && ctx.variant !== 'web'" />

	<div class="flex flex-wrap items-center justify-between gap-3">
		<NavTabs
			v-if="ctx.showProjectTypeTabs.value"
			:links="ctx.selectableProjectTypes.value"
			:replace="ctx.variant === 'app'"
		/>
		<div v-else />
		<slot name="header-controls" />
	</div>

	<Input
		v-model="ctx.query.value"
		:icon="SearchIcon"
		type="text"
		autocomplete="off"
		:placeholder="
			formatMessage(messages.searchPlaceholder, {
				projectType: formatMessage(
					getProjectTypeCategoryMessage(ctx.projectType.value),
				).toLowerCase(),
			})
		"
		clearable
		wrapper-class="w-full"
		size="large"
		@clear="ctx.clearSearch()"
	/>

	<Admonition
		v-if="ctx.linkOverridesAdvancedPrefs.value"
		type="info"
		:header="formatMessage(messages.linkOverridingPreferences)"
		inline-actions
		center-content
	>
		<template #actions>
			<Button type="colored" color="blue" @click="ctx.applySavedAdvancedPrefs()">
				<RotateCounterClockwiseIcon />
				{{ formatMessage(messages.applySavedPreferences) }}
			</Button>
		</template>
	</Admonition>

	<div class="flex flex-wrap items-center gap-2">
		<Combobox
			:model-value="ctx.effectiveCurrentSortType.value"
			:options="sortOptions"
			trigger-type="base"
			:class="
				ctx.variant === 'web'
					? '!w-[16rem] min-w-max max-w-full flex-grow md:flex-grow-0'
					: '!w-[16rem] min-w-max max-w-full'
			"
			@update:model-value="(val: SortType) => (ctx.effectiveCurrentSortType.value = val)"
		>
			<template #prefix>
				<span class="font-semibold text-primary">{{
					formatMessage(commonMessages.sortByLabel)
				}}</span>
			</template>
		</Combobox>

		<Combobox
			:model-value="ctx.maxResults.value"
			:options="maxResultsOptions"
			trigger-type="base"
			:class="
				ctx.variant === 'web'
					? '!w-[9rem] min-w-max max-w-full flex-grow md:flex-grow-0'
					: '!w-[9rem] min-w-max max-w-full'
			"
			:placeholder="formatMessage(commonMessages.viewLabel)"
			@update:model-value="(val: number) => (ctx.maxResults.value = val)"
		>
			<template #prefix>
				<span class="font-semibold text-primary">{{ formatMessage(messages.viewPrefix) }}</span>
			</template>
		</Combobox>

		<div v-if="ctx.filtersMenuOpen && !ctx.filtersMenuOpen.value" class="lg:hidden">
			<Button @click="ctx.filtersMenuOpen.value = true">
				{{ formatMessage(messages.filterResults) }}
			</Button>
		</div>

		<IconButton
			v-if="ctx.cycleDisplayMode || ctx.displayMode"
			:circular="false"
			type="base"
			size="md"
			:label="ctx.effectiveLayout.value === 'list' ? formatMessage(messages.gridView) : formatMessage(messages.listView)"
			v-tooltip="ctx.effectiveLayout.value === 'list' ? formatMessage(messages.gridView) : formatMessage(messages.listView)"
			class="!h-9 !w-9 !rounded-xl shrink-0"
			@click="toggleDisplayMode"
		>
			<LayoutGridIcon v-if="ctx.effectiveLayout.value === 'list'" class="size-5" />
			<ListIcon v-else class="size-5" />
		</IconButton>

		<Pagination
			:page="ctx.currentPage.value"
			:count="ctx.pageCount.value"
			:class="ctx.variant === 'web' ? 'mx-auto sm:ml-auto sm:mr-0' : 'ml-auto'"
			@switch-page="ctx.setPage"
		/>
	</div>

	<SearchFilterControl
		v-if="ctx.isServerType.value"
		v-model:selected-filters="ctx.serverCurrentFilters.value"
		:filters="(ctx.serverFilterTypes?.value ?? []).filter((f) => f.display !== 'none' && f.id !== 'server_status' && !(ctx.hiddenFilterTypes?.value ?? []).includes(f.id))"
		:project-type="ctx.projectType.value"
		:provided-filters="[]"
		:overridden-provided-filter-types="[]"
	/>
	<SearchFilterControl
		v-else
		v-model:selected-filters="ctx.currentFilters.value"
		:filters="
			(ctx.filters?.value ?? []).filter(
				(f) => f.display !== 'none' && !(ctx.hiddenFilterTypes?.value ?? []).includes(f.id),
			)
		"
		:project-type="ctx.projectType.value"
		:provided-filters="ctx.providedFilters?.value ?? []"
		:overridden-provided-filter-types="ctx.overriddenProvidedFilterTypes.value"
		:provided-message="lockedMessages?.providedBy"
	/>

	<div class="search mt-1 [overflow-anchor:none]">
		<section v-if="ctx.loading.value" class="offline">
			<component :is="ctx.loadingComponent ?? LoadingIndicator" />
		</section>
		<section v-else-if="ctx.offline?.value && ctx.totalHits.value === 0" class="offline">
			{{ formatMessage(messages.offline) }}
		</section>
		<section
			v-else-if="
				ctx.isServerType.value
					? ctx.serverHits.value.length === 0
					: ctx.projectHits.value.length === 0
			"
			class="offline"
		>
			<p>{{ formatMessage(messages.noResults) }}</p>
		</section>

		<ProjectCardList v-else :layout="ctx.effectiveLayout.value">
			<template v-if="ctx.isServerType.value">
				<ProjectCard
					v-for="result in ctx.serverHits.value"
					:key="`server-card-${result.project_id}`"
					:title="result.name"
					:icon-url="result.icon_url || undefined"
					:summary="result.summary"
					:tags="result.categories"
					:link="ctx.getServerProjectLink(result)"
					:server-online-players="result.minecraft_java_server?.ping?.data?.players_online ?? 0"
					:server-region="result.minecraft_server?.region"
					:server-recent-plays="result.minecraft_java_server?.verified_plays_2w ?? 0"
					:server-modpack-content="ctx.getServerModpackContent?.(result)"
					:server-ping="ctx.serverPings?.value?.[result.project_id]"
					:server-status-online="!!result.minecraft_java_server?.ping?.data"
					:hide-online-players-label="ctx.variant === 'app'"
					:hide-recent-plays-label="ctx.variant === 'app'"
					:layout="ctx.effectiveLayout.value"
					:max-tags="2"
					is-server-project
					exclude-loaders
					:color="result.color ?? undefined"
					:banner="result.featured_gallery ?? undefined"
					@contextmenu.prevent.stop="(event: MouseEvent) => ctx.onContextMenu?.(event, result)"
					@mouseenter="ctx.onServerProjectHover?.(result)"
					@mouseleave="ctx.onProjectHoverEnd?.()"
				>
					<template v-if="ctx.getCardActions?.(result, ctx.projectType.value)?.length" #actions>
						<div class="flex items-center gap-2 w-full">
							<template
								v-for="action in ctx.getCardActions(result, ctx.projectType.value)"
								:key="action.key"
							>
								<IconButton
									v-if="action.circular"
									v-tooltip="action.tooltip"
									:type="cardActionType(action)"
									:color="cardActionColor(action)"
									:class="['!shrink-0', cardActionClass(action)]"
									:label="action.label || action.tooltip || action.key"
									:disabled="action.disabled"
									@click.stop="action.onClick"
								>
									<component :is="action.icon" :class="action.iconClass" />
								</IconButton>
								<Button
									v-else
									v-tooltip="action.tooltip"
									:type="cardActionType(action)"
									:color="cardActionColor(action)"
									:class="['!flex-1 !min-w-0 justify-center', cardActionClass(action)]"
									:disabled="action.disabled"
									@click.stop="action.onClick"
								>
									<component :is="action.icon" :class="action.iconClass" />
									{{ action.label }}
								</Button>
							</template>
						</div>
					</template>
				</ProjectCard>
			</template>
			<template v-else>
				<ProjectCard
					v-for="result in ctx.projectHits.value"
					:key="result.project_id"
					:link="ctx.getProjectLink(result)"
					:title="result.name"
					:icon-url="result.icon_url ?? undefined"
					:author="{
						name: result.organization == null ? result.author : result.organization,
						link:
							result.project_id?.startsWith('cf:')
								? (result as any).author_url || `https://www.curseforge.com/members/${encodeURIComponent(result.author)}`
								: result.organization_id == null
									? `/user/${encodeURIComponent(result.author_id ?? result.author)}`
									: ctx.variant === 'web'
										? `/organization/${result.organization_id}`
										: `https://modrinth.com/organization/${result.organization_id}`,
					}"
					:date-updated="result.date_modified"
					:date-published="result.date_created"
					:displayed-date="
						ctx.effectiveCurrentSortType.value.name === 'newest' ? 'published' : 'updated'
					"
					:downloads="result.downloads"
					:summary="result.summary"
					:tags="getProjectCardTags(result, true)"
					:all-tags="getProjectCardTags(result, false)"
					:deprioritized-tags="ctx.deprioritizedTags.value"
					:exclude-loaders="ctx.excludeLoaders.value"
					:followers="result.project_id.startsWith('cf:') ? undefined : result.follows"
					:banner="result.featured_gallery ?? undefined"
					:color="result.color ?? undefined"
					:environment="
						['mod', 'modpack'].includes(ctx.projectType.value)
							? result.project_loader_fields?.environment?.[0]
							: undefined
					"
					:layout="ctx.effectiveLayout.value"
					@contextmenu.prevent.stop="(event: MouseEvent) => ctx.onContextMenu?.(event, result)"
					@mouseenter="ctx.onProjectHover?.(result)"
					@mouseleave="ctx.onProjectHoverEnd?.()"
				>
					<template v-if="ctx.getCardActions?.(result, ctx.projectType.value)?.length" #actions>
						<div class="flex items-center gap-2 w-full">
							<template
								v-for="action in ctx.getCardActions(result, ctx.projectType.value)"
								:key="action.key"
							>
								<IconButton
									v-if="action.circular"
									v-tooltip="action.tooltip"
									:type="cardActionType(action)"
									:color="cardActionColor(action)"
									:class="['!shrink-0', cardActionClass(action)]"
									:label="action.label || action.tooltip || action.key"
									:disabled="action.disabled"
									@click.stop="action.onClick"
								>
									<component :is="action.icon" :class="action.iconClass" />
								</IconButton>
								<Button
									v-else
									v-tooltip="action.tooltip"
									:type="cardActionType(action)"
									:color="cardActionColor(action)"
									:class="['!flex-1 !min-w-0 justify-center', cardActionClass(action)]"
									:disabled="action.disabled"
									@click.stop="action.onClick"
								>
									<component :is="action.icon" :class="action.iconClass" />
									{{ action.label }}
								</Button>
							</template>
						</div>
					</template>
				</ProjectCard>
			</template>
		</ProjectCardList>

		<div :class="ctx.variant === 'web' ? 'pagination-after mt-3' : 'flex justify-end mt-3'">
			<Pagination
				:page="ctx.currentPage.value"
				:count="ctx.pageCount.value"
				:class="ctx.variant === 'web' ? 'justify-end' : 'pagination-after'"
				@switch-page="ctx.setPage"
			/>
		</div>
	</div>

	<slot name="after" />
</template>
