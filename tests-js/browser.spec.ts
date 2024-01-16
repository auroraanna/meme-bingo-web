const { test, expect } = require('@playwright/test');

test('has title', async ({ page }) => {
	await page.goto('/');

	// Expect a title "to contain" a substring.
	await expect(page).toHaveTitle("Meme bingo");
});

// This depends on the inline style having a nonce and the CSP having the same nonce.
test('heading has ::before that is an emoji', async ({ page }) => {
	await page.goto('/');

	let before: string = await page.evaluate('window.getComputedStyle(document.querySelector("h1"), ":before").content');
	console.log(before);
	expect(/\p{Extended_Pictographic}/u.test(before)).toBeTruthy();
});

test('bingo size limit enforced', async ({ page }) => {
	await page.goto('/edit?size=16').then((response) => {
		expect(response.status() == 400).toBeTruthy();
	});
});

test('submit form to create new bingo', async ({ page }) => {
	await page.goto('/new');

	await page.locator('input, input[type="number"]').pressSequentially('5');
	await page.locator('button, input[type="submit"]').press('Enter');
	let path: string = await page.evaluate(
		'window.location.pathname + window.location.search'
	);
	expect(path.startsWith('/edit?size=')).toBeTruthy();
});

test('submit form to update bingo address', async ({ page }) => {
	await page.goto('/edit?size=5');

	await page.locator('button, input[type="submit"]').press('Enter');
	let path: string = await page.evaluate(
		'window.location.search'
	);
	expect(!path.includes('size=')).toBeTruthy();
});
