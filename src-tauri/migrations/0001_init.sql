CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE collections (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  auth_json TEXT,
  variables_json TEXT,
  sort_order REAL NOT NULL
);

CREATE TABLE folders (
  id TEXT PRIMARY KEY,
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  parent_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  auth_json TEXT,
  sort_order REAL NOT NULL
);

CREATE TABLE requests (
  id TEXT PRIMARY KEY,
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  folder_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
  spec_json TEXT NOT NULL,
  sort_order REAL NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE environments (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  variables_json TEXT NOT NULL,
  is_active INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE history (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  request_json TEXT NOT NULL,
  status INTEGER,
  duration_ms INTEGER,
  size_bytes INTEGER,
  sent_at INTEGER NOT NULL
);

CREATE INDEX idx_collections_workspace ON collections(workspace_id);
CREATE INDEX idx_folders_collection ON folders(collection_id);
CREATE INDEX idx_requests_collection ON requests(collection_id);
CREATE INDEX idx_environments_workspace ON environments(workspace_id);
CREATE INDEX idx_history_sent ON history(workspace_id, sent_at DESC);
