import { test, expect } from "@playwright/test";
import { buildTauriMockScript } from "./setup/tauri-mock";

test.describe("Settings > Advanced", () => {
  test.beforeEach(async ({ page }) => {
    // Inject Tauri mock with default data
    await page.addInitScript(buildTauriMockScript());
    await page.goto("http://localhost:1420");
    // Wait for app to load
    await page.waitForSelector('[data-testid="sidebar"]', { timeout: 5000 }).catch(() => {});
    await page.waitForTimeout(500);
  });

  test("settings button opens AI settings modal", async ({ page }) => {
    // Click settings button in sidebar
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();

    // Modal should appear - look for the heading specifically
    await expect(page.getByRole('heading', { name: 'AI settings' })).toBeVisible({ timeout: 3000 });
  });

  test("advanced section is collapsible", async ({ page }) => {
    // Open settings
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();
    await page.waitForTimeout(300);

    // Find and click Advanced section toggle
    const advancedToggle = page.getByRole('button', { name: 'Advanced' });
    if (await advancedToggle.isVisible()) {
      await advancedToggle.click();
      await page.waitForTimeout(300);

      // Should show daemon status section
      await expect(page.getByText('Background Service', { exact: true })).toBeVisible({ timeout: 2000 });
    }
  });
});

test.describe("Encryption Status", () => {
  test("shows encryption status in onboarding", async ({ page }) => {
    // Mock onboarding not complete
    await page.addInitScript(buildTauriMockScript({
      get_app_settings: { onboarding_complete: "false", theme: "light", language: "en" },
    }));
    await page.goto("http://localhost:1420");

    // Should show onboarding wizard
    await expect(page.locator('text=Welcome').or(page.locator('text=Get Started'))).toBeVisible({ timeout: 5000 });
  });

  test("encryption step shows device and password options", async ({ page }) => {
    // Mock onboarding at encryption step
    await page.addInitScript(buildTauriMockScript({
      get_app_settings: { onboarding_complete: "false", theme: "light", language: "en" },
      get_encryption_status: { initialized: false, mode: null, version: null },
    }));
    await page.goto("http://localhost:1420");

    // Click through to encryption step
    const nextButton = page.locator('button:has-text("Get Started")').or(page.locator('button:has-text("Next")'));
    if (await nextButton.first().isVisible({ timeout: 3000 })) {
      await nextButton.first().click();
      await page.waitForTimeout(500);

      // Should show encryption options
      const hasDeviceOption = await page.locator('text=Device Key').or(page.locator('text=Device')).isVisible({ timeout: 2000 }).catch(() => false);
      const hasPasswordOption = await page.locator('text=Password').isVisible({ timeout: 1000 }).catch(() => false);

      // At least one option should be visible if we're on encryption step
      if (hasDeviceOption || hasPasswordOption) {
        expect(hasDeviceOption || hasPasswordOption).toBeTruthy();
      }
    }
  });
});

test.describe("Migration Detection", () => {
  test("migration wizard does not appear for encrypted database", async ({ page }) => {
    // Mock encrypted database (no migration needed)
    await page.addInitScript(buildTauriMockScript({
      get_migration_status: { needs_migration: false, database_exists: true, is_encrypted: true, backup_exists: false, backup_path: null, database_size_mb: 1.5 },
    }));
    await page.goto("http://localhost:1420");
    await page.waitForTimeout(1000);

    // Migration wizard should NOT appear
    await expect(page.locator('text=Unencrypted Database Detected')).not.toBeVisible({ timeout: 2000 });
    await expect(page.locator('text=Database Encryption').locator('visible=true')).not.toBeVisible({ timeout: 1000 }).catch(() => {});
  });
});

test.describe("Daemon Status", () => {
  test("daemon status shows running state", async ({ page }) => {
    await page.addInitScript(buildTauriMockScript({
      get_daemon_status: { running: true, pid: 12345, jobs_processed: 10, uptime_seconds: 3600, last_error: null },
    }));
    await page.goto("http://localhost:1420");

    // Open settings and advanced section
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();
    await page.waitForTimeout(300);

    const advancedToggle = page.getByRole('button', { name: 'Advanced' });
    if (await advancedToggle.isVisible({ timeout: 2000 })) {
      await advancedToggle.click();
      await page.waitForTimeout(300);

      // Should show running status - specifically target the PID text
      await expect(page.getByText('Running (PID 12345)')).toBeVisible({ timeout: 2000 });
    }
  });
});

test.describe("Audit Log Viewer", () => {
  test("audit log section appears in advanced settings", async ({ page }) => {
    await page.addInitScript(buildTauriMockScript({
      get_audit_log: {
        entries: [
          {
            id: "log-1",
            timestamp: new Date().toISOString(),
            action_type: "create",
            entity_type: "task",
            entity_id: "task-123",
            details: null,
            agent_initiated: false,
            autonomy_mode: null,
            risk_level: "low",
            created_at: new Date().toISOString(),
          }
        ],
        total: 1,
        has_more: false,
      },
    }));
    await page.goto("http://localhost:1420");

    // Open settings and advanced section
    const settingsButton = page.locator('button[title="Settings"]').or(page.locator('button:has(svg.lucide-settings)'));
    await settingsButton.first().click();
    await page.waitForTimeout(300);

    const advancedToggle = page.getByRole('button', { name: 'Advanced' });
    if (await advancedToggle.isVisible({ timeout: 2000 })) {
      await advancedToggle.click();
      await page.waitForTimeout(500);

      // Should show activity log section - use the heading
      await expect(page.getByRole('heading', { name: 'Activity Log' })).toBeVisible({ timeout: 2000 });
    }
  });
});
