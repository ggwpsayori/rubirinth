<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import ButtonFrame from './ButtonFrame.vue'
import type { ButtonNativeType } from './types'

withDefaults(
	defineProps<{
		checked: boolean
		disabled?: boolean
		nativeType?: ButtonNativeType
	}>(),
	{
		disabled: false,
		nativeType: 'button',
	},
)

const frame = ref<InstanceType<typeof ButtonFrame> | null>(null)
const element = computed(() => frame.value?.element ?? null)

defineExpose({ element })
</script>

<template>
	<ButtonFrame
		ref="frame"
		as="button"
		type="quiet"
		size="lg"
		:disabled="disabled"
		:native-type="nativeType"
		role="radio"
		:aria-checked="checked"
		class="w-full !justify-between !gap-4 !whitespace-normal !border !border-solid !px-3 !text-left !transition-all !duration-200 !ease-out cursor-pointer active:scale-[0.985]"
		:class="
			checked
				? '!border-brand !bg-brand-highlight !text-contrast'
				: '!border-transparent !bg-transparent !text-contrast enabled:hover:!bg-surface-3 enabled:active:!bg-surface-4'
		"
	>
		<span class="flex min-w-0 flex-1 items-center gap-2">
			<slot />
		</span>
		<span
			class="relative flex size-6 shrink-0 items-center justify-center rounded-full border border-solid transition-all duration-200 ease-out"
			:class="
				checked
					? 'border-brand bg-brand text-brand-inverted scale-100'
					: 'border-surface-5 bg-transparent text-transparent scale-95'
			"
		>
			<CheckIcon
				class="size-4 transition-all duration-200 ease-out"
				:class="checked ? 'opacity-100 scale-100' : 'opacity-0 scale-50'"
				aria-hidden="true"
			/>
		</span>
	</ButtonFrame>
</template>
