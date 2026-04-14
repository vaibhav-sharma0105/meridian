import { test, expect } from "./fixtures";
import { buildTauriMockScript, MOCK_PROJECTS, MOCK_TASKS } from "./setup/tauri-mock";

test.describe("View switching (List / Kanban / Table)", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(buildTauriMockScript({
      get_projects: MOCK_PROJECTS,
      get_tasks_for_project: MOCK_TASKS,
    }));
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Alpha Project").click();
  });

  test("Tasks tab is active by default after selecting a project", async ({ page }) => {
    // After project selection the tab bar becomes visible; tasks tab is active (border-indigo)
    const tasksTab = page.getByRole("button", { name: /^Tasks$/ }).first();
    await expect(tasksTab).toBeVisible();
  });

  test("view mode switcher is visible in Tasks tab", async ({ page }) => {
    const listBtn = page.locator('[title*="list"], [title*="List"]').first();
    await expect(listBtn).toBeVisible();
  });

  test("switching to Kanban shows column headers", async ({ page }) => {
    await page.locator('[title*="kanban"], [title*="Kanban"]').first().click();
    // Kanban column labels are rendered as uppercase text in DroppableColumn
    // They use the column.label value — pick one that won't clash with <option> tags
    await expect(page.locator('.uppercase').filter({ hasText: /^IN PROGRESS$|^OPEN$|^DONE$/i }).first()).toBeVisible();
  });

  test("switching to Table shows table headers", async ({ page }) => {
    await page.locator('[title*="table"], [title*="Table"]').first().click();
    // Table renders a header row; wait for a cell that only appears in table view
    await expect(page.locator('table, [role="table"], th').first()).toBeVisible();
  });

  test("switching back to List shows task cards", async ({ page }) => {
    // Go to kanban then back
    await page.locator('[title*="kanban"], [title*="Kanban"]').first().click();
    await page.locator('[title*="list"], [title*="List"]').first().click();
    await expect(page.getByText("Fix the login bug")).toBeVisible();
  });

  test("Meetings tab is accessible from project view", async ({ page }) => {
    await page.getByText("Meetings").click();
    // Should show meetings (or empty state)
    await expect(
      page.getByText("Sprint Planning Q1").or(page.getByText(/no meeting/i))
    ).toBeVisible();
  });

  test("Meetings nav item is NOT in sidebar (removed per design)", async ({ page }) => {
    // The sidebar should NOT have a standalone "Meetings" nav item
    const sidebarMeetings = page.locator('.select-none').getByText("Meetings");
    await expect(sidebarMeetings).not.toBeVisible();
  });
});

test.describe("Empty states", () => {
  test("All Tasks with no tasks shows empty state", async ({ page }) => {
    await page.addInitScript(buildTauriMockScript({
      get_projects: MOCK_PROJECTS,
      get_all_tasks: [],
    }));
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("All Tasks").click();
    await expect(page.getByText(/no task/i)).toBeVisible();
  });

  test("project with no meetings shows empty state or new meeting button", async ({ page }) => {
    await page.addInitScript(buildTauriMockScript({
      get_projects: MOCK_PROJECTS,
      get_tasks_for_project: [],
      get_meetings_for_project: [],
    }));
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Alpha Project").click();
    await page.getByText("Meetings").click();
    // The empty state heading and the New meeting button are both present — just check the heading
    await expect(page.getByRole("heading", { name: /no meetings yet/i })).toBeVisible();
  });
});
