import type {
	CurseforgeCategory,
	CurseforgeDescriptionResponse,
	CurseforgeFile,
	CurseforgeFileResponse,
	CurseforgeFilesResponse,
	CurseforgeMod,
	CurseforgeModResponse,
	CurseforgeSearchResponse,
} from './types'

export const CURSEFORGE_DEFAULT_API_KEY = '$2a$10$X70Sj4q5B9i03tElmqpvfezFNT/AQGwIa0yy5qG8Q7yhowAnLmxY.'
export const CURSEFORGE_API_BASE = 'https://api.curseforge.com/v1'
export const MINECRAFT_GAME_ID = 432

export const CURSEFORGE_CLASS_IDS = {
	MODS: 6,
	MODPACKS: 4471,
	RESOURCE_PACKS: 12,
	SHADERS: 6552,
	WORLDS: 17,
} as const

export const CURSEFORGE_LOADER_TYPES = {
	ANY: 0,
	FORGE: 1,
	CAULDRON: 2,
	LITE_LOADER: 3,
	FABRIC: 4,
	QUILT: 5,
	NEOFORGE: 6,
} as const

export interface CurseforgeSearchParams {
	gameId?: number
	classId?: number
	categoryId?: number
	gameVersion?: string
	searchFilter?: string
	sortField?: number // 1: Featured, 2: Popularity, 3: LastUpdated, 4: Name, 5: Author, 6: TotalDownloads, 7: Category, 8: GameVersion
	sortOrder?: 'asc' | 'desc'
	modLoaderType?: number // 1: Forge, 4: Fabric, 5: Quilt, 6: NeoForge
	gameVersionTypeId?: number
	authorId?: number
	slug?: string
	index?: number
	pageSize?: number
}

export type CurseforgeRequester = <T>(path: string, options?: RequestInit) => Promise<T>

export class CurseforgeClient {
	private apiKey: string
	private baseUrl: string
	private fetchFn: typeof fetch
	private customRequester?: CurseforgeRequester

	constructor(options?: { apiKey?: string; baseUrl?: string; fetch?: typeof fetch; requester?: CurseforgeRequester }) {
		this.apiKey = options?.apiKey || CURSEFORGE_DEFAULT_API_KEY
		this.baseUrl = options?.baseUrl || CURSEFORGE_API_BASE
		this.fetchFn = options?.fetch || globalThis.fetch.bind(globalThis)
		this.customRequester = options?.requester
	}

	public setRequester(requester: CurseforgeRequester) {
		this.customRequester = requester
	}

	private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
		if (this.customRequester) {
			return await this.customRequester<T>(path, options)
		}

		const url = `${this.baseUrl}${path.startsWith('/') ? path : `/${path}`}`
		const headers = new Headers(options.headers || {})
		headers.set('x-api-key', this.apiKey)
		headers.set('Accept', 'application/json')
		if (!headers.has('User-Agent')) {
			headers.set('User-Agent', 'Rubirinth-App')
		}

		const response = await this.fetchFn(url, {
			...options,
			headers,
		})

		if (!response.ok) {
			const errorText = await response.text().catch(() => '')
			throw new Error(`CurseForge API Error (${response.status} ${response.statusText}): ${errorText}`)
		}

		return (await response.json()) as T
	}

	async searchMods(params: CurseforgeSearchParams): Promise<CurseforgeSearchResponse> {
		const query = new URLSearchParams()
		query.set('gameId', String(params.gameId ?? MINECRAFT_GAME_ID))

		if (params.classId !== undefined) query.set('classId', String(params.classId))
		if (params.categoryId !== undefined) query.set('categoryId', String(params.categoryId))
		if (params.gameVersion) query.set('gameVersion', params.gameVersion)
		if (params.searchFilter) query.set('searchFilter', params.searchFilter)
		if (params.sortField !== undefined) query.set('sortField', String(params.sortField))
		if (params.sortOrder) query.set('sortOrder', params.sortOrder)
		if (params.modLoaderType !== undefined && params.modLoaderType !== 0) {
			query.set('modLoaderType', String(params.modLoaderType))
		}
		if (params.slug) query.set('slug', params.slug)
		if (params.index !== undefined) query.set('index', String(params.index))
		if (params.pageSize !== undefined) query.set('pageSize', String(params.pageSize))

		return await this.request<CurseforgeSearchResponse>(`/mods/search?${query.toString()}`)
	}

	async getMod(modId: number | string): Promise<CurseforgeMod> {
		const res = await this.request<CurseforgeModResponse>(`/mods/${modId}`)
		return res.data
	}

	async getModDescription(modId: number | string): Promise<string> {
		const res = await this.request<CurseforgeDescriptionResponse>(`/mods/${modId}/description`)
		return res.data
	}

	async getModFiles(
		modId: number | string,
		params?: {
			gameVersion?: string
			modLoaderType?: number
			gameVersionTypeId?: number
			index?: number
			pageSize?: number
		},
	): Promise<CurseforgeFilesResponse> {
		const query = new URLSearchParams()
		if (params?.gameVersion) query.set('gameVersion', params.gameVersion)
		if (params?.modLoaderType !== undefined && params.modLoaderType !== 0) {
			query.set('modLoaderType', String(params.modLoaderType))
		}
		if (params?.index !== undefined) query.set('index', String(params.index))
		if (params?.pageSize !== undefined) query.set('pageSize', String(params.pageSize))

		const queryString = query.toString()
		const endpoint = `/mods/${modId}/files${queryString ? `?${queryString}` : ''}`
		return await this.request<CurseforgeFilesResponse>(endpoint)
	}

	async getFile(modId: number | string, fileId: number | string): Promise<CurseforgeFile> {
		const res = await this.request<CurseforgeFileResponse>(`/mods/${modId}/files/${fileId}`)
		return res.data
	}

	async getFilesBatch(fileIds: number[]): Promise<CurseforgeFile[]> {
		if (!fileIds || fileIds.length === 0) return []
		const results: CurseforgeFile[] = []
		for (let i = 0; i < fileIds.length; i += 100) {
			const chunk = fileIds.slice(i, i + 100)
			const res = await this.request<{ data: CurseforgeFile[] }>('/mods/files', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ fileIds: chunk }),
			})
			if (res?.data) {
				results.push(...res.data)
			}
		}
		return results
	}

	async getCategories(classId?: number): Promise<CurseforgeCategory[]> {
		const query = new URLSearchParams()
		query.set('gameId', String(MINECRAFT_GAME_ID))
		if (classId !== undefined) query.set('classId', String(classId))
		const res = await this.request<{ data: CurseforgeCategory[] }>(`/categories?${query.toString()}`)
		return res.data
	}
}

export const curseforgeClient = new CurseforgeClient()
