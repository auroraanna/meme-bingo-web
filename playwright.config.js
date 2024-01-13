// @ts-check
const { defineConfig, devices } = require('@playwright/test');

module.exports = defineConfig({
	testDir: './tests-js',
	timeout: 5000,
	fullyParallel: true,
	forbidOnly: false,
	retries: 0,
	workers: undefined,
	reporter: 'list',
	use: {
		baseURL: `http://localhost:${process.env.MEME_BINGO_PORT}`
	},

	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] },
		}
	]
});

