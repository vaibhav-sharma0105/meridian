import { test, expect, mountWithMocks } from "./fixtures";

test.describe("Role indicator", () => {
  test("shows the confirmed role in the Activity header", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-activity").click();
    await expect(page.getByText("Tech Lead")).toBeVisible();
  });

  test("tooltip explains the role on hover", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-activity").click();
    await page.getByText("Tech Lead").hover();
    await expect(page.getByText(/Reviews code, unblocks team/i)).toBeVisible();
  });

  test("no drift banner when the backend reports no drift", async ({ mockedPage: page }) => {
    await page.getByTestId("sidebar-activity").click();
    await expect(page.getByText("Role Change Detected")).toHaveCount(0);
  });
});

test.describe("Role confirmation", () => {
  test("prompts for confirmation when inference is pending", async ({ page }) => {
    await mountWithMocks(page, {
      get_role_inference_status: {
        type: "PendingConfirmation",
        inferred: "ic",
        confidence: 0.62,
      },
    });
    await page.getByTestId("sidebar-activity").click();

    await expect(page.getByText("Confirm?")).toBeVisible();
    await expect(page.getByText("Individual Contributor")).toBeVisible();
  });

  test("opens the confirmation modal when the indicator is clicked", async ({ page }) => {
    await mountWithMocks(page, {
      get_role_inference_status: {
        type: "PendingConfirmation",
        inferred: "ic",
        confidence: 0.62,
      },
    });
    await page.getByTestId("sidebar-activity").click();
    await page.getByText("Confirm?").click();

    await expect(
      page.getByRole("heading", { name: "Confirm Your Role" })
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Confirm Role" })
    ).toBeVisible();
  });

  test("modal lists all four roles with scores from the profile", async ({ page }) => {
    await mountWithMocks(page, {
      get_role_inference_status: {
        type: "PendingConfirmation",
        inferred: "ic",
        confidence: 0.62,
      },
    });
    await page.getByTestId("sidebar-activity").click();
    await page.getByText("Confirm?").click();

    await expect(page.getByText("People Manager")).toBeVisible();
    await expect(page.getByText("Product Manager")).toBeVisible();
    // Scores come from get_user_profile.role_scores (tech_lead 0.45 -> 45%)
    await expect(page.getByText("45%")).toBeVisible();
  });

  test("shows learning state before the activity threshold", async ({ page }) => {
    await mountWithMocks(page, {
      get_role_inference_status: {
        type: "Learning",
        message: "Getting to know your role...",
        progress: 40,
      },
    });
    await page.getByTestId("sidebar-activity").click();

    await expect(page.getByText("Learning...")).toBeVisible();
  });
});

test.describe("Role drift", () => {
  test("shows the drift banner when the backend reports drift", async ({ page }) => {
    await mountWithMocks(page, {
      get_role_drift_alert: {
        previous_role: "ic",
        suggested_role: "tech_lead",
        confidence: 0.55,
      },
    });
    await page.getByTestId("sidebar-activity").click();

    await expect(page.getByText("Role Change Detected")).toBeVisible();
    await expect(page.getByRole("button", { name: "Update Role" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Keep Current" })).toBeVisible();
  });

  test("dismissing the drift banner hides it", async ({ page }) => {
    await mountWithMocks(page, {
      get_role_drift_alert: {
        previous_role: "ic",
        suggested_role: "tech_lead",
        confidence: 0.55,
      },
    });
    await page.getByTestId("sidebar-activity").click();
    await page.getByRole("button", { name: "Keep Current" }).click();

    await expect(page.getByText("Role Change Detected")).toHaveCount(0);
  });
});
