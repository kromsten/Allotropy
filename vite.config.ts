import { mdsvex } from 'mdsvex';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			// === Svelte compiler options ===
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true,
			},

			// === SvelteKit configuration ===
			adapter: adapter({
				pages: 'build',
				assets: 'build',
				fallback: 'fallback.html',
			}),
			
			preprocess: [
				vitePreprocess(),
				mdsvex({
					extensions: ['.svx', '.md']
				})
			],

			extensions: ['.svelte', '.svx', '.md'],

			alias: {
				"$config": "./config",
				"$config/*": "./config/*",
				"$types": "./src/lib/types.ts"
			},

		})
	],

	// === Vitest configuration (unchanged) ===
	test: {
		testTimeout: 60000,
		expect: { requireAssertions: true },
		projects: [
			{
				extends: './vite.config.ts',
				test: {
					name: 'client',
					testTimeout: 60000,
					browser: {
						enabled: true,
						provider: playwright(),
						instances: [{ browser: 'chromium', headless: true }]
					},
					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					exclude: ['src/lib/server/**']
				}
			},
			{
				extends: './vite.config.ts',
				test: {
					name: 'server',
					testTimeout: 60000,
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}', 'tests/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});