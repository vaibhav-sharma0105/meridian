import { test as base, Page } from "@playwright/test";
import { buildTauriMockScript } from "./setup/tauri-mock";

type MeridianFixtures = {
  mockedPage: Page;
};

export const test = base.extend<MeridianFixtures>({
  mockedPage: async ({ page }, use) => {
    // Inject Tauri mock before any page script runs
    await page.addInitScript(buildTauriMockScript());
    await page.goto("/");
    // Wait for AppShell — onboarding is bypassed by the mock returning onboarding_complete=true
    // The sidebar always renders "Meridian" as the brand label
    await page.waitForSelector('text=Meridian', { timeout: 15000 });
    await use(page);
  },
});

/**
 * Mounts the app with per-test mock overrides.
 *
 * Use this instead of the `mockedPage` fixture when a test needs a different
 * backend response — overrides are baked into the init script at build time,
 * so they cannot be changed after `mockedPage` has already navigated.
 *
 *   test("...", async ({ page }) => {
 *     await mountWithMocks(page, { get_role_drift_alert: { ... } });
 *   });
 */
export async function mountWithMocks(
  page: Page,
  overrides: Record<string, unknown> = {}
): Promise<Page> {
  await page.addInitScript(buildTauriMockScript(overrides));
  await page.goto("/");
  await page.waitForSelector("text=Meridian", { timeout: 15000 });
  return page;
}

export { expect } from "@playwright/test";
