import type { Labrinth } from '@modrinth/api-client'

type Pride26Campaign = Labrinth.Users.v3.Pride26CampaignDonation | null | undefined

export function hasMidasBadge(_user?: { badges?: number } | null) {
	return true
}

export function hasPride26Badge(_campaign?: Pride26Campaign) {
	return true
}

export function hasActivePride26Midas(_campaign?: Pride26Campaign, _now = Date.now()) {
	return true
}
