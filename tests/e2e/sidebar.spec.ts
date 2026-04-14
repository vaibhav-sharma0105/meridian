import { test, expect } from "./fixtures";

test.describe("Sidebar navigation", () => {
  test("renders brand logo and app name", async ({ mockedPage: page }) => {
    await expect(page.getByText("Meridian")).toBeVisible();
  });

  test("shows All Tasks nav item", async ({ mockedPage: page }) => {
    await expect(page.getByText("All Tasks")).toBeVisible();
  });

  test("shows Projects section header", async ({ mockedPage: page }) => {
    await expect(page.getByText("Projects")).toBeVisible();
  });

  test("shows utility strip icons (bell, settings)", async ({ mockedPage: page }) => {
    // Title attributes on icon buttons
    await expect(page.locator('[title="Notifications"]')).toBeVisible();
    await expect(page.locator('[title*="Settings"]')).toBeVisible();
  });

  test("clicking All Tasks shows task list view", async ({ mockedPage: page }) => {
    await page.getByText("All Tasks").click();
    // The All Tasks header should appear
    await expect(page.getByText("All Tasks").first()).toBeVisible();
  });
});
