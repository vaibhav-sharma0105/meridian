import { test, expect } from "./fixtures";

test.describe("My Activity Dashboard", () => {
  test("shows My Activity sidebar entry", async ({ mockedPage: page }) => {
    await expect(page.getByTestId("sidebar-activity")).toBeVisible();
  });

  test("clicking My Activity navigates to activity view", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-activity").click();
    await expect(page.getByRole("heading", { name: "My Activity" })).toBeVisible();
  });

  test("displays attention items grouped by severity", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-activity").click();
    // Look for severity section headers (buttons that collapse/expand)
    await expect(page.getByRole("button", { name: /Critical/i })).toBeVisible();
  });

  test("shows attention item reason text", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-activity").click();
    // The reason_text from the mock data
    await expect(page.getByText("Fix the login bug - 5 days overdue")).toBeVisible();
  });

  test("shows source type labels", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-activity").click();
    // Source type is shown as uppercase label - use exact match
    await expect(page.getByText("task", { exact: true })).toBeVisible();
  });
});

test.describe("Integration Browser", () => {
  // Helper to get the Integrations tab (not the sidebar button)
  const getIntegrationsTab = (page: import("@playwright/test").Page) =>
    page.locator('button:has-text("Integrations")').filter({ hasText: /^Integrations$/ }).last();

  test("shows Integrations tab when project is selected", async ({ mockedPage: page }) => {
    // Click on a project in the sidebar
    await page.locator('[data-testid="project-item"]').first().click();
    await expect(getIntegrationsTab(page)).toBeVisible();
  });

  test("clicking Integrations tab shows browser view", async ({ mockedPage: page }) => {
    await page.locator('[data-testid="project-item"]').first().click();
    await getIntegrationsTab(page).click();
    await expect(page.getByPlaceholder("Search integration items...")).toBeVisible();
  });

  test("displays filter dropdowns", async ({ mockedPage: page }) => {
    await page.locator('[data-testid="project-item"]').first().click();
    await getIntegrationsTab(page).click();
    await expect(page.getByRole("combobox").first()).toBeVisible();
  });

  test("shows cached integration items", async ({ mockedPage: page }) => {
    await page.locator('[data-testid="project-item"]').first().click();
    await getIntegrationsTab(page).click();
    // Wait for items to load
    await page.waitForTimeout(500);
    // Should show at least the GitHub issue title from mock data
    await expect(page.getByText("Bug: Login fails on Safari").first()).toBeVisible();
  });

  test("can expand integration item to see details", async ({ mockedPage: page }) => {
    await page.locator('[data-testid="project-item"]').first().click();
    await getIntegrationsTab(page).click();
    await page.waitForTimeout(500);
    // Click on an item row to expand
    await page.getByText("Bug: Login fails on Safari").first().click();
    // Should show the description
    await expect(page.getByText("Users report login issues on Safari browser")).toBeVisible();
  });

  test("search input filters items", async ({ mockedPage: page }) => {
    await page.locator('[data-testid="project-item"]').first().click();
    await getIntegrationsTab(page).click();
    const searchInput = page.getByPlaceholder("Search integration items...");
    await searchInput.fill("login");
    await page.waitForTimeout(500);
    // Should filter to matching items
    await expect(page.getByText("Bug: Login fails on Safari").first()).toBeVisible();
  });
});

test.describe("Attention badge in sidebar", () => {
  test("activity button is visible in sidebar", async ({ mockedPage: page }) => {
    // The mock returns attention items count [2, 1] (critical+warning, info)
    const activityButton = page.getByTestId("sidebar-activity");
    await expect(activityButton).toBeVisible();
  });
});
