<script setup lang="ts">
import { DropdownIcon, UsersIcon } from '@modrinth/assets'
import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import FriendsList from '@/components/ui/friends/FriendsList.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useFriends } from '@/composables/use-friends'
import { get as getCreds, login as loginFlow, type ModrinthCredentials } from '@/helpers/mr_auth'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const credentials = ref<ModrinthCredentials | null>(null)
const isOpen = ref(false)

function toggleOpen() {
	isOpen.value = !isOpen.value
}

function close() {
	isOpen.value = false
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape' && isOpen.value) {
		close()
	}
}

async function fetchCredentials() {
	try {
		credentials.value = await getCreds()
	} catch {
		credentials.value = null
	}
}

const { friends } = useFriends({
	currentUserId: () => credentials.value?.user_id,
	getCredentials: () => credentials.value,
	onError: () => {},
})

const onlineCount = computed(() =>
	friends.value.filter((f) => f.online && f.accepted).length,
)

const pendingCount = computed(() =>
	friends.value.filter((f) => !f.accepted && f.id === credentials.value?.user_id).length,
)

async function signIn() {
	try {
		await loginFlow('sign-in')
		await fetchCredentials()
	} catch (error) {
		handleError(error)
	}
}

useAppEvent('mr_auth', async () => {
	await fetchCredentials()
})

onMounted(() => {
	void fetchCredentials()
	window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
	window.removeEventListener('keydown', handleKeydown)
})

const messages = defineMessages({
	friends: {
		id: 'friends.heading',
		defaultMessage: 'Friends',
	},
})
</script>

<template>
	<div class="relative flex items-center">
		<!-- Trigger button matching header style -->
		<button
			type="button"
			class="flex border-solid border-surface-5 text-sm items-center gap-2 py-1.5 px-3 rounded-xl border bg-transparent hover:bg-surface-2 cursor-pointer transition-colors text-primary font-medium select-none"
			@click="toggleOpen"
		>
			<UsersIcon class="size-4 text-brand shrink-0" />
			<span class="text-contrast font-medium">
				{{ formatMessage(messages.friends) }}
			</span>
			<span
				v-if="pendingCount > 0"
				class="flex items-center justify-center px-1.5 text-[10px] font-bold rounded-full bg-brand text-[var(--color-accent-contrast)] leading-none min-w-[16px] h-4"
			>
				{{ pendingCount }}
			</span>
			<span
				v-else-if="onlineCount > 0"
				class="flex items-center gap-1 text-xs text-green font-semibold"
			>
				<span class="size-1.5 rounded-full bg-green inline-block"></span>
				{{ onlineCount }}
			</span>
			<DropdownIcon
				class="size-3 text-secondary transition-transform shrink-0"
				:class="{ 'rotate-180': isOpen }"
			/>
		</button>

		<!-- Backdrop to close on outside click -->
		<div
			v-if="isOpen"
			class="fixed inset-0 z-40 bg-transparent"
			@click="close"
		/>

		<!-- Dropdown popup -->
		<Transition
			enter-active-class="transition duration-150 ease-out"
			enter-from-class="transform scale-95 opacity-0"
			enter-to-class="transform scale-100 opacity-100"
			leave-active-class="transition duration-100 ease-in"
			leave-from-class="transform scale-100 opacity-100"
			leave-to-class="transform scale-95 opacity-0"
		>
			<div
				v-if="isOpen"
				class="absolute right-0 top-full mt-2 z-50 flex w-[22rem] max-h-[30rem] flex-col p-3 bg-bg-raised border border-solid border-surface-5 rounded-2xl shadow-2xl overflow-y-auto"
				@click.stop
			>
				<FriendsList
					:credentials="credentials"
					:sign-in="signIn"
				/>
			</div>
		</Transition>
	</div>
</template>
