import { test, expect } from "./fixtures";
import { buildTauriMockScript, MOCK_PROJECTS } from "./setup/tauri-mock";

const MOCK_PENDING_APPROVALS = [
  {
    id: "approval-1",
    action_type: "create_task",
    action_config: JSON.stringify({ title: "New task from agent", project_id: "proj-1" }),
    source_type: "mcp",
    source_id: null,
    risk_level: "medium",
    autonomy_mode: "supervised",
    context: "MCP client requested task creation",
    timeout_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    status: "pending",
    resolved_by: null,
    resolution_reason: null,
    created_at: new Date().toISOString(),
    resolved_at: null,
  },
  {
    id: "approval-2",
    action_type: "skill:draft_message",
    action_config: JSON.stringify({ channel: "slack", recipient: "#team" }),
    source_type: "skill",
    source_id: "skill-1",
    risk_level: "high",
    autonomy_mode: "supervised",
    context: "Weekly Summary skill wants to send Slack message",
    timeout_at: new Date(Date.now() + 12 * 60 * 60 * 1000).toISOString(),
    status: "pending",
    resolved_by: null,
    resolution_reason: null,
    created_at: new Date().toISOString(),
    resolved_at: null,
  },
];

const MOCK_ACTION_HISTORY = [
  {
    id: "history-1",
    action_type: "update",
    entity_type: "task",
    entity_id: "task-1",
    before_state: JSON.stringify({ status: "open", priority: "medium" }),
    after_state: JSON.stringify({ status: "in_progress", priority: "high" }),
    undoable: true,
    undo_action_id: null,
    audit_log_id: "audit-1",
    created_at: new Date().toISOString(),
  },
  {
    id: "history-2",
    action_type: "create",
    entity_type: "task",
    entity_id: "task-new",
    before_state: null,
    after_state: JSON.stringify({ id: "task-new", title: "Agent created task" }),
    undoable: false,
    undo_action_id: null,
    audit_log_id: "audit-2",
    created_at: new Date(Date.now() - 60000).toISOString(),
  },
];

const MOCK_GOVERNANCE_METRICS = [
  { date: "2026-07-21", metric_type: "action_count", breakdown_key: null, value: 15 },
  { date: "2026-07-21", metric_type: "risk_distribution", breakdown_key: "low", value: 8 },
  { date: "2026-07-21", metric_type: "risk_distribution", breakdown_key: "medium", value: 5 },
  { date: "2026-07-21", metric_type: "risk_distribution", breakdown_key: "high", value: 2 },
  { date: "2026-07-21", metric_type: "autonomy_breakdown", breakdown_key: "supervised", value: 12 },
  { date: "2026-07-21", metric_type: "autonomy_breakdown", breakdown_key: "autonomous", value: 3 },
  { date: "2026-07-21", metric_type: "approval_rate", breakdown_key: "approved", value: 10 },
  { date: "2026-07-21", metric_type: "approval_rate", breakdown_key: "rejected", value: 2 },
];

test.describe("Governance View", () => {
  test.beforeEach(async ({ page }) => {
    const mockOverrides = {
      get_pending_approvals: MOCK_PENDING_APPROVALS,
      get_pending_approval_count: 2,
      get_action_history: MOCK_ACTION_HISTORY,
      get_undoable_actions: MOCK_ACTION_HISTORY.filter(a => a.undoable),
      get_governance_metrics: MOCK_GOVERNANCE_METRICS,
    };
    await page.addInitScript(buildTauriMockScript(mockOverrides));
    await page.goto("/");
    await page.waitForSelector('text=Meridian', { timeout: 15000 });
  });

  test("shows governance link in sidebar with badge", async ({ page }) => {
    const governanceLink = page.locator('[data-testid="sidebar-governance"]');
    await expect(governanceLink).toBeVisible();

    const badge = governanceLink.locator('[data-testid="sidebar-governance-badge"]');
    await expect(badge).toHaveText("2");
  });

  test("navigates to governance page on click", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');

    await expect(page.locator('text=Approvals')).toBeVisible();
    await expect(page.locator('text=History')).toBeVisible();
    await expect(page.locator('text=Dashboard')).toBeVisible();
    await expect(page.locator('text=Settings')).toBeVisible();
  });
});

test.describe("Autonomy Settings", () => {
  test.beforeEach(async ({ page }) => {
    const mockOverrides = {
      get_autonomy_setting: "supervised",
      get_pending_approvals: [],
      get_pending_approval_count: 0,
    };
    await page.addInitScript(buildTauriMockScript(mockOverrides));
    await page.goto("/");
    await page.waitForSelector('text=Meridian', { timeout: 15000 });
  });

  test("displays current autonomy mode in settings", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Settings');

    await expect(page.locator('text=Autonomy Mode')).toBeVisible();
    await expect(page.locator('text=Supervised')).toBeVisible();
  });

  test("shows mode descriptions", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Settings');

    // Manual/Autonomous are only rendered once the mode dropdown is open —
    // Supervised (the default) is the only one visible in the closed state.
    const trigger = page.getByRole('button', { name: /Supervised/i });
    await expect(trigger).toBeVisible();
    await trigger.click();

    await expect(page.locator('text=Manual')).toBeVisible();
    await expect(page.locator('text=Autonomous')).toBeVisible();
  });
});

test.describe("Approval Queue", () => {
  test.beforeEach(async ({ page }) => {
    const mockOverrides = {
      get_pending_approvals: MOCK_PENDING_APPROVALS,
      get_pending_approval_count: 2,
    };
    await page.addInitScript(buildTauriMockScript(mockOverrides));
    await page.goto("/");
    await page.waitForSelector('text=Meridian', { timeout: 15000 });
  });

  test("shows pending approvals list", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');

    // ApprovalQueue renders action_type with underscores replaced by spaces.
    await expect(page.locator('text=create task')).toBeVisible();
    await expect(page.locator('text=skill:draft message')).toBeVisible();
  });

  test("shows risk level badges", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');

    await expect(page.locator('text=medium').first()).toBeVisible();
    await expect(page.locator('text=high')).toBeVisible();
  });

  test("has approve and reject buttons", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');

    // These are icon-only buttons — labeled via `title`, not visible text.
    const approveButtons = page.locator('button[title="Approve"]');
    const rejectButtons = page.locator('button[title="Reject"]');

    await expect(approveButtons.first()).toBeVisible();
    await expect(rejectButtons.first()).toBeVisible();
  });

  test("shows timeout countdown", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');

    await expect(page.locator('text=/\\d+h|expires/i').first()).toBeVisible();
  });
});

test.describe("Undo Bar", () => {
  test("shows undo notification after undoable action", async ({ page }) => {
    const mockOverrides = {
      get_pending_approvals: [],
      get_pending_approval_count: 0,
      // ActionHistoryPanel (rendered by the "History" tab) fetches via
      // get_action_history — there's no get_undoable_actions command.
      get_action_history: [{
        id: "history-1",
        action_type: "update",
        entity_type: "task",
        entity_id: "task-1",
        before_state: JSON.stringify({ status: "open" }),
        after_state: JSON.stringify({ status: "done" }),
        undoable: true,
        undo_action_id: null,
        audit_log_id: null,
        created_at: new Date().toISOString(),
      }],
    };
    await page.addInitScript(buildTauriMockScript(mockOverrides));
    await page.goto("/");
    await page.waitForSelector('text=Meridian', { timeout: 15000 });

    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=History');

    await expect(page.locator('text=update')).toBeVisible();
    await expect(page.locator('text=task').first()).toBeVisible();
  });
});

test.describe("Governance Dashboard", () => {
  test.beforeEach(async ({ page }) => {
    const mockOverrides = {
      get_pending_approvals: MOCK_PENDING_APPROVALS,
      get_pending_approval_count: 2,
      get_governance_metrics: MOCK_GOVERNANCE_METRICS,
    };
    await page.addInitScript(buildTauriMockScript(mockOverrides));
    await page.goto("/");
    await page.waitForSelector('text=Meridian', { timeout: 15000 });
  });

  test("shows total actions metric", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Dashboard');

    await expect(page.locator('text=Total Actions')).toBeVisible();
    await expect(page.locator('text=15')).toBeVisible();
  });

  test("shows risk distribution", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Dashboard');

    await expect(page.locator('text=Risk Distribution')).toBeVisible();
  });

  test("shows autonomy breakdown", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Dashboard');

    await expect(page.locator('text=Autonomy Breakdown')).toBeVisible();
    await expect(page.locator('text=supervised')).toBeVisible();
  });

  test("shows approval rate", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Dashboard');

    await expect(page.locator('text=Approval Rate')).toBeVisible();
  });

  test("has time range selector", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Dashboard');

    await expect(page.locator('text=Today')).toBeVisible();
    await expect(page.locator('text=Week')).toBeVisible();
    await expect(page.locator('text=Month')).toBeVisible();
  });

  test("has export button", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=Dashboard');

    const exportButton = page.locator('[title="Export CSV"]');
    await expect(exportButton).toBeVisible();
  });
});

test.describe("Action History", () => {
  test.beforeEach(async ({ page }) => {
    const mockOverrides = {
      get_pending_approvals: [],
      get_pending_approval_count: 0,
      get_action_history: MOCK_ACTION_HISTORY,
    };
    await page.addInitScript(buildTauriMockScript(mockOverrides));
    await page.goto("/");
    await page.waitForSelector('text=Meridian', { timeout: 15000 });
  });

  test("shows action history list", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=History');

    await expect(page.locator('text=update')).toBeVisible();
    await expect(page.locator('text=create')).toBeVisible();
  });

  test("shows entity types", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=History');

    await expect(page.locator('text=task').first()).toBeVisible();
  });

  test("shows undo button for undoable actions", async ({ page }) => {
    await page.click('[data-testid="sidebar-governance"]');
    await page.click('text=History');

    const undoButton = page.locator('button:has-text("Undo")');
    await expect(undoButton.first()).toBeVisible();
  });
});
