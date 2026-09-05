import type { Labrinth } from '../labrinth/types'
import type { CurseforgeFile, CurseforgeMod } from './types'

export function isCurseforgeId(id: string): boolean {
	return typeof id === 'string' && id.startsWith('cf:')
}

export function extractCurseforgeId(id: string): number {
	if (isCurseforgeId(id)) {
		return parseInt(id.slice(3), 10)
	}
	return parseInt(id, 10)
}

export function mapCurseforgeModToSearchV3Project(
	mod: CurseforgeMod,
): Labrinth.Search.v3.ResultSearchProject {
	const cfId = `cf:${mod.id}`

	let projectType: 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack' = 'mod'
	if (mod.classId === 4471) projectType = 'modpack'
	else if (mod.classId === 12) projectType = 'resourcepack'
	else if (mod.classId === 6552) projectType = 'shader'
	else if (mod.classId === 6945 || mod.categories?.some((c) => c.id === 5193 || c.slug === 'data-packs')) projectType = 'datapack'

	const categories = mod.categories.map((c) => c.slug)

	const gameVersions = Array.from(
		new Set(
			mod.latestFilesIndexes
				?.map((idx) => idx.gameVersion)
				.filter((v) => v && /^\d+\.\d+(\.\d+)?$/.test(v)) ?? [],
		),
	)

	const loaders = Array.from(
		new Set(
			mod.latestFilesIndexes
				?.map((idx) => {
					switch (idx.modLoader) {
						case 1:
							return 'forge'
						case 4:
							return 'fabric'
						case 5:
							return 'quilt'
						case 6:
							return 'neoforge'
						default:
							return null
					}
				})
				.filter(Boolean) as string[],
		),
	)

	if (projectType === 'datapack' && loaders.length === 0) {
		loaders.push('datapack')
	}

	const iconUrl = mod.logo?.thumbnailUrl || mod.logo?.url || null
	const gallery = mod.screenshots?.map((s) => s.url) ?? []

	return {
		project_id: cfId,
		project_types: [projectType],
		all_project_types: [projectType],
		slug: mod.slug,
		author: mod.authors?.[0]?.name ?? 'CurseForge Creator',
		author_url: mod.authors?.[0]?.url || (mod.authors?.[0]?.name ? `https://www.curseforge.com/members/${encodeURIComponent(mod.authors[0].name)}` : 'https://www.curseforge.com'),
		author_id: null,
		organization: null,
		organization_id: null,
		name: mod.name,
		summary: mod.summary || '',
		categories,
		display_categories: categories,
		downloads: mod.downloadCount ?? 0,
		follows: mod.thumbsUpCount ?? 0,
		icon_url: iconUrl,
		date_created: mod.dateCreated,
		date_modified: mod.dateModified,
		license: 'custom',
		gallery,
		featured_gallery: gallery[0] ?? null,
		color: null,
		loaders,
		disclosure_types: [],
		project_loader_fields: {
			loaders,
			game_versions: gameVersions,
			environment: ['required' as Labrinth.Projects.v3.Environment],
		},
	}
}

export function mapCurseforgeModToProject(
	mod: CurseforgeMod,
	bodyHtml?: string,
): Labrinth.Projects.v2.Project {
	const cfId = `cf:${mod.id}`

	let projectType: 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack' = 'mod'
	if (mod.classId === 4471) projectType = 'modpack'
	else if (mod.classId === 12) projectType = 'resourcepack'
	else if (mod.classId === 6552) projectType = 'shader'
	else if (mod.classId === 6945 || mod.categories?.some((c) => c.id === 5193 || c.slug === 'data-packs')) projectType = 'datapack'

	const categories = mod.categories.map((c) => c.slug)

	const gameVersions = Array.from(
		new Set(
			mod.latestFilesIndexes
				?.map((idx) => idx.gameVersion)
				.filter((v) => v && /^\d+\.\d+(\.\d+)?$/.test(v)) ?? [],
		),
	)

	const loaders = Array.from(
		new Set(
			mod.latestFilesIndexes
				?.map((idx) => {
					switch (idx.modLoader) {
						case 1:
							return 'forge'
						case 4:
							return 'fabric'
						case 5:
							return 'quilt'
						case 6:
							return 'neoforge'
						default:
							return null
					}
				})
				.filter(Boolean) as string[],
		),
	)

	if (projectType === 'datapack' && loaders.length === 0) {
		loaders.push('datapack')
	}

	const fileIds = new Set<string>()
	for (const f of mod.latestFiles ?? []) {
		fileIds.add(`cf:${f.id}`)
	}
	for (const idx of mod.latestFilesIndexes ?? []) {
		if (idx.fileId) {
			fileIds.add(`cf:${idx.fileId}`)
		}
	}
	const versions = Array.from(fileIds)

	return {
		id: cfId,
		slug: mod.slug,
		project_type: projectType,
		actualProjectType: projectType,
		thread_id: '',
		monetization_status: 'demonetized',
		organization: null,
		team: `cf-team-${mod.id}`,
		title: mod.name,
		description: mod.summary || '',
		body: bodyHtml || mod.summary || '',
		published: mod.dateCreated,
		updated: mod.dateModified,
		approved: mod.dateCreated,
		status: 'approved',
		license: {
			id: 'custom',
			name: 'CurseForge',
			url: mod.links?.websiteUrl || undefined,
		},
		client_side: 'required',
		server_side: 'optional',
		downloads: mod.downloadCount ?? 0,
		followers: mod.thumbsUpCount ?? 0,
		categories,
		additional_categories: [],
		game_versions: gameVersions,
		loaders,
		versions,
		icon_url: mod.logo?.thumbnailUrl || mod.logo?.url || undefined,
		raw_icon_url: mod.logo?.url || undefined,
		issues_url: mod.links?.issuesUrl || undefined,
		source_url: mod.links?.sourceUrl || undefined,
		wiki_url: mod.links?.wikiUrl || undefined,
		discord_url: undefined,
		donation_urls: [],
		gallery:
			mod.screenshots?.map((s, i) => ({
				url: s.url,
				featured: i === 0,
				title: s.title || '',
				description: s.description || undefined,
				created: mod.dateCreated,
				ordering: i,
			})) ?? [],
	}
}

export function mapCurseforgeFileToVersion(
	file: CurseforgeFile,
	modId?: number,
): Labrinth.Versions.v2.Version {
	const cfId = `cf:${file.id}`
	const projectCfId = modId ? `cf:${modId}` : `cf:${file.modId}`

	let versionType: 'release' | 'beta' | 'alpha' = 'release'
	if (file.releaseType === 2) versionType = 'beta'
	else if (file.releaseType === 3) versionType = 'alpha'

	const sha1Hash = file.hashes?.find((h) => h.algo === 1)?.value ?? ''

	const gameVersions = (file.gameVersions || []).filter((v) => /^\d+\.\d+(\.\d+)?$/.test(v))
	const loaders = (file.gameVersions || [])
		.map((v) => v.toLowerCase())
		.filter((v) => [
			'forge',
			'fabric',
			'quilt',
			'neoforge',
			'iris',
			'optifine',
			'canvas',
			'vanilla',
			'minecraft',
			'datapack',
		].includes(v))

	if (loaders.length === 0 && file.modules?.some((m) => m.name === 'data')) {
		loaders.push('datapack')
	}

	const primaryFile: Labrinth.Versions.v2.VersionFile = {
		hashes: {
			sha1: sha1Hash,
			sha512: '',
		},
		url: file.downloadUrl || '',
		filename: file.fileName,
		primary: true,
		size: file.fileLength,
	}

	const dependencies: Labrinth.Versions.v2.Dependency[] = (file.dependencies || []).map((dep) => {
		let depType: 'required' | 'optional' | 'incompatible' | 'embedded' = 'optional'
		if (dep.relationType === 3) depType = 'required'
		else if (dep.relationType === 5) depType = 'incompatible'
		else if (dep.relationType === 1 || dep.relationType === 6) depType = 'embedded'

		return {
			version_id: '',
			project_id: `cf:${dep.modId}`,
			dependency_type: depType,
		}
	})

	return {
		id: cfId,
		project_id: projectCfId,
		author_id: 'curseforge',
		featured: false,
		name: file.displayName || file.fileName,
		version_number: file.displayName || file.fileName,
		changelog: '',
		date_published: file.fileDate,
		downloads: file.downloadCount ?? 0,
		version_type: versionType,
		files: [primaryFile],
		dependencies,
		game_versions: gameVersions,
		loaders,
		status: 'listed',
	}
}


