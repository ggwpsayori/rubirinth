<template>
	<PageHeaderMetadata>
		<PageHeaderMetadataItem v-if="loadingServerPing && playersOnline !== undefined">
			<ServerOnlinePlayers :online="playersOnline" :status-online="statusOnline" hide-label />
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem
			v-if="minecraftServer?.region || (loadingServerPing && ping !== undefined)"
		>
			<ServerRegion v-if="minecraftServer?.region" :region="minecraftServer.region" />
			<ServerPing
				v-if="loadingServerPing && ping !== undefined"
				:ping="ping"
				:status-online="statusOnline"
			/>
		</PageHeaderMetadataItem>
		<PageHeaderMetadataItem
			v-if="showInstancePlayTime && playtimeLabel"
			:icon="TimerIcon"
			:tooltip="formatMessage(messages.totalPlaytimeTooltip)"
		>
			{{ playtimeLabel }}
		</PageHeaderMetadataItem>
	</PageHeaderMetadata>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { TimerIcon } from '@modrinth/assets'
import {
	defineMessages,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	ServerOnlinePlayers,
	ServerPing,
	ServerRegion,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	totalPlaytimeTooltip: {
		id: 'instance.metadata.playtime.tooltip',
		defaultMessage: 'Total playtime',
	},
})

defineProps<{
	loadingServerPing?: boolean
	playersOnline?: number
	statusOnline?: boolean
	ping?: number
	minecraftServer?: Labrinth.Projects.v3.Project['minecraft_server']
	showInstancePlayTime?: boolean
	playtimeLabel?: string
}>()
</script>
