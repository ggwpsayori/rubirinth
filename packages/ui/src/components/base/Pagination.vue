<template>
	<div v-if="count > 1" class="flex items-center gap-1 shrink-0">
		<ButtonLink
			v-if="linkFunction"
			v-tooltip="page > 1 ? formatMessage(messages.previousPage) : undefined"
			:aria-label="formatMessage(messages.previousPage)"
			:href="page > 1 ? linkFunction(page - 1) : undefined"
			:disabled="page <= 1"
			type="quiet"
			class="!w-9 !h-9 !px-0 !rounded-full shrink-0 justify-center"
			@click.prevent="page > 1 ? switchPage(page - 1) : null"
		>
			<ChevronLeftIcon aria-hidden="true" />
		</ButtonLink>
		<IconButton
			v-else
			v-tooltip="page > 1 ? formatMessage(messages.previousPage) : undefined"
			:label="formatMessage(messages.previousPage)"
			:disabled="page <= 1"
			type="quiet"
			class="!w-9 !h-9 !rounded-full shrink-0 justify-center"
			@click="page > 1 ? switchPage(page - 1) : null"
		>
			<ChevronLeftIcon aria-hidden="true" />
		</IconButton>

		<div
			v-for="(item, index) in pages"
			:key="'page-' + item + '-' + index"
			:class="{
				'page-number': page !== item,
				shrink: typeof item === 'number' && item > 99,
			}"
			class="page-number-container flex items-center justify-center !w-9 !h-9 shrink-0"
		>
			<form v-if="item === '-'" class="grid place-content-center w-full h-full" @submit.prevent="goToPage">
				<Input
					v-if="showPageInput === index"
					:ref="focusInput"
					v-model="pageInput"
					v-tooltip="formatMessage(messages.goToPage)"
					type="number"
					:min="1"
					:max="props.count"
					placeholder="..."
					clamp
					class="w-14"
					:aria-label="formatMessage(messages.goToPage)"
					@focusout="showPageInput = undefined"
					@keydown.escape="showPageInput = undefined"
				/>

				<div v-else class="rotate-90 flex items-center justify-center w-full h-full">
					<button
						v-tooltip="formatMessage(messages.goToPage)"
						type="button"
						:aria-label="formatMessage(messages.goToPage)"
						class="grid place-content-center size-8 rounded-full border-0 bg-transparent text-secondary hover:text-primary cursor-pointer transition-colors"
						@click="openPageInput(index)"
					>
						<EllipsisVerticalIcon aria-hidden="true" />
					</button>
				</div>
			</form>
			<template v-else>
				<ButtonLink
					v-if="linkFunction"
					:href="linkFunction(item)"
					type="quiet"
					:color="page === item ? 'brand' : undefined"
					:interaction="page === item ? 'filled' : undefined"
					:aria-current="page === item ? 'page' : undefined"
					:class="['!w-9 !h-9 !px-0 !rounded-full justify-center', page === item ? '!bg-brand-highlight' : '']"
					@click.prevent="page !== item ? switchPage(item) : null"
				>
					{{ item }}
				</ButtonLink>
				<Button
					v-else
					type="quiet"
					:color="page === item ? 'brand' : undefined"
					:interaction="page === item ? 'filled' : undefined"
					:aria-current="page === item ? 'page' : undefined"
					:class="['!w-9 !h-9 !px-0 !rounded-full justify-center', page === item ? '!bg-brand-highlight' : '']"
					@click="page !== item ? switchPage(item) : null"
				>
					{{ item }}
				</Button>
			</template>
		</div>

		<ButtonLink
			v-if="linkFunction"
			v-tooltip="page < count ? formatMessage(messages.nextPage) : undefined"
			:aria-label="formatMessage(messages.nextPage)"
			:href="page < count ? linkFunction(page + 1) : undefined"
			:disabled="page >= count"
			type="quiet"
			class="!w-9 !h-9 !px-0 !rounded-full shrink-0 justify-center"
			@click.prevent="page < count ? switchPage(page + 1) : null"
		>
			<ChevronRightIcon aria-hidden="true" />
		</ButtonLink>
		<IconButton
			v-else
			v-tooltip="page < count ? formatMessage(messages.nextPage) : undefined"
			:label="formatMessage(messages.nextPage)"
			:disabled="page >= count"
			type="quiet"
			class="!w-9 !h-9 !rounded-full shrink-0 justify-center"
			@click="page < count ? switchPage(page + 1) : null"
		>
			<ChevronRightIcon aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon, EllipsisVerticalIcon } from '@modrinth/assets'
import { type ComponentPublicInstance, computed, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n.ts'

import { Button, ButtonLink, IconButton } from './buttons'
import Input from './inputs/Input.vue'

const emit = defineEmits<{
	'switch-page': [page: number]
}>()

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		page: number
		count: number
		linkFunction?: (page: number) => string | undefined
	}>(),
	{
		page: 1,
		count: 1,
	},
)
const showPageInput = ref<number | undefined>(undefined)
const pageInput = ref<number | undefined>(undefined)

const pages = computed<Array<number | '-'>>(() => {
	const last = Math.max(1, props.count || 1)
	const current = Math.min(Math.max(1, props.page || 1), last)

	if (last <= 7) {
		const res: Array<number | '-'> = []
		for (let i = 1; i <= last; i++) res.push(i)
		return res
	}

	if (current <= 4) {
		return [1, 2, 3, 4, 5, '-', last]
	}

	if (current >= last - 3) {
		return [1, '-', last - 4, last - 3, last - 2, last - 1, last]
	}

	return [1, '-', current - 1, current, current + 1, '-', last]
})

function switchPage(newPage: number) {
	emit('switch-page', Math.min(Math.max(newPage, 1), props.count))
}

function focusInput(element: Element | ComponentPublicInstance | null) {
	if (element && 'focus' in element) {
		const input = element as InstanceType<typeof Input>
		input.focus()
	}
}

function openPageInput(index: number) {
	pageInput.value = undefined
	showPageInput.value = index
}

function goToPage() {
	if (pageInput.value !== undefined && pageInput.value >= 1 && pageInput.value <= props.count) {
		switchPage(pageInput.value)
	}

	showPageInput.value = undefined
	pageInput.value = undefined
}

const messages = defineMessages({
	goToPage: {
		id: 'ui.pagination.go-to-page',
		defaultMessage: 'Go to page',
	},
	previousPage: {
		id: 'ui.pagination.previous-page',
		defaultMessage: 'Previous page',
	},
	nextPage: {
		id: 'ui.pagination.next-page',
		defaultMessage: 'Next page',
	},
})
</script>
