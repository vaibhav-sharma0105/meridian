import { test, expect, mountWithMocks } from "./fixtures";

const digestMessage = {
  id: "msg_digest_1",
  project_id: null,
  message_type: "digest",
  title: "Daily digest — Aug 12, 2026",
  content: "## Daily digest\n\n- **2** task(s) completed today\n",
  source_id: null,
  source_type: "digest",
  auto_pinned: false,
  pinned_reason: null,
  file_refs: null,
  ai_visible_until: null,
  deleted_at: null,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

const linkedNotification = {
  id: "notif_1",
  type: "digest",
  title: "Daily digest — Aug 12, 2026",
  body: "View full digest",
  task_id: null,
  project_id: null,
  skill_run_id: null,
  integration_id: null,
  severity: "info",
  desktop: false,
  is_read: false,
  created_at: new Date().toISOString(),
  message_id: "msg_digest_1",
};

test.describe("Notification deep link to Message Center", () => {
  test("shows 'View full result' only when the notification carries a message_id", async ({
    page,
  }) => {
    await mountWithMocks(page, {
      get_notifications: [
        linkedNotification,
        { ...linkedNotification, id: "notif_2", title: "Plain note", message_id: null },
      ],
    });

    await page.getByTitle(/^Notifications/).click();
    await expect(page.getByRole("button", { name: /View full result/i })).toHaveCount(1);
  });

  test("clicking the link opens Message Center and highlights the message", async ({
    page,
  }) => {
    await mountWithMocks(page, {
      get_notifications: [linkedNotification],
      get_messages: { messages: [digestMessage], total: 1 },
    });

    await page.getByTitle(/^Notifications/).click();
    await page.getByRole("button", { name: /View full result/i }).click();

    // Notification panel closes and the Message Center takes over the canvas.
    await expect(page.getByText("Message Center")).toBeVisible();
    await expect(page.getByText("Daily digest — Aug 12, 2026")).toBeVisible();
  });

  test("no link is rendered when no notification references a message", async ({
    page,
  }) => {
    await mountWithMocks(page, {
      get_notifications: [{ ...linkedNotification, message_id: null }],
    });

    await page.getByTitle(/^Notifications/).click();
    await expect(page.getByRole("button", { name: /View full result/i })).toHaveCount(0);
  });
});

test.describe("Identity settings", () => {
  test("identity section is reachable from AI settings", async ({ mockedPage: page }) => {
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.getByText("Your Identity").click();

    await expect(page.getByPlaceholder("Ada Lovelace")).toBeVisible();
    await expect(page.getByPlaceholder("ada@example.com")).toBeVisible();
  });

  test("warns when no identity is set, since role ordering silently degrades", async ({
    page,
  }) => {
    await mountWithMocks(page, {
      get_user_profile: {
        id: "default",
        inferred_role: "ic",
        secondary_role: null,
        custom_role_description: null,
        role_confirmed: true,
        role_confirmed_at: new Date().toISOString(),
        role_scores: null,
        last_inference_at: null,
        productivity_patterns: null,
        productivity_tracking_enabled: true,
        ai_context_days: 30,
        message_retention: "forever",
        display_name: null,
        user_email: null,
        user_aliases: [],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    });

    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.getByText("Your Identity").click();

    await expect(page.getByText(/falls back to sorting by severity only/i)).toBeVisible();
  });
});
