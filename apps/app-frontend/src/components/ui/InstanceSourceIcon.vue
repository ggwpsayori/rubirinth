<script setup lang="ts">
import { CurseForgeIcon, ModrinthIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { getInstanceModpackSource } from '@/helpers/instance'
import type { InstanceModpackSource } from '@/helpers/instance'

const props = withDefaults(
	defineProps<{
		instance?: { link?: any } | null
		source?: InstanceModpackSource
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
	}>(),
	{
		instance: null,
		source: null,
		size: 'sm',
	},
)

const resolvedSource = computed<InstanceModpackSource>(() => {
	if (props.source) return props.source
	if (props.instance) return getInstanceModpackSource(props.instance)
	return null
})

const sizeClasses = computed(() => {
	switch (props.size) {
		case 'xs':
			return 'size-3.5'
		case 'sm':
			return 'size-4'
		case 'md':
			return 'size-4.5'
		case 'lg':
			return 'size-5'
		case 'xl':
			return 'size-6'
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
