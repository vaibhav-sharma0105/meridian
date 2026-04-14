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

export { expect } from "@playwright/test";
