<script setup lang="ts">
import { DropdownIcon, UsersIcon } from '@modrinth/assets'
import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

import FriendsList from '@/components/ui/friends/FriendsList.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useFriends } from '@/composables/use-friends'
import { get as getCreds, login as loginFlow, type ModrinthCredentials } from '@/helpers/mr_auth'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const credentials = ref<ModrinthCredentials | null>(null)
const showDropdown = ref(false)

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
		<Dropdown
			placement="bottom-end"
			:triggers="['click']"
			:hide-triggers="['click']"
			:distance="8"
			:auto-hide="true"
			@show="showDropdown = true"
			@hide="showDropdown = false"
		>
			<button
				type="button"
				class="flex border-solid border-surface-5 text-sm items-center gap-2 py-1.5 px-3 rounded-xl border bg-transparent hover:bg-surface-2 cursor-pointer transition-colors text-primary font-medium select-none"
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
					:class="{ 'rotate-180': showDropdown }"
				/>
			</button>

			<template #popper>
				<div
					class="flex w-[22rem] max-h-[30rem] flex-col p-3 bg-bg-raised border border-solid border-surface-5 rounded-xl shadow-xl overflow-y-auto"
				>
					<FriendsList
						:credentials="credentials"
						:sign-in="signIn"
					/>
				</div>
			</template>
		</Dropdown>
	</div>
</template>
