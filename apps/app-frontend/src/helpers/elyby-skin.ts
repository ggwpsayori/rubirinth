import { arrayBufferToBase64 } from '@modrinth/utils'
import { invoke } from '@tauri-apps/api/core'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { ref } from 'vue'

import { normalize_skin_texture } from '@/helpers/skins'

// Reactive cache: lowercase username -> headUrl (blob: or data:)
export const elybyHeadCache = ref<Map<string, string>>(new Map())

const inFlight = new Map<string, Promise<string | null>>()

async function safeFetch(url: string, init?: RequestInit): Promise<Response> {
	try {
		if (url.startsWith('/')) {
			return await fetch(url, init)
		}
		return await tauriFetch(url, init)
	} catch {
		return await fetch(url, init)
	}
}

export interface ElyBySkinInfo {
	url: string
	variant: 'classic' | 'slim'
}

/**
 * Queries http://skinsystem.ely.by/textures/<username> to get full skin info (URL and model)
 */
export async function getElyBySkinFull(username: string): Promise<ElyBySkinInfo | null> {
	const trimmed = username.trim()
	if (!trimmed) return null

	const endpoints = [
		'/elyby-api/textures/' + encodeURIComponent(trimmed),
		'https://skinsystem.ely.by/textures/' + encodeURIComponent(trimmed),
		'http://skinsystem.ely.by/textures/' + encodeURIComponent(trimmed),
	]

	for (const endpoint of endpoints) {
		try {
			const res = await safeFetch(endpoint)
			if (res.status === 200) {
				const data = (await res.json()) as {
					SKIN?: { url?: string; metadata?: { model?: string } }
				}
				if (data?.SKIN?.url) {
					const variant = data.SKIN.metadata?.model === 'slim' ? 'slim' : 'classic'
					return { url: data.SKIN.url, variant }
				}
			} else if (res.status === 204) {
				return null
			}
		} catch (e) {
			console.warn('[Ely.by] Error querying ' + endpoint + ':', e)
		}
	}
	return null
}

/**
 * Queries http://skinsystem.ely.by/textures/<username> to get the player's skin URL
 */
export async function getElyBySkinTextureUrl(username: string): Promise<string | null> {
	const info = await getElyBySkinFull(username)
	return info?.url ?? null
}

/**
 * High-level helper to upload a skin, equip it on Ely.by, and invalidate the avatar cache.
 * Uses native seamless Ely.by authentication bridge.
 */
export async function uploadAndWearElyBySkin(
	skinBlob: Blob,
	username: string,
): Promise<{ id: number }> {
	const arrayBuffer = await skinBlob.arrayBuffer()
	const skinBase64 = arrayBufferToBase64(new Uint8Array(arrayBuffer))
	const skinId = await invoke<number>('plugin:auth|elyby_upload_and_wear_skin', {
		skinBase64,
	})
	await invalidateElyByHead(username)
	window.dispatchEvent(new CustomEvent('rubirinth-accounts-updated'))
	return { id: skinId }
}

/**
 * Uploads a skin file directly (legacy fallback)
 */
export async function uploadElyBySkin(skinBlob: Blob): Promise<{ id: number }> {
	const arrayBuffer = await skinBlob.arrayBuffer()
	const skinBase64 = arrayBufferToBase64(new Uint8Array(arrayBuffer))
	const skinId = await invoke<number>('plugin:auth|elyby_upload_and_wear_skin', {
		skinBase64,
	})
	return { id: skinId }
}

/**
 * Equips a skin on Ely.by (legacy fallback)
 */
export async function wearElyBySkin(_skinId: number): Promise<void> {
	// Handled natively by uploadAndWearElyBySkin
}

/**
 * Invalidates the cached head for a given Ely.by username and reloads it.
 */
export async function invalidateElyByHead(username: string): Promise<string | null> {
	const key = username.trim().toLowerCase()
	if (!key) return null

	const existing = elybyHeadCache.value.get(key)
	if (existing && existing.startsWith('blob:')) {
		try {
			URL.revokeObjectURL(existing)
		} catch {}
	}

	const newMap = new Map(elybyHeadCache.value)
	newMap.delete(key)
	elybyHeadCache.value = newMap
	inFlight.delete(key)

	return await loadElyByHead(username)
}

/**
 * Renders an 8x8 Minecraft head + 8x8 outer hat layer from a skin image to a square head blob.
 * Face: X: 8..15, Y: 8..15
 * Hat/Helmet: X: 40..47, Y: 8..15
 */
export async function renderHeadFromImage(img: HTMLImageElement, size = 128): Promise<Blob> {
	const canvas = document.createElement('canvas')
	canvas.width = size
	canvas.height = size
	const ctx = canvas.getContext('2d')
	if (!ctx) throw new Error('No 2d context')

	ctx.imageSmoothingEnabled = false

	// Base face is at (8, 8, 8, 8) in a 64x64 or 64x32 skin
	ctx.drawImage(img, 8, 8, 8, 8, 0, 0, size, size)

	// Hat / outer layer is at (40, 8, 8, 8)
	// Check if hat contains non-transparent pixels
	const tempCanvas = document.createElement('canvas')
	tempCanvas.width = 8
	tempCanvas.height = 8
	const tempCtx = tempCanvas.getContext('2d')
	if (tempCtx) {
		tempCtx.drawImage(img, 40, 8, 8, 8, 0, 0, 8, 8)
		const imgData = tempCtx.getImageData(0, 0, 8, 8).data
		let hasHat = false
		for (let i = 3; i < imgData.length; i += 4) {
			if (imgData[i] > 0) {
				hasHat = true
				break
			}
		}
		if (hasHat) {
			ctx.drawImage(img, 40, 8, 8, 8, 0, 0, size, size)
		}
	}

	return new Promise((resolve, reject) => {
		canvas.toBlob(
			(blob) => (blob ? resolve(blob) : reject(new Error('Canvas toBlob failed'))),
			'image/png',
		)
	})
}

/**
 * Loads and renders the Ely.by head for a given username.
 * Returns the head object URL or null if not found.
 */
export async function loadElyByHead(username: string): Promise<string | null> {
	const key = username.trim().toLowerCase()
	if (!key) return null

	if (elybyHeadCache.value.has(key)) {
		return elybyHeadCache.value.get(key)!
	}

	if (inFlight.has(key)) {
		return inFlight.get(key)!
	}

	const promise = (async () => {
		try {
			const skinUrl = await getElyBySkinTextureUrl(username)
			if (!skinUrl) {
				return null
			}

			const img = new Image()
			let loaded = false

			// 1. Direct proxy load (Vite dev)
			try {
				const proxyUrl = skinUrl.replace(/^https?:\/\/ely\.by/, '/elyby-storage')
				await new Promise<void>((resolve, reject) => {
					img.onload = () => resolve()
					img.onerror = reject
					img.src = proxyUrl
				})
				loaded = true
			} catch {
				// Fall back to Rust normalization
			}

			// 2. Rust-based download and normalization (works anywhere)
			if (!loaded) {
				const rawData = await normalize_skin_texture(skinUrl)
				const base64 = arrayBufferToBase64(new Uint8Array(rawData))
				const dataUrl = 'data:image/png;base64,' + base64

				await new Promise<void>((resolve, reject) => {
					img.onload = () => resolve()
					img.onerror = () => reject(new Error('Failed to load base64 skin image'))
					img.src = dataUrl
				})
			}

			// 3. Render 128x128 Minecraft head
			const headBlob = await renderHeadFromImage(img, 128)
			const headUrl = URL.createObjectURL(headBlob)

			const newMap = new Map(elybyHeadCache.value)
			newMap.set(key, headUrl)
			elybyHeadCache.value = newMap

			console.log('[Ely.by] Successfully loaded head for ' + username + ':', headUrl)
			return headUrl
		} catch (error) {
			console.warn('[Ely.by] Error loading head for ' + username + ':', error)
			return null
		} finally {
			inFlight.delete(key)
		}
	})()

	inFlight.set(key, promise)
	return promise
}
