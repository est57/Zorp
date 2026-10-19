CREATE TABLE cookie_jars (
  workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
  jar_json TEXT NOT NULL
);
