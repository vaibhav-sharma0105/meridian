import { test, expect } from "./fixtures";
import { buildTauriMockScript, MOCK_PROJECTS } from "./setup/tauri-mock";

const MOCK_SKILLS = [
  {
    id: "skill-1",
    name: "Weekly Summary",
    description: "Generate a weekly progress summary",
    trigger_type: "schedule",
    trigger_config: JSON.stringify({ cron: "0 9 * * 1" }),
    context_config: JSON.stringify({ system_prompt: "# Instructions\n\nSummarize the week's progress using {{tasks}} and {{meetings}}." }),
    action_config: JSON.stringify({ action_type: "summarize" }),
    approval_mode: "notify",
    autonomy_mode: null,
    enabled: true,
    shared: false,
    owner_id: null,
    category: "reporting",
    icon: null,
    tags: JSON.stringify(["reporting", "weekly"]),
    next_run_at: "2026-07-21T09:00:00Z",
    cloned_from_id: null,
    is_builtin: true,
    // Phase 9 fields
    sync_source: null,
    sync_path: null,
    sync_commit: null,
    last_sync_check: null,
    content_hash: null,
    trust_state: null,
    trust_granted_at: null,
    network_mode: null,
    network_allowlist: null,
    created_at: "2026-07-01T00:00:00Z",
    updated_at: "2026-07-01T00:00:00Z",
  },
  {
    id: "skill-2",
    name: "Meeting Follow-up",
    description: "Draft follow-up after meetings",
    trigger_type: "event",
    trigger_config: JSON.stringify({ event_type: "meeting_imported" }),
    context_config: null,
    action_config: JSON.stringify({ action_type: "draft_message" }),
    approval_mode: "approve_first",
    autonomy_mode: null,
    enabled: true,
    shared: false,
    owner_id: null,
    category: "communication",
    icon: null,
    tags: JSON.stringify(["meetings"]),
    next_run_at: null,
    cloned_from_id: null,
    is_builtin: false,
    // Phase 9 fields
    sync_source: null,
    sync_path: null,
    sync_commit: null,
    last_sync_check: null,
    content_hash: null,
    trust_state: null,
    trust_granted_at: null,
    network_mode: null,
    network_allowlist: null,
    created_at: "2026-07-02T00:00:00Z",
    updated_at: "2026-07-02T00:00:00Z",
  },
  {
    id: "skill-3",
    name: "GitHub Watcher",
    description: "Watch GitHub for new PRs",
    trigger_type: "schedule",
    trigger_config: JSON.stringify({ cron: "0 */6 * * *" }),
    context_config: null,
    action_config: JSON.stringify({ action_type: "analyze" }),
    approval_mode: "notify",
    autonomy_mode: null,
    enabled: true,
    shared: false,
    owner_id: null,
    category: "custom",
    icon: null,
    tags: JSON.stringify(["github", "sync"]),
    next_run_at: "2026-07-21T12:00:00Z",
    cloned_from_id: null,
    is_builtin: false,
    // Phase 9 fields - synced skill
    sync_source: "github:example/skills-repo",
    sync_path: ".claude/skills/github-watcher",
    sync_commit: "abc123",
    last_sync_check: "2026-07-20T12:00:00Z",
    content_hash: "def456",
    trust_state: "trusted",
    trust_granted_at: "2026-07-15T10:00:00Z",
    network_mode: "allowlist",
    network_allowlist: JSON.stringify(["api.github.com"]),
    created_at: "2026-07-10T00:00:00Z",
    updated_at: "2026-07-15T10:00:00Z",
  },
];

const MOCK_SKILL_RUNS = [
  {
    id: "run-1",
    skill_id: "skill-1",
    status: "completed",
    trigger_type: "schedule",
    trigger_context: null,
    output: "Generated weekly summary: 5 tasks completed, 2 in progress.",
    error: null,
    pending_changes: null,
    started_at: "2026-07-14T09:00:00Z",
    completed_at: "2026-07-14T09:00:02Z",
    duration_ms: 2100,
    approval_decision: null,
    approval_reason: null,
    created_at: "2026-07-14T09:00:00Z",
  },
  {
    id: "run-2",
    skill_id: "skill-1",
    status: "failed",
    trigger_type: "schedule",
    trigger_context: null,
    output: null,
    error: "AI provider unreachable",
    pending_changes: null,
    started_at: "2026-07-07T09:00:00Z",
    completed_at: "2026-07-07T09:00:05Z",
    duration_ms: 5000,
    approval_decision: null,
    approval_reason: null,
    created_at: "2026-07-07T09:00:00Z",
  },
];

test.describe("Skills - Creation Flow", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(
      buildTauriMockScript({
        get_projects: MOCK_PROJECTS,
        list_skills: MOCK_SKILLS,
        get_skill_runs: MOCK_SKILL_RUNS,
        get_skill_stats: {
          total_runs: 10,
          completed_runs: 8,
          failed_runs: 2,
          success_rate: 0.8,
          avg_duration_ms: 2500,
          last_run_at: "2026-07-14T09:00:02Z",
        },
      })
    );
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Skills", { exact: true }).first().click();
  });

  test("skills list shows existing skills", async ({ page }) => {
    await expect(page.getByText("Weekly Summary")).toBeVisible();
    await expect(page.getByText("Meeting Follow-up")).toBeVisible();
  });

  test("skills page shows header with reset button", async ({ page }) => {
    await expect(page.getByText("Reset defaults")).toBeVisible();
  });

  test("category tabs are visible", async ({ page }) => {
    await expect(page.getByRole("button", { name: "All", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Productivity" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Reporting" })).toBeVisible();
  });

  test("clicking New Skill opens the editor modal in basic mode", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    await expect(page.getByRole("heading", { name: "Create Skill" })).toBeVisible();
    // Basic mode shows form fields - use label text
    await expect(page.locator("label").filter({ hasText: "Name *" })).toBeVisible();
    await expect(page.locator("label").filter({ hasText: "Instructions *" })).toBeVisible();
  });

  test("editor can switch to YAML mode", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    await page.getByRole("button", { name: "YAML" }).click();
    // YAML+MD editor shows frontmatter hint
    await expect(page.getByText("YAML frontmatter + Markdown body")).toBeVisible();
    const textarea = page.locator("textarea").first();
    const value = await textarea.inputValue();
    expect(value).toContain("---");
    expect(value).toContain("name:");
  });

  test("editor has Insert variable button", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    const varButton = page.getByRole("button", { name: /insert variable/i });
    await expect(varButton).toBeVisible();
    await varButton.click();
    await expect(page.getByText("{{tasks}}")).toBeVisible();
    await expect(page.getByText("{{meetings}}")).toBeVisible();
    await expect(page.getByText("{{project_name}}")).toBeVisible();
    await expect(page.getByText("{{date}}")).toBeVisible();
  });

  test("form submission creates skill and closes modal", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    // Fill basic mode fields
    await page.getByPlaceholder("My Skill").fill("My New Skill");
    await page.getByPlaceholder("What this skill does...").fill("Automates reporting");
    await page.locator("textarea").first().fill("Summarize the week's progress.");
    await page.getByRole("button", { name: /create skill/i }).click();
    await expect(page.getByText("Create Skill")).not.toBeVisible({ timeout: 5000 });
  });

  test("validation shows error when name is empty", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    const textarea = page.locator("textarea").first();
    await textarea.fill(`---
name: ""
trigger:
  type: manual
action:
  type: summarize
---

# Instructions

Do something.
`);
    await page.getByRole("button", { name: /create skill/i }).click();
    await expect(page.getByText("Name is required")).toBeVisible();
  });

  test("validation shows error when instructions missing", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    // Fill name but leave instructions empty
    await page.getByPlaceholder("My Skill").fill("Test Skill");
    await page.getByRole("button", { name: /create skill/i }).click();
    await expect(page.getByText("Instructions are required")).toBeVisible();
  });

  test("cancel button closes modal without saving", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(page.getByText("Create Skill")).not.toBeVisible();
  });
});

test.describe("Skills - History View", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(
      buildTauriMockScript({
        get_projects: MOCK_PROJECTS,
        list_skills: MOCK_SKILLS,
        get_skill_runs: MOCK_SKILL_RUNS,
        get_skill_stats: {
          total_runs: 10,
          completed_runs: 8,
          failed_runs: 2,
          success_rate: 0.8,
          avg_duration_ms: 2500,
          last_run_at: "2026-07-14T09:00:02Z",
        },
      })
    );
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Skills", { exact: true }).first().click();
  });

  test("opening skill menu shows History option", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();

    await expect(page.getByText("History")).toBeVisible();
  });

  test("clicking History opens the history panel", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();

    await page.getByText("History").click();

    await expect(page.getByText("Run History")).toBeVisible();
    await expect(page.getByText("Total Runs")).toBeVisible();
    await expect(page.getByText("Success Rate")).toBeVisible();
  });

  test("history panel shows stats", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();
    await page.getByText("History").click();

    await expect(page.getByText("Total Runs")).toBeVisible();
    await expect(page.getByText("80%")).toBeVisible();
  });

  test("history panel shows status filter dropdown", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();
    await page.getByText("History").click();

    await expect(page.locator("select").filter({ hasText: "All" }).last()).toBeVisible();
  });

  test("run cards display with correct status", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();
    await page.getByText("History").click();

    await expect(page.getByText("Generated weekly summary")).toBeVisible();
  });

  test("failed runs show error text", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();
    await page.getByText("History").click();

    await expect(page.getByText("AI provider unreachable")).toBeVisible();
  });

  test("close button dismisses history panel", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();
    await page.getByText("History").click();

    await page.locator(".w-96").getByRole("button").first().click();
    await expect(page.getByText("Run History")).not.toBeVisible();
  });
});

test.describe("Skills - YAML+MD Editor", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(
      buildTauriMockScript({
        get_projects: MOCK_PROJECTS,
        list_skills: MOCK_SKILLS,
        get_skill_runs: MOCK_SKILL_RUNS,
        get_skill_stats: {
          total_runs: 10,
          completed_runs: 8,
          failed_runs: 2,
          success_rate: 0.8,
          avg_duration_ms: 2500,
          last_run_at: "2026-07-14T09:00:02Z",
        },
      })
    );
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Skills", { exact: true }).first().click();
  });

  test("editing existing skill loads fields in basic mode", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();
    await page.getByText("Edit").click();

    await expect(page.getByRole("heading", { name: "Edit Skill" })).toBeVisible();
    // Check name field has the skill name
    const nameInput = page.getByPlaceholder("My Skill");
    await expect(nameInput).toHaveValue("Weekly Summary");
  });

  test("editing existing skill shows Test button", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();
    await page.getByText("Edit").click();

    await expect(page.getByRole("button", { name: "Test" })).toBeVisible();
  });

  test("basic mode shows trigger type options", async ({ page }) => {
    await page.getByRole("button", { name: /new skill/i }).click();
    // Check for trigger option buttons - they have descriptions
    await expect(page.getByText("Run on demand")).toBeVisible();
    await expect(page.getByText("Run on a cron schedule")).toBeVisible();
    await expect(page.getByText("Run when an event occurs")).toBeVisible();
  });

  test("export menu option is available on skill cards", async ({ page }) => {
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();

    await expect(page.getByText("Export")).toBeVisible();
  });

  test("built-in skill shows Built-in badge", async ({ page }) => {
    await expect(page.getByText("Built-in").first()).toBeVisible();
  });

  test("built-in skill menu does not show Delete option", async ({ page }) => {
    // First skill (Weekly Summary) is built-in
    await page.locator(".space-y-3 > div").first()
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();

    await expect(page.getByText("Edit")).toBeVisible();
    await expect(page.getByText("Clone")).toBeVisible();
    await expect(page.getByText("Delete")).not.toBeVisible();
  });

  test("user-created skill menu shows Delete option", async ({ page }) => {
    // Second skill (Meeting Follow-up) is user-created
    await page.locator(".space-y-3 > div").nth(1)
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();

    await expect(page.getByText("Delete")).toBeVisible();
  });
});

test.describe("Skills - Phase 9: Sync & Trust", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(
      buildTauriMockScript({
        get_projects: MOCK_PROJECTS,
        list_skills: MOCK_SKILLS,
        get_skill_runs: MOCK_SKILL_RUNS,
        get_skill_stats: {
          total_runs: 10,
          completed_runs: 8,
          failed_runs: 2,
          success_rate: 0.8,
          avg_duration_ms: 2500,
          last_run_at: "2026-07-14T09:00:02Z",
        },
        check_skill_updates: { status: "up_to_date" },
        list_integrations: [],
      })
    );
    await page.goto("/");
    await page.waitForSelector("text=Meridian", { timeout: 15000 });
    await page.getByText("Skills", { exact: true }).first().click();
  });

  test("synced skill shows Synced badge", async ({ page }) => {
    // Third skill (GitHub Watcher) is synced
    await expect(page.getByText("Synced").first()).toBeVisible();
  });

  test("trusted skill shows Trusted badge", async ({ page }) => {
    // Third skill (GitHub Watcher) is trusted
    await expect(page.getByText("Trusted").first()).toBeVisible();
  });

  test("synced skill menu shows Check for Updates option", async ({ page }) => {
    // Third skill (GitHub Watcher) is synced
    await page.locator(".space-y-3 > div").nth(2)
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();

    await expect(page.getByText("Check for Updates")).toBeVisible();
  });

  test("synced skill menu shows Manage Trust option", async ({ page }) => {
    // Third skill (GitHub Watcher) is synced
    await page.locator(".space-y-3 > div").nth(2)
      .locator("button")
      .filter({ has: page.locator("svg") })
      .last()
      .click();

    await expect(page.getByText("Manage Trust")).toBeVisible();
  });

  test("Import from GitHub button is visible", async ({ page }) => {
    await expect(page.getByRole("button", { name: /import from github/i })).toBeVisible();
  });
});
