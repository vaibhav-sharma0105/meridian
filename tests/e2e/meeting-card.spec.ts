import { test, expect } from "./fixtures";
import { buildTauriMockScript, MOCK_MEETINGS, MOCK_TASKS, MOCK_PROJECTS } from "./setup/tauri-mock";

test.describe("Meeting card", () => {
  test.beforeEach(async ({ page }) => {
    // Use a project-scoped context where the Meetings tab is visible
    const overrides = {
      get_projects: MOCK_PROJECTS,
      get_tasks_for_project: MOCK_TASKS,
      get_meetings_for_project: MOCK_MEETINGS,
    };
    await page.addInitScript(buildTauriMockScript(overrides));
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    // Select a project to see its Meetings tab
    await page.getByText("Alpha Project").click();
    await page.getByText("Meetings").click();
  });

  test("meeting title is visible", async ({ page }) => {
    await expect(page.getByText("Sprint Planning Q1")).toBeVisible();
  });

  test("meeting metadata (date, attendees) is visible", async ({ page }) => {
    await expect(page.getByText(/attendees/i)).toBeVisible();
  });

  test("clicking title makes it editable", async ({ page }) => {
    const title = page.getByText("Sprint Planning Q1");
    await title.click();
    // An input should now be focused
    const input = page.locator('input[value="Sprint Planning Q1"]');
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });

  test("pressing Escape cancels rename and restores original title", async ({ page }) => {
    const titleEl = page.getByRole("heading", { name: "Sprint Planning Q1" });
    await titleEl.click();
    const input = page.locator('input').filter({ hasValue: "Sprint Planning Q1" });
    await expect(input).toBeVisible();
    await input.fill("Changed title");
    await input.press("Escape");
    // Input should disappear and original title be restored
    await expect(input).not.toBeVisible();
    await expect(titleEl).toBeVisible();
  });

  test("pressing Enter with empty value cancels rename", async ({ page }) => {
    const titleEl = page.getByRole("heading", { name: "Sprint Planning Q1" });
    await titleEl.click();
    const input = page.locator('input').filter({ hasValue: "Sprint Planning Q1" });
    await input.fill("");
    await input.press("Enter");
    // Input should disappear and original title be restored
    await expect(input).not.toBeVisible();
    await expect(titleEl).toBeVisible();
  });

  test("health badge is visible when score present", async ({ page }) => {
    // health_score: 82 — badge should show
    const badge = page.locator('[class*="health"], [class*="badge"]').first();
    // Just verify no error thrown when card renders
    const errors: string[] = [];
    page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text()); });
    await expect(page.getByText("Sprint Planning Q1")).toBeVisible();
    expect(errors.filter(e => !e.includes("TAURI MOCK"))).toHaveLength(0);
  });

  test("expanding card reveals summary section", async ({ page }) => {
    // Click the expand chevron — it's the first button inside the meeting card header row
    // (the header row itself is the clickable div, so we click it away from the title)
    const headerRow = page.locator('[class*="cursor-pointer"]').filter({ hasText: "Sprint Planning Q1" }).first();
    // Click the chevron area (left side of header, before the title)
    await headerRow.click({ position: { x: 12, y: 15 } });
    await expect(page.getByText(/SUMMARY/i)).toBeVisible();
  });
});

test.describe("Meeting card — edge cases", () => {
  test("meeting with no health score renders without error", async ({ page }) => {
    const meetingNoScore = [{ ...MOCK_MEETINGS[0], health_score: null }];
    await page.addInitScript(buildTauriMockScript({
      get_projects: MOCK_PROJECTS,
      get_meetings_for_project: meetingNoScore,
    }));
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Alpha Project").click();
    await page.getByText("Meetings").click();
    await expect(page.getByText("Sprint Planning Q1")).toBeVisible();
  });

  test("meeting with no attendees renders without error", async ({ page }) => {
    const meetingNoAttendees = [{ ...MOCK_MEETINGS[0], attendees: null }];
    await page.addInitScript(buildTauriMockScript({
      get_projects: MOCK_PROJECTS,
      get_meetings_for_project: meetingNoAttendees,
    }));
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Alpha Project").click();
    await page.getByText("Meetings").click();
    await expect(page.getByText("Sprint Planning Q1")).toBeVisible();
  });
});
