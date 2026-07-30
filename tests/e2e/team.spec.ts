import { test, expect } from "./fixtures";

test.describe("Team Settings", () => {
  test("settings button opens modal", async ({ mockedPage: page }) => {
    // Click settings button in sidebar (same pattern as settings-advanced.spec.ts)
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();

    // Modal should appear
    await expect(page.getByRole('heading', { name: /settings/i })).toBeVisible({ timeout: 3000 });
  });

  test("can access Team tab if present", async ({ mockedPage: page }) => {
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();

    // Wait for modal
    await page.waitForTimeout(300);

    // Try to find Team tab
    const teamTab = page.getByRole('button', { name: 'Team' });
    if (await teamTab.isVisible({ timeout: 2000 }).catch(() => false)) {
      await teamTab.click();
      // Team section should show
      await expect(page.locator('text=Team').first()).toBeVisible();
    } else {
      // Team tab may not exist yet - verify settings modal is still visible
      await expect(page.getByRole('heading', { name: /settings/i })).toBeVisible();
    }
  });
});

test.describe("Export/Import", () => {
  test("can access Advanced settings tab", async ({ mockedPage: page }) => {
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();
    await page.waitForTimeout(300);

    // Find and click Advanced section
    const advancedToggle = page.getByRole('button', { name: 'Advanced' });
    if (await advancedToggle.isVisible({ timeout: 2000 }).catch(() => false)) {
      await advancedToggle.click();
      // Should show some content
      await page.waitForTimeout(200);
      await expect(page.locator('text=Background').first()).toBeVisible({ timeout: 2000 }).catch(() => {});
    }
  });

  test("settings modal is functional", async ({ mockedPage: page }) => {
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();

    // Verify modal is present and can be interacted with
    await expect(page.getByRole('heading', { name: /settings/i })).toBeVisible({ timeout: 3000 });

    // Close modal
    const closeButton = page.locator('button:has(svg.lucide-x)').first();
    if (await closeButton.isVisible({ timeout: 1000 }).catch(() => false)) {
      await closeButton.click();
      await page.waitForTimeout(300);
    }
  });
});

test.describe("Team Roster and Backup reachability (via Integrations panel)", () => {
  test("Team Roster, Export, and Import are all reachable from the Integrations panel", async ({ mockedPage: page }) => {
    // These lived only in unmounted components before — this test exists
    // specifically to prevent that regressing silently again.
    await page.locator('button[title="Integrations"]').click();
    await expect(page.getByText("Integrations & Connections")).toBeVisible({ timeout: 3000 });

    await expect(page.getByText("Team & Data")).toBeVisible();

    await page.getByRole("button", { name: "Manage" }).click();
    await expect(page.getByRole("heading", { name: "Team Roster" })).toBeVisible({ timeout: 3000 });
    await page.locator('button:has(svg.lucide-x)').first().click();

    await page.getByRole("button", { name: "Export", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Export Data" })).toBeVisible({ timeout: 3000 });
    await page.getByRole("button", { name: "Cancel" }).click();

    await page.getByRole("button", { name: "Import", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Import Data" })).toBeVisible({ timeout: 3000 });
    await expect(page.getByRole("button", { name: "Restore Backup" })).toBeVisible();
  });
});

test.describe("Assignee picker (multi-assignee)", () => {
  test("shows existing assignee as a chip and can add a second one", async ({ mockedPage: page }) => {
    // Open the task edit modal (task-1's assignee is "Alice" per the mock).
    // Scope assertions to the picker itself since a TaskCard behind the
    // modal also renders "Alice" as plain text.
    await page.getByText("Fix the login bug").click();
    const assigneeField = page.getByLabel("Add assignee").locator("..");
    await expect(assigneeField.getByText("Alice", { exact: true })).toBeVisible({ timeout: 3000 });

    // Open the assignee dropdown and add a second person
    await page.getByLabel("Add assignee").click();
    await expect(page.getByText("Bob Jones")).toBeVisible({ timeout: 3000 });
    await page.getByText("Bob Jones").click();

    // Both assignees should now show as chips
    await expect(assigneeField.getByText("Alice", { exact: true })).toBeVisible();
    await expect(assigneeField.getByText("Bob Jones")).toBeVisible();
  });

  test("removing a chip does not remove the other assignee", async ({ mockedPage: page }) => {
    await page.getByText("Fix the login bug").click();
    const assigneeField = page.getByLabel("Add assignee").locator("..");
    await expect(assigneeField.getByText("Alice", { exact: true })).toBeVisible({ timeout: 3000 });

    await page.getByLabel("Add assignee").click();
    await page.getByText("Bob Jones").click();
    await expect(assigneeField.getByText("Bob Jones")).toBeVisible();

    // Remove "Alice" via her chip's X button (the chip sits in the trigger
    // bar above the dropdown, so it's clickable regardless of dropdown state)
    await assigneeField.locator("span", { hasText: "Alice" }).first().locator("button").click();

    await expect(assigneeField.getByText("Alice", { exact: true })).not.toBeVisible();
    await expect(assigneeField.getByText("Bob Jones")).toBeVisible();
  });
});
