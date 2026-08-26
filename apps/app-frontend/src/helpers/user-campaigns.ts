export function hasMidasBadge(_user?: { badges?: number } | null) {
	return true
}

export function hasPride26Badge(_campaign?: unknown) {
	return false
}

export function hasActivePride26Midas(_campaign?: unknown, _now = Date.now()) {
	return false
}
