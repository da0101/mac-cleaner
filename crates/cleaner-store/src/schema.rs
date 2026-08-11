pub(crate) const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    plan_json TEXT NOT NULL,
    approval_id TEXT,
    approved_at TEXT,
    updated_at TEXT NOT NULL,
    error TEXT
);
CREATE TABLE IF NOT EXISTS plan_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id TEXT NOT NULL,
    state TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    detail TEXT,
    FOREIGN KEY(plan_id) REFERENCES plans(id)
);
CREATE TABLE IF NOT EXISTS receipts (
    plan_id TEXT PRIMARY KEY,
    completed_at TEXT NOT NULL,
    receipt_json TEXT NOT NULL,
    FOREIGN KEY(plan_id) REFERENCES plans(id)
);
CREATE TABLE IF NOT EXISTS quarantine_entries (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL,
    state TEXT NOT NULL,
    entry_json TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(plan_id) REFERENCES plans(id)
);
CREATE TABLE IF NOT EXISTS quarantine_actions (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL,
    state TEXT NOT NULL,
    action_json TEXT NOT NULL,
    approval_id TEXT,
    approved_at TEXT,
    updated_at TEXT NOT NULL,
    error TEXT,
    FOREIGN KEY(entry_id) REFERENCES quarantine_entries(id)
);
CREATE TABLE IF NOT EXISTS quarantine_action_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    action_id TEXT NOT NULL,
    state TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    detail TEXT,
    FOREIGN KEY(action_id) REFERENCES quarantine_actions(id)
);
CREATE TABLE IF NOT EXISTS tool_cleanup_plans (
    id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    plan_json TEXT NOT NULL,
    approval_id TEXT,
    approved_at TEXT,
    updated_at TEXT NOT NULL,
    error TEXT
);
CREATE TABLE IF NOT EXISTS tool_cleanup_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id TEXT NOT NULL,
    state TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    detail TEXT,
    FOREIGN KEY(plan_id) REFERENCES tool_cleanup_plans(id)
);
CREATE TABLE IF NOT EXISTS tool_cleanup_receipts (
    plan_id TEXT PRIMARY KEY,
    completed_at TEXT NOT NULL,
    receipt_json TEXT NOT NULL,
    FOREIGN KEY(plan_id) REFERENCES tool_cleanup_plans(id)
);
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS imports (
    source_fingerprint TEXT PRIMARY KEY,
    imported_at TEXT NOT NULL,
    summary_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS legacy_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_fingerprint TEXT NOT NULL,
    occurred_at TEXT,
    summary_json TEXT NOT NULL
);
";
