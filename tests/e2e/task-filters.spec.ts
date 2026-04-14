import { test, expect } from "./fixtures";

test.describe("Task filter bar", () => {
  test.beforeEach(async ({ mockedPage: page }) => {
    // Navigate to All Tasks view
    await page.getByText("All Tasks").click();
  });

  test("search input is visible and accepts text", async ({ mockedPage: page }) => {
    const search = page.locator('input[placeholder*="Search"]').first();
    await expect(search).toBeVisible();
    await search.fill("login bug");
    await expect(search).toHaveValue("login bug");
  });

  test("status filter select is present", async ({ mockedPage: page }) => {
    const statusSelect = page.locator('select').first();
    await expect(statusSelect).toBeVisible();
  });

  test("priority filter select is present", async ({ mockedPage: page }) => {
    const selects = page.locator('select');
    await expect(selects.nth(1)).toBeVisible();
  });

  test("date filter button shows 'Created date' when inactive", async ({ mockedPage: page }) => {
    await expect(page.getByText("Created date")).toBeVisible();
  });

  test("date filter popover opens on click", async ({ mockedPage: page }) => {
    await page.getByText("Created date").click();
    await expect(page.getByText("Today")).toBeVisible();
    await expect(page.getByText("Last 7 days")).toBeVisible();
    await expect(page.getByText("Last 30 days")).toBeVisible();
    await expect(page.getByText("Last 3 months")).toBeVisible();
    await expect(page.getByText("Last year")).toBeVisible();
    await expect(page.getByText("Custom range")).toBeVisible();
  });

  test("selecting 'Today' preset closes popover and shows active chip", async ({ mockedPage: page }) => {
    await page.getByText("Created date").click();
    await page.getByRole("button", { name: "Today" }).click();
    // Popover should close
    await expect(page.getByText("Last 7 days")).not.toBeVisible();
    // Button now shows the selected preset
    await expect(page.getByText("Today")).toBeVisible();
  });

  test("clear filters button appears when a filter is active", async ({ mockedPage: page }) => {
    const search = page.locator('input[placeholder*="Search"]').first();
    await search.fill("test query");
    await expect(page.getByText(/clear/i)).toBeVisible();
  });

  test("clearing filters resets search input", async ({ mockedPage: page }) => {
    const search = page.locator('input[placeholder*="Search"]').first();
    await search.fill("test query");
    await page.getByText(/clear/i).click();
    await expect(search).toHaveValue("");
  });

  test("custom date range inputs appear when Custom is selected", async ({ mockedPage: page }) => {
    await page.getByText("Created date").click();
    await page.getByText("Custom range").click();
    await expect(page.locator('input[type="date"]').first()).toBeVisible();
    await expect(page.locator('input[type="date"]').nth(1)).toBeVisible();
    await expect(page.getByRole("button", { name: "Apply" })).toBeVisible();
  });

  test("Apply button is disabled when no custom dates entered", async ({ mockedPage: page }) => {
    await page.getByText("Created date").click();
    await page.getByText("Custom range").click();
    const applyBtn = page.getByRole("button", { name: "Apply" });
    await expect(applyBtn).toBeDisabled();
  });
});
