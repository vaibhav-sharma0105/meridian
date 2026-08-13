import { test, expect } from "./fixtures";

test.describe("Message Center", () => {
  test("shows Messages sidebar entry", async ({ mockedPage: page }) => {
    await expect(page.getByTestId("sidebar-messages")).toBeVisible();
  });

  test("clicking Messages navigates to the Message Center", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    await expect(
      page.getByRole("heading", { name: "Message Center" })
    ).toBeVisible();
  });

  test("lists messages from the backend", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    await expect(page.getByText("Weekly Progress Report")).toBeVisible();
    await expect(page.getByText("Architecture discussion")).toBeVisible();
  });

  test("shows the message type badge", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    await expect(page.getByText("Skill Result").first()).toBeVisible();
  });

  test("labels integration_sync messages instead of showing the raw type", async ({
    mockedPage: page,
  }) => {
    await page.getByTestId("sidebar-messages").click();
    // `exact` is required: the filter dropdown also contains an
    // "Integration Syncs" option, which substring-matches otherwise.
    await expect(page.getByText("Integration Sync", { exact: true })).toBeVisible();
    // The raw identifier leaking through means TYPE_LABELS is missing a key.
    await expect(page.getByText("integration_sync", { exact: true })).toHaveCount(0);
  });

  test("offers a filter for every producible message type", async ({
    mockedPage: page,
  }) => {
    await page.getByTestId("sidebar-messages").click();
    const options = await page
      .getByLabel("Filter by message type")
      .locator("option")
      .allTextContents();

    expect(options).toContain("Skill Results");
    expect(options).toContain("Integration Syncs");
    expect(options).toContain("Pinned Chats");
    expect(options).toContain("Digests");
    // Nothing produces these — offering them yields permanently empty results.
    expect(options).not.toContain("Drafts");
    expect(options).not.toContain("Reports");
  });

  test("shows attached file references", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    // file_refs are rendered basename-only
    await expect(page.getByText("summary.md")).toBeVisible();
  });

  test("filters by message type", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    const typeFilter = page.getByLabel("Filter by message type");
    await typeFilter.selectOption("skill_result");
    await expect(typeFilter).toHaveValue("skill_result");
  });

  test("search box accepts input", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    const search = page.getByPlaceholder("Search messages...");
    await search.fill("architecture");
    await expect(search).toHaveValue("architecture");
  });

  test("shows the storage usage indicator", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    await expect(page.getByText("Storage")).toBeVisible();
    await expect(page.getByText("3 messages")).toBeVisible();
  });

  test("settings toggle reveals retention controls", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    await page.getByLabel("Message Center settings").click();

    await expect(page.getByText("AI context window")).toBeVisible();
    await expect(page.getByText("Keep messages for")).toBeVisible();
  });

  test("retention settings reflect the stored profile", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    await page.getByLabel("Message Center settings").click();

    // Mock profile: ai_context_days = 30, message_retention = "forever"
    await expect(page.getByLabel("AI context window")).toHaveValue("30");
    await expect(page.getByLabel("Keep messages for")).toHaveValue("forever");
  });

  test("explains the dual retention model", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-messages").click();
    await page.getByLabel("Message Center settings").click();
    await expect(
      page.getByText(/only sees messages inside the context window/i)
    ).toBeVisible();
  });
});
