<script setup lang="ts">
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

type ModalHandle = {
	hide: () => void
	show: () => void
}

const props = defineProps<{
	offlineLoginDisabled: boolean
	offlinePlayerName: string
}>()

const emit = defineEmits<{
	(event: 'submit-offline'): void
	(event: 'update:offlinePlayerName', value: string): void
}>()

const { formatMessage } = useVIntl()
const addOfflineModal = ref<ModalHandle | null>(null)

const messages = defineMessages({
	loginAction: {
		id: 'astralrinth.app.minecraft-account.input.login-action',
		defaultMessage: 'Login',
	},
	addOfflineHeader: {
		id: 'astralrinth.app.minecraft-account.input.offline.header',
		defaultMessage: 'Add new offline account',
	},
	offlineNameLabel: {
		id: 'astralrinth.app.minecraft-account.input.offline.name.label',
		defaultMessage: 'Enter your player name',
	},
	offlineNamePlaceholder: {
		id: 'astralrinth.app.minecraft-account.input.offline.name.placeholder',
		defaultMessage: 'Your player name here...',
	},
})

defineExpose({
	hideOffline: () => addOfflineModal.value?.hide(),
	showOffline: () => addOfflineModal.value?.show(),
})
</script>

<template>
	<ModalWrapper
		ref="addOfflineModal"
		class="modal"
		:header="formatMessage(messages.addOfflineHeader)"
	>
		<div class="flex flex-col gap-4 px-6 py-5">
			<label class="label form-label">{{ formatMessage(messages.offlineNameLabel) }}</label>
			<input
				:value="props.offlinePlayerName"
				type="text"
				:placeholder="formatMessage(messages.offlineNamePlaceholder)"
				class="input soft-input"
				@input="emit('update:offlinePlayerName', ($event.target as HTMLInputElement).value)"
			/>
			<div class="mt-6 ml-auto">
				<Button
					color="primary"
					:disabled="props.offlineLoginDisabled"
					@click="emit('submit-offline')"
				>
					{{ formatMessage(messages.loginAction) }}
				</Button>
			</div>
		</div>
	</ModalWrapper>
</template>

<style scoped lang="scss">
@import '../../../../../../../../packages/assets/styles/astralrinth/soft-inputs.scss';

.modal {
	position: absolute;
}
</style>
