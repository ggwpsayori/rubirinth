<script setup lang="ts">
import { CurseForgeIcon, ModrinthIcon } from '@modrinth/assets'
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		source?: 'modrinth' | 'curseforge' | string | null
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
	}>(),
	{
		source: null,
		size: 'sm',
	},
)

const resolvedSource = computed<'modrinth' | 'curseforge' | null>(() => {
	if (!props.source) return null
	if (props.source === 'curseforge' || props.source.startsWith('cf:')) return 'curseforge'
	if (props.source === 'modrinth') return 'modrinth'
	return 'modrinth'
})

const sizeClasses = computed(() => {
	switch (props.size) {
		case 'xs':
			return 'size-3.5'
		case 'sm':
			return 'size-4'
		case 'md':
			return 'size-5'
		case 'lg':
			return 'size-6'
		case 'xl':
			return 'size-7'
		default:
			return 'size-4'
	}
})
</script>

<template>
	<ModrinthIcon
		v-if="resolvedSource === 'modrinth'"
		v-tooltip="'Modrinth'"
		class="shrink-0 text-[#00af5c]"
		:class="sizeClasses"
		aria-label="Modrinth"
	/>
	<CurseForgeIcon
		v-else-if="resolvedSource === 'curseforge'"
		v-tooltip="'CurseForge'"
		class="shrink-0 text-[#f16436]"
		:class="sizeClasses"
		aria-label="CurseForge"
	/>
</template>
