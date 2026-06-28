import { test } from "./fixtures";

test("screenshot all views", async ({ mockedPage: page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });

  // AI panel is closed by default — kanban full width
  await page.click("text=Alpha Project");
  await page.waitForTimeout(400);
  await page.screenshot({ path: "/tmp/kanban-no-ai.png" });

  // Click a task to open modal
  await page.click(".group >> nth=0");
  await page.waitForTimeout(300);
  await page.screenshot({ path: "/tmp/task-modal-light.png" });
  await page.keyboard.press("Escape");
  await page.waitForTimeout(200);

  // Open AI panel via Sparkles button
  await page.locator('button[title="Show AI panel"]').click();
  await page.waitForTimeout(400);
  await page.screenshot({ path: "/tmp/kanban-with-ai.png" });

  // Dark mode
  await page.evaluate(() => document.documentElement.classList.add("dark"));
  await page.waitForTimeout(200);
  await page.screenshot({ path: "/tmp/kanban-dark.png" });

  // Task modal dark
  await page.click(".group >> nth=0");
  await page.waitForTimeout(300);
  await page.screenshot({ path: "/tmp/task-modal-dark.png" });
});
