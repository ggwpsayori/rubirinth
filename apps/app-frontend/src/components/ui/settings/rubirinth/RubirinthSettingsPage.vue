<script setup lang="ts">
defineProps<{
	description: string
	title: string
}>()
</script>

<template>
	<div class="flex min-w-0 flex-col gap-5">
		<header
			class="relative overflow-hidden rounded-2xl border border-solid border-[rgba(70,127,197,0.38)] bg-[linear-gradient(135deg,rgba(70,127,197,0.14),rgba(70,127,197,0.035)_58%,transparent)] p-5"
		>
			<div
				class="pointer-events-none absolute -right-16 -top-20 size-56 rounded-full bg-[rgba(70,127,197,0.12)] blur-3xl"
			/>
			<div class="relative flex items-center gap-4">
				<div
					class="flex size-12 shrink-0 items-center justify-center rounded-2xl border border-solid border-[rgba(70,127,197,0.35)] bg-[rgba(70,127,197,0.16)] text-brand shadow-[0_12px_32px_rgba(0,0,0,0.18)]"
				>
					<svg
						viewBox="0 0 424 419"
						fill="currentColor"
						xmlns="http://www.w3.org/2000/svg"
						class="w-7 h-7"
						aria-hidden="true"
					>
						<path d="M0 0L423.407 75.8519L307.852 170.074H258.519L423.407 418.963L99.4074 123.407L220.296 132.741L222.074 111.407L84.7407 54.5185L56.2963 71.8519L0 15.4074V0Z" />
						<path d="M91.5658 265.778L187.852 337.481H31.3467L91.5658 265.778Z" />
					</svg>
				</div>
				<div class="min-w-0">
					<h1 class="m-0 text-xl font-bold leading-tight text-contrast">
						{{ title }}
					</h1>
					<p class="m-0 mt-1 max-w-2xl text-sm leading-relaxed text-secondary">
						{{ description }}
					</p>
				</div>
				<div v-if="$slots.actions" class="ml-auto shrink-0">
					<slot name="actions" />
				</div>
			</div>
		</header>

		<main>
			<slot />
		</main>

		<footer v-if="$slots.footer" class="px-1 text-sm text-secondary">
			<slot name="footer" />
		</footer>
	</div>
</template>
