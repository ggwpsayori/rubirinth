import { invoke } from '@tauri-apps/api/core'
import type { MinecraftCredential } from '@/models/astralrinth/authentication'

/**
 * Check if the authentication servers are reachable, throwing an exception if
 * not reachable.
 */
export async function check_reachable(): Promise<void> {
	await invoke('plugin:auth|check_reachable')
}

/**
 * Authenticate a user with Hydra / Microsoft.
 */
export async function login(): Promise<MinecraftCredential | null> {
	return await invoke('plugin:auth|login')
}

/**
 * Retrieves the default user ID
 */
export async function get_default_user(): Promise<string | undefined> {
	return await invoke('plugin:auth|get_default_user')
}

/**
 * Updates the default user
 */
export async function set_default_user(user: string): Promise<void> {
	return await invoke('plugin:auth|set_default_user', { user })
}

/**
 * Remove a user account from the database
 */
export async function remove_user(user: string): Promise<void> {
	return await invoke('plugin:auth|remove_user', { user })
}

/**
 * Returns a list of users
 */
export async function users(): Promise<MinecraftCredential[]> {
	return await invoke('plugin:auth|get_users')
}

export async function offline_login(name: string): Promise<MinecraftCredential> {
	return await invoke('plugin:auth|offline_login', { name })
}
