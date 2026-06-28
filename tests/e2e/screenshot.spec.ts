import { test } from "./fixtures";

test("screenshot bulk bar", async ({ mockedPage: page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });

  await page.click("text=Alpha Project");
  await page.waitForTimeout(400);

  // Default: no AI panel, kanban full width
  await page.screenshot({ path: "/tmp/kanban-no-ai.png" });

  // Inject a task selection directly into Zustand store
  await page.evaluate(() => {
    // Zustand stores are accessible via their module; React re-renders automatically.
    // We trigger toggleTaskSelection on the task card via a known task id.
    const storeKey = Object.keys((window as any)).find(k => k.includes("zustand"));
    // Fallback: dispatch a click on the card checkbox label
  });

  // Hover the first card to make the checkbox visible, then click the label
  await page.locator(".group").first().hover();
  await page.waitForTimeout(150);
  // The checkbox is sr-only but the label click still fires the onChange
  await page.locator("label").first().click({ force: true });
  await page.waitForTimeout(300);
  await page.screenshot({ path: "/tmp/bulk-bar.png" });
});
