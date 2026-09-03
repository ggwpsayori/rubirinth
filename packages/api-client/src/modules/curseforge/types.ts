export interface CurseforgeAttachment {
	id: number
	modId: number
	title: string
	description: string
	thumbnailUrl: string
	url: string
}

export interface CurseforgeAuthor {
	id: number
	name: string
	url: string
}

export interface CurseforgeLogo {
	id: number
	modId: number
	title: string
	description: string
	thumbnailUrl: string
	url: string
}

export interface CurseforgeFileHash {
	value: string
	algo: number // 1 = Sha1, 2 = Md5
}

export interface CurseforgeFileDependency {
	modId: number
	relationType: number // 1 = EmbeddedLibrary, 2 = OptionalDependency, 3 = RequiredDependency, 4 = Tool, 5 = Incompatible, 6 = Include
}

export interface CurseforgeModule {
	name: string
	fingerprint: number
}

export interface CurseforgeFile {
	id: number
	gameId: number
	modId: number
	isAvailable: boolean
	displayName: string
	fileName: string
	releaseType: number // 1 = Release, 2 = Beta, 3 = Alpha
	fileStatus: number
	hashes: CurseforgeFileHash[]
	fileDate: string
	fileLength: number
	downloadCount: number
	downloadUrl: string | null
	gameVersions: string[]
	sortableGameVersions?: {
		gameVersionName: string
		gameVersionPadded: string
		gameVersion: string
		gameVersionReleaseDate: string
		gameVersionTypeId: number | null
	}[]
	dependencies: CurseforgeFileDependency[]
	exposeAsAlternative?: boolean
	parentProjectFileId?: number
	alternateFileId?: number
	isServerPack?: boolean
	serverPackFileId?: number
	fileFingerprint?: number
	modules?: CurseforgeModule[]
}

export interface CurseforgeFileIndex {
	gameVersion: string
	fileId: number
	filename: string
	releaseType: number
	gameVersionTypeId: number | null
	modLoader: number | null // 1 = Forge, 4 = Fabric, 5 = Quilt, 6 = NeoForge
}

export interface CurseforgeCategory {
	id: number
	gameId: number
	name: string
	slug: string
	url: string
	iconUrl: string
	dateModified: string
	isClass?: boolean
	classId: number
	parentCategoryId: number
	displayIndex: number
}

export interface CurseforgeMod {
	id: number
	gameId: number
	name: string
	slug: string
	links: {
		websiteUrl?: string
		wikiUrl?: string
		issuesUrl?: string
		sourceUrl?: string
	}
	summary: string
	status: number
	downloadCount: number
	isFeatured: boolean
	primaryCategoryId: number
	categories: CurseforgeCategory[]
	classId: number | null
	authors: CurseforgeAuthor[]
	logo: CurseforgeLogo | null
	screenshots: CurseforgeAttachment[]
	mainFileId: number
	latestFiles: CurseforgeFile[]
	latestFilesIndexes: CurseforgeFileIndex[]
	dateCreated: string
	dateModified: string
	dateReleased: string
	allowModDistribution: boolean | null
	gamePopularityRank: number
	isAvailable: boolean
	thumbsUpCount: number
}

export interface CurseforgePagination {
	index: number
	pageSize: number
	resultCount: number
	totalCount: number
}

export interface CurseforgeSearchResponse {
	data: CurseforgeMod[]
	pagination: CurseforgePagination
}

export interface CurseforgeModResponse {
	data: CurseforgeMod
}

export interface CurseforgeDescriptionResponse {
	data: string
}

export interface CurseforgeFilesResponse {
	data: CurseforgeFile[]
	pagination: CurseforgePagination
}

export interface CurseforgeFileResponse {
	data: CurseforgeFile
}

export interface CurseforgeManifestFile {
	projectID: number
	fileID: number
	required: boolean
}

export interface CurseforgeManifestMinecraft {
	version: string
	modLoaders: {
		id: string
		primary: boolean
	}[]
}

export interface CurseforgeManifest {
	minecraft: CurseforgeManifestMinecraft
	manifestType: string
	manifestVersion: number
	name: string
	version: string
	author?: string
	description?: string
	files: CurseforgeManifestFile[]
	overrides: string
}
