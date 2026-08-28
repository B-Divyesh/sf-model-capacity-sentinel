PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS probes (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  provider TEXT NOT NULL,
  endpoint_url TEXT NOT NULL,
  model TEXT NOT NULL,
  api_key_cipher TEXT NOT NULL,
  prompt TEXT NOT NULL,
  required_fields TEXT NOT NULL DEFAULT '[]',
  interval_minutes INTEGER NOT NULL DEFAULT 5,
  timeout_ms INTEGER NOT NULL DEFAULT 15000,
  latency_slo_ms INTEGER NOT NULL DEFAULT 3000,
  availability_slo_percent REAL NOT NULL DEFAULT 99,
  max_output_tokens INTEGER NOT NULL DEFAULT 128,
  daily_token_cap INTEGER NOT NULL DEFAULT 5000,
  enabled INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS observations (
  id TEXT PRIMARY KEY,
  probe_id TEXT NOT NULL REFERENCES probes(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL,
  latency_ms INTEGER NOT NULL,
  http_status INTEGER,
  outcome TEXT NOT NULL,
  error_class TEXT,
  detail TEXT,
  input_tokens INTEGER,
  output_tokens INTEGER,
  invariant_valid INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_observations_probe_time ON observations(probe_id, started_at DESC);

CREATE TABLE IF NOT EXISTS alerts (
  id TEXT PRIMARY KEY,
  probe_id TEXT NOT NULL REFERENCES probes(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'open',
  message TEXT NOT NULL,
  opened_at TEXT NOT NULL,
  resolved_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_alerts_probe_status ON alerts(probe_id, status, opened_at DESC);
