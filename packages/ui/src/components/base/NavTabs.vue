<template>
	<div
		v-if="filteredLinks.length > 1"
		:class="pageNav ? '-mx-6 -mt-2 mb-1 overflow-x-auto px-6 py-2' : 'contents'"
		v-bind="pageNav ? $attrs : {}"
	>
		<nav
			ref="scrollContainer"
			class="relative flex w-fit rounded-full bg-bg-raised p-1 text-xs sm:text-sm font-bold"
			:class="{
				'card-shadow border border-solid border-surface-4': mode === 'navigation',
				'tab-color-delayed': colorChangeDelayed,
			}"
			v-bind="pageNav ? {} : $attrs"
		>
			<template v-if="mode === 'navigation'">
				<RouterLink
					v-for="(link, index) in filteredLinks"
					v-show="link.shown ?? true"
					:key="link.href"
					ref="tabLinkElements"
					:replace="replace"
					:to="getTargetUrl(link)"
					class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
					:class="getSSRFallbackClasses(index)"
					@mouseenter="link.onHover?.()"
					@focus="link.onHover?.()"
					@click="handleNavClick($event, index, link)"
				>
					<component
						:is="link.icon"
						v-if="link.icon"
						class="tab-color hidden sm:block size-5"
						:class="getIconClasses(index)"
					/>
					<span class="tab-color text-nowrap" :class="getLabelClasses(index)">
						{{ link.label }}
					</span>
				</RouterLink>
			</template>

			<template v-else>
				<div
					v-for="(link, index) in filteredLinks"
					v-show="link.shown ?? true"
					:key="link.href"
					ref="tabLinkElements"
					class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 hover:cursor-pointer focus:rounded-full"
					:class="getSSRFallbackClasses(index)"
					@click="handleLocalClick(index, link)"
				>
					<component
						:is="link.icon"
						v-if="link.icon"
						class="tab-color size-5"
						:class="getIconClasses(index)"
					/>
					<span class="tab-color text-nowrap" :class="getLabelClasses(index)">
						{{ link.label }}
					</span>
				</div>
			</template>

			<div
				v-if="sliderReady && currentActiveIndex !== -1"
				class="pointer-events-none absolute rounded-full navtabs-slider"
				:class="[
					subpageSelected ? 'bg-button-bg' : 'bg-button-bgSelected',
					{ 'navtabs-transition': transitionsEnabled },
				]"
				:style="sliderStyle"
				aria-hidden="true"
			/>
		</nav>
	</div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

defineOptions({ inheritAttrs: false })

interface Tab {
	label: string
	href: string
	shown?: boolean
	icon?: Component
	subpages?: string[]
	onHover?: () => void
}

const props = withDefaults(
	defineProps<{
		replace?: boolean
		links: Tab[]
		query?: string
		mode?: 'navigation' | 'local'
		activeIndex?: number
		pageNav?: boolean
	}>(),
	{
		mode: 'navigation',
		query: undefined,
		activeIndex: undefined,
		pageNav: false,
	},
)

const emit = defineEmits<{
	tabClick: [index: number, tab: Tab]
}>()

// DOM refs
const scrollContainer = ref<HTMLElement | null>(null)
const tabLinkElements = ref<HTMLElement[]>()

const sliderX = ref(0)
const sliderY = ref(0)
const sliderWidth = ref(0)
const sliderHeight = ref(0)

const currentActiveIndex = ref(-1)
const subpageSelected = ref(false)

const sliderReady = ref(false)
const transitionsEnabled = ref(false)
const colorChangeDelayed = ref(false)

const filteredLinks = computed(() => props.links.filter((link) => link.shown ?? true))

function getTargetUrl(link: Tab) {
	if (props.query) {
		return link.href ? `?${props.query}=${link.href}` : '?'
	}
	return link.href
}

const sliderStyle = computed(() => ({
	transform: `translate3d(${sliderX.value}px, ${sliderY.value}px, 0)`,
	width: `${sliderWidth.value}px`,
	height: `${sliderHeight.value}px`,
}))

const isActiveAndNotSubpage = computed(
	() => (index: number) => currentActiveIndex.value === index && !subpageSelected.value,
)

function getSSRFallbackClasses(index: number) {
	if (sliderReady.value) return {}
	if (currentActiveIndex.value !== index) return {}

	return {
		'rounded-full': true,
		'bg-button-bgSelected': !subpageSelected.value,
		'bg-button-bg': subpageSelected.value,
	}
}

function getIconClasses(index: number) {
	return {
		'text-button-textSelected': isActiveAndNotSubpage.value(index),
		'text-secondary': !isActiveAndNotSubpage.value(index),
	}
}

function getLabelClasses(index: number) {
	return {
		'text-button-textSelected': isActiveAndNotSubpage.value(index),
		'text-contrast': !isActiveAndNotSubpage.value(index),
	}
}

function computeActiveIndex(): { index: number; isSubpage: boolean } {
	if (props.mode === 'local' && props.activeIndex !== undefined) {
		return {
			index: Math.min(props.activeIndex, filteredLinks.value.length - 1),
			isSubpage: false,
		}
	}

	for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
		const link = filteredLinks.value[i]
		const decodedPath = decodeURIComponent(route.path)
		const decodedHref = decodeURIComponent(link.href.split('?')[0])

		if (props.query) {
			const queryValue = route.query[props.query]
			if (queryValue === link.href || (!queryValue && !link.href)) {
				return { index: i, isSubpage: false }
			}
			continue
		}

		if (decodedPath === decodedHref) {
			return { index: i, isSubpage: false }
		}

		const isSubpageMatch =
			(decodedPath.startsWith(decodedHref) &&
				(decodedPath.length === decodedHref.length || decodedPath[decodedHref.length] === '/')) ||
			link.subpages?.some((subpage) => decodedPath.includes(subpage))

		if (isSubpageMatch) {
			return { index: i, isSubpage: true }
		}
	}

	return { index: -1, isSubpage: false }
}

function getTabElement(index: number): HTMLElement | null {
	if (index === -1) return null

	const container = scrollContainer.value as HTMLElement | undefined
	if (!container) return null

	const tabs = container.querySelectorAll('.button-animation')
	const element = tabs[index] as HTMLElement | undefined

	if (!element) return null

	return element
}

function getSliderPosition(targetIndex = currentActiveIndex.value) {
	const el = getTabElement(targetIndex)
	if (!el) return null

	return {
		x: el.offsetLeft,
		y: el.offsetTop,
		width: el.offsetWidth,
		height: el.offsetHeight,
	}
}

function applySliderPosition(newPosition: {
	x: number
	y: number
	width: number
	height: number
}) {
	sliderX.value = newPosition.x
	sliderY.value = newPosition.y
	sliderWidth.value = newPosition.width
	sliderHeight.value = newPosition.height
}

function positionSlider(animate = true, targetIndex = currentActiveIndex.value) {
	const newPosition = getSliderPosition(targetIndex)
	if (!newPosition) {
		return
	}

	if (!sliderReady.value) {
		applySliderPosition(newPosition)
		sliderReady.value = true
		requestAnimationFrame(() => {
			transitionsEnabled.value = true
		})
		return
	}

	if (!animate) {
		transitionsEnabled.value = false
		applySliderPosition(newPosition)
		requestAnimationFrame(() => {
			transitionsEnabled.value = true
		})
		return
	}

	applySliderPosition(newPosition)
}

let navTimer: ReturnType<typeof setTimeout> | null = null

function handleNavClick(e: MouseEvent, index: number, link: Tab) {
	if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey || e.button !== 0) {
		return
	}
	e.preventDefault()

	if (index === currentActiveIndex.value) return

	currentActiveIndex.value = index
	subpageSelected.value = false
	positionSlider(true, index)

	if (navTimer) clearTimeout(navTimer)
	const targetUrl = getTargetUrl(link)
	navTimer = setTimeout(() => {
		if (props.replace) {
			void router.replace(targetUrl)
		} else {
			void router.push(targetUrl)
		}
	}, 190)
}

function handleLocalClick(index: number, link: Tab) {
	if (index === currentActiveIndex.value) return

	currentActiveIndex.value = index
	subpageSelected.value = false
	positionSlider(true, index)

	if (navTimer) clearTimeout(navTimer)
	navTimer = setTimeout(() => {
		emit('tabClick', index, link)
	}, 190)
}

function resetSliderPosition() {
	if (!sliderReady.value || currentActiveIndex.value === -1) {
		return
	}
	positionSlider(false)
}

async function updateActiveTab() {
	await nextTick()
	const { index, isSubpage } = computeActiveIndex()
	colorChangeDelayed.value = sliderReady.value && index !== currentActiveIndex.value
	currentActiveIndex.value = index
	subpageSelected.value = isSubpage

	if (index !== -1) {
		positionSlider()
	}
}

const initialActive = computeActiveIndex()
currentActiveIndex.value = initialActive.index
subpageSelected.value = initialActive.isSubpage

let resizeObserver: ResizeObserver | undefined

onMounted(() => {
	updateActiveTab()
	resizeObserver = new ResizeObserver(resetSliderPosition)
	if (scrollContainer.value) {
		resizeObserver.observe(scrollContainer.value)
	}
})

onUnmounted(() => {
	if (navTimer) clearTimeout(navTimer)
	resizeObserver?.disconnect()
})

watch(
	() => [route.path, route.query],
	() => {
		if (props.mode === 'navigation') {
			updateActiveTab()
		}
	},
)

watch(
	() => props.activeIndex,
	() => {
		if (props.mode === 'local') {
			updateActiveTab()
		}
	},
)

watch(
	() => props.links,
	async () => {
		await nextTick()
		updateActiveTab()
	},
	{ deep: true },
)
</script>

<style scoped>
.navtabs-slider {
	top: 0;
	left: 0;
	will-change: transform, width, height;
	transform-origin: 0 0;
}

.navtabs-transition {
	transition:
		transform 180ms cubic-bezier(0.2, 0, 0, 1),
		width 180ms cubic-bezier(0.2, 0, 0, 1),
		height 180ms cubic-bezier(0.2, 0, 0, 1);
}

.tab-color {
	transition: color 120ms ease;
}

.tab-color-delayed .tab-color {
	transition-delay: 0ms;
}
</style>
