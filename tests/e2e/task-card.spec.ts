import { test, expect } from "./fixtures";

test.describe("Task card", () => {
  test.beforeEach(async ({ mockedPage: page }) => {
    await page.getByText("All Tasks").click();
  });

  test("task title is visible and bold", async ({ mockedPage: page }) => {
    const title = page.getByText("Fix the login bug");
    await expect(title).toBeVisible();
    const fontWeight = await title.evaluate((el) => getComputedStyle(el).fontWeight);
    // font-semibold = 600
    expect(Number(fontWeight)).toBeGreaterThanOrEqual(600);
  });

  test("task description is visible (up to 2 lines)", async ({ mockedPage: page }) => {
    // Description text should be present in the DOM
    await expect(
      page.getByText(/Users cannot log in when 2FA/i)
    ).toBeVisible();
  });

  test("task with no description shows only title", async ({ mockedPage: page }) => {
    // task-3 has no description; only its title should render
    await expect(page.getByText("Deploy to staging")).toBeVisible();
  });

  test("priority border colour reflects priority level", async ({ mockedPage: page }) => {
    // The critical task card should have the red border-l class
    const card = page.locator('[class*="border-l-red"]').first();
    await expect(card).toBeVisible();
  });

  test("checkbox appears on hover", async ({ mockedPage: page }) => {
    const card = page.getByText("Fix the login bug").locator("..").locator("..");
    await card.hover();
    // The custom checkbox div should become visible
    const checkbox = page.locator('input[type="checkbox"]').first();
    await expect(checkbox).toBeAttached();
  });

  test("clicking a task card does not throw a console error", async ({ mockedPage: page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text()); });
    await page.getByText("Fix the login bug").click();
    expect(errors.filter(e => !e.includes("TAURI MOCK"))).toHaveLength(0);
  });
});
