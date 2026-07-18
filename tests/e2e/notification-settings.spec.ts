import { test, expect } from "./fixtures";

test.describe("Notification Settings", () => {
  test("notification bell button is visible in sidebar", async ({
    mockedPage: page,
  }) => {
    const bellButton = page.locator('[title^="Notifications"]');
    await expect(bellButton).toBeVisible();
  });

  test("notification center opens when clicking bell", async ({
    mockedPage: page,
  }) => {
    await page.locator('[title^="Notifications"]').click();
    // The notification center panel should slide in (fixed positioning)
    await expect(page.locator(".fixed.inset-0")).toBeVisible();
  });

  test("integrations button is visible in sidebar", async ({
    mockedPage: page,
  }) => {
    const integrationsButton = page.locator('[title="Integrations"]');
    await expect(integrationsButton).toBeVisible();
  });

  test("integrations modal opens when clicking integrations button", async ({
    mockedPage: page,
  }) => {
    await page.locator('[title="Integrations"]').click();
    // Wait for the modal overlay to appear
    await page.waitForSelector(".fixed.inset-0.z-50", { timeout: 5000 });
    // Should see the integrations page content inside the modal
    await expect(page.locator("text=Native Integrations").first()).toBeVisible();
  });

  test("MCP Servers section exists in integrations page", async ({
    mockedPage: page,
  }) => {
    await page.locator('[title="Integrations"]').click();
    await page.waitForSelector(".fixed.inset-0.z-50", { timeout: 5000 });
    // MCP Servers is a collapsible section
    await expect(page.locator("text=MCP Servers").first()).toBeVisible();
  });

  test("integrations page shows available integration types", async ({
    mockedPage: page,
  }) => {
    await page.locator('[title="Integrations"]').click();
    await page.waitForSelector(".fixed.inset-0.z-50", { timeout: 5000 });
    // Should show at least one integration option
    await expect(page.locator("text=Slack").or(page.locator("text=GitHub")).first()).toBeVisible();
  });
});
