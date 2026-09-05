import { invoke } from '@tauri-apps/api/core'
import {
	curseforgeClient,
	extractCurseforgeId,
	isCurseforgeId,
	mapCurseforgeFileToVersion,
	mapCurseforgeModToProject,
	mapCurseforgeModToSearchV3Project,
} from '@modrinth/api-client'

curseforgeClient.setRequester(async (path, options = {}) => {
	const body = options.body
		? typeof options.body === 'string'
			? JSON.parse(options.body)
			: options.body
		: undefined
	return await invoke('plugin:cache|curseforge_request', {
		path,
		method: options.method || 'GET',
		body,
	})
})

export async function get_project(id, cacheBehaviour) {
	if (isCurseforgeId(id)) {
		const modId = extractCurseforgeId(id)
		const [mod, body] = await Promise.all([
			curseforgeClient.getMod(modId),
			curseforgeClient.getModDescription(modId).catch(() => ''),
		])
		return mapCurseforgeModToProject(mod, body)
	}
	try {
		return await invoke('plugin:cache|get_project', { id, cacheBehaviour })
	} catch (err) {
		if (typeof id === 'string' && !id.includes(' ')) {
			try {
				const cfRes = await curseforgeClient.searchMods({ slug: id.toLowerCase(), pageSize: 1 })
				if (cfRes?.data?.[0]) {
					const mod = cfRes.data[0]
					const body = await curseforgeClient.getModDescription(mod.id).catch(() => '')
					return mapCurseforgeModToProject(mod, body)
				}
			} catch {}
		}
		throw err
	}
}

export async function get_project_many(ids, cacheBehaviour) {
	const cfIds = ids.filter(isCurseforgeId)
	const mrIds = ids.filter((id) => !isCurseforgeId(id))

	const cfPromises = cfIds.map((id) => get_project(id, cacheBehaviour))
	const mrPromise =
		mrIds.length > 0
			? invoke('plugin:cache|get_project_many', { ids: mrIds, cacheBehaviour })
			: Promise.resolve([])

	const [cfResults, mrResults] = await Promise.all([Promise.all(cfPromises), mrPromise])
	return [...cfResults, ...(mrResults || [])]
}

export async function get_project_v3(id, cacheBehaviour) {
	if (isCurseforgeId(id)) {
		const modId = extractCurseforgeId(id)
		const mod = await curseforgeClient.getMod(modId)
		const res = mapCurseforgeModToSearchV3Project(mod)
		return {
			...res,
			status: 'approved',
			link_urls: {
				site: mod.links?.websiteUrl ? { url: mod.links.websiteUrl } : null,
				store: null,
				wiki: mod.links?.wikiUrl ? { url: mod.links.wikiUrl } : null,
				source: mod.links?.sourceUrl ? { url: mod.links.sourceUrl } : null,
				issues: mod.links?.issuesUrl ? { url: mod.links.issuesUrl } : null,
				discord: null,
			},
		}
	}
	try {
		return await invoke('plugin:cache|get_project_v3', { id, cacheBehaviour })
	} catch (err) {
		if (typeof id === 'string' && !id.includes(' ')) {
			try {
				const cfRes = await curseforgeClient.searchMods({ slug: id.toLowerCase(), pageSize: 1 })
				if (cfRes?.data?.[0]) {
					const mod = cfRes.data[0]
					const res = mapCurseforgeModToSearchV3Project(mod)
					return {
						...res,
						status: 'approved',
						link_urls: {
							site: mod.links?.websiteUrl ? { url: mod.links.websiteUrl } : null,
							store: null,
							wiki: mod.links?.wikiUrl ? { url: mod.links.wikiUrl } : null,
							source: mod.links?.sourceUrl ? { url: mod.links.sourceUrl } : null,
							issues: mod.links?.issuesUrl ? { url: mod.links.issuesUrl } : null,
							discord: null,
						},
					}
				}
			} catch {}
		}
		throw err
	}
}

export async function get_project_v3_many(ids, cacheBehaviour) {
	const cfIds = ids.filter(isCurseforgeId)
	const mrIds = ids.filter((id) => !isCurseforgeId(id))

	const cfPromises = cfIds.map((id) => get_project_v3(id, cacheBehaviour))
	const mrPromise =
		mrIds.length > 0
			? invoke('plugin:cache|get_project_v3_many', { ids: mrIds, cacheBehaviour })
			: Promise.resolve([])

	const [cfResults, mrResults] = await Promise.all([Promise.all(cfPromises), mrPromise])
	return [...cfResults, ...(mrResults || [])]
}

export async function get_project_versions(id, cacheBehaviour, options) {
	if (isCurseforgeId(id)) {
		const modId = extractCurseforgeId(id)
		const query = { pageSize: 100 }
		if (options?.gameVersion) query.gameVersion = options.gameVersion
		if (options?.modLoaderType) query.modLoaderType = options.modLoaderType
		const res = await curseforgeClient.getModFiles(modId, query)
		return (res?.data || []).map((file) => mapCurseforgeFileToVersion(file, modId))
	}
	return await invoke('plugin:cache|get_project_versions', { id, projectId: id, cacheBehaviour })
}

export async function get_version(id, cacheBehaviour) {
	if (isCurseforgeId(id)) {
		const fileId = extractCurseforgeId(id)
		const batch = await curseforgeClient.getFilesBatch([fileId]).catch(() => [])
		if (batch && batch.length > 0) {
			return mapCurseforgeFileToVersion(batch[0])
		}
	}
	return await invoke('plugin:cache|get_version', { id, cacheBehaviour })
}

export async function get_version_many(ids, cacheBehaviour) {
	const cfIds = ids.filter(isCurseforgeId)
	const mrIds = ids.filter((id) => !isCurseforgeId(id))

	let cfVersions = []
	if (cfIds.length > 0) {
		const numericIds = cfIds.map(extractCurseforgeId)
		const files = await curseforgeClient.getFilesBatch(numericIds).catch(() => [])
		cfVersions = files.map((f) => mapCurseforgeFileToVersion(f))
	}

	const mrPromise =
		mrIds.length > 0
			? invoke('plugin:cache|get_version_many', { ids: mrIds, cacheBehaviour })
			: Promise.resolve([])
	const mrVersions = await mrPromise
	return [...cfVersions, ...(mrVersions || [])]
}

export async function get_user(id, cacheBehaviour) {
	if (typeof id === 'string' && (id.startsWith('cf:') || id === 'curseforge')) {
		return {
			id: 'curseforge',
			username: 'CurseForge',
			name: 'CurseForge Author',
			avatar_url: null,
			bio: '',
			created: new Date().toISOString(),
			role: 'curseforge',
		}
	}
	return await invoke('plugin:cache|get_user', { id, cacheBehaviour })
}

export async function get_user_many(ids, cacheBehaviour) {
	const cfIds = ids.filter((id) => typeof id === 'string' && (id.startsWith('cf:') || id === 'curseforge'))
	const mrIds = ids.filter((id) => typeof id !== 'string' || (!id.startsWith('cf:') && id !== 'curseforge'))
	const cfUsers = cfIds.map(() => ({
		id: 'curseforge',
		username: 'CurseForge',
		name: 'CurseForge Author',
		avatar_url: null,
		bio: '',
		created: new Date().toISOString(),
		role: 'curseforge',
	}))
	const mrPromise =
		mrIds.length > 0
			? invoke('plugin:cache|get_user_many', { ids: mrIds, cacheBehaviour })
			: Promise.resolve([])
	const mrUsers = await mrPromise
	return [...cfUsers, ...(mrUsers || [])]
}

export async function get_team(id, cacheBehaviour) {
	if (typeof id === 'string' && (id.startsWith('cf:') || id.startsWith('cf-team-'))) {
		const modId = id.replace(/^(cf:|cf-team-)/, '')
		try {
			const mod = await curseforgeClient.getMod(parseInt(modId, 10))
			const authors = mod?.authors || []
			if (authors.length > 0) {
				return authors.map((author, index) => ({
					id: `cf-user-${author.id}`,
					role: index === 0 ? 'Owner' : 'Author',
					is_owner: index === 0,
					accepted: true,
					user: {
						id: `cf-user-${author.id}`,
						username: author.name,
						name: author.name,
						avatar_url: null,
						url: author.url || `https://www.curseforge.com/members/${encodeURIComponent(author.name)}`,
					},
				}))
			}
		} catch {
			// fallback
		}
		return [
			{
				id: 'cf-author',
				role: 'Owner',
				is_owner: true,
				accepted: true,
				user: {
					id: 'cf-author',
					username: 'CurseForge Author',
					name: 'CurseForge Author',
					avatar_url: null,
				},
			},
		]
	}
	return await invoke('plugin:cache|get_team', { id, cacheBehaviour })
}

export async function get_team_many(ids, cacheBehaviour) {
	const mrIds = ids.filter(
		(id) => typeof id !== 'string' || (!id.startsWith('cf:') && !id.startsWith('cf-team-')),
	)
	if (mrIds.length === 0) return []
	return await invoke('plugin:cache|get_team_many', { ids: mrIds, cacheBehaviour })
}

export async function get_organization(id, cacheBehaviour) {
	return await invoke('plugin:cache|get_organization', { id, cacheBehaviour })
}

export async function get_organization_many(ids, cacheBehaviour) {
	return await invoke('plugin:cache|get_organization_many', { ids, cacheBehaviour })
}

export async function get_search_results(id, cacheBehaviour) {
	return await invoke('plugin:cache|get_search_results', { id, cacheBehaviour })
}

export async function get_search_results_many(ids, cacheBehaviour) {
	return await invoke('plugin:cache|get_search_results_many', { ids, cacheBehaviour })
}

export async function get_search_results_v3(id, cacheBehaviour) {
	return await invoke('plugin:cache|get_search_results_v3', { id, cacheBehaviour })
}

export async function get_search_results_v3_many(ids, cacheBehaviour) {
	return await invoke('plugin:cache|get_search_results_v3_many', { ids, cacheBehaviour })
}

export async function purge_cache_types(cacheTypes) {
	return await invoke('plugin:cache|purge_cache_types', { cacheTypes })
}
