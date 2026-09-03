<template>
	<nav
		ref="scrollContainer"
		class="card-shadow relative flex w-fit overflow-x-auto rounded-full bg-bg-raised p-1 text-sm font-bold" :class="{ 'opacity-50 pointer-events-none cursor-not-allowed': disabled }"
	>
		<Button
			v-for="(option, index) in options"
			:key="`option-group-${index}`"
			ref="optionButtons"
			type="quiet"
			interaction="none"
			:disabled="disabled" class="z-[1] !h-auto !gap-2 !rounded-full !px-4 !py-2"
			:class="{
				'!text-brand': modelValue === option,
				'!text-primary': modelValue !== option,
			}"
			@click="!disabled && setOption(option)"
		>
			<slot :option="option" :selected="modelValue === option" />
		</Button>
		<div
			class="optiongroup-slider pointer-events-none absolute overflow-hidden rounded-full bg-button-bgSelected p-1"
			:class="{ 'navtabs-transition': initialized }"
			:style="{
				transform: `translate3d(${sliderX}px, ${sliderY}px, 0)`,
				width: `${sliderWidth}px`,
				height: `${sliderHeight}px`,
				opacity: initialized ? 1 : 0,
			}"
			aria-hidden="true"
		></div>
	</nav>
</template>

<script setup lang="ts" generic="T">
import { computed, onMounted, ref, watch } from 'vue'

import Button from './buttons/Button.vue'

const modelValue = defineModel<T>({ required: true })

const props = defineProps<{
	options: T[]
	disabled?: boolean
}>()

const scrollContainer = ref<HTMLElement | null>(null)

const sliderX = ref(4)
const sliderY = ref(4)
const sliderWidth = ref(0)
const sliderHeight = ref(0)

const optionButtons = ref()
const initialized = ref(false)

function setOption(option: T) {
	modelValue.value = option
}

watch(modelValue, () => {
	startAnimation(props.options.indexOf(modelValue.value))
})

function startAnimation(index: number) {
	const el = optionButtons.value[index]?.element

	if (!el || !el.offsetParent) return

	sliderX.value = el.offsetLeft
	sliderY.value = el.offsetTop
	sliderWidth.value = el.offsetWidth
	sliderHeight.value = el.offsetHeight

	initialized.value = true
}

onMounted(() => {
	startAnimation(props.options.indexOf(modelValue.value))
})
</script>

<style scoped>
.optiongroup-slider {
	top: 0;
	left: 0;
	will-change: transform, width, height;
	transform-origin: 0 0;
}

.navtabs-transition {
	transition:
		transform 180ms cubic-bezier(0.2, 0, 0, 1),
		width 180ms cubic-bezier(0.2, 0, 0, 1),
		height 180ms cubic-bezier(0.2, 0, 0, 1),
		opacity 200ms cubic-bezier(0.2, 0, 0, 1);
}

.card-shadow {
	box-shadow: var(--shadow-card);
}
</style>
