ALTER TABLE workspaces ADD COLUMN folder_path TEXT;
ALTER TABLE workspaces ADD COLUMN last_opened_at INTEGER NOT NULL DEFAULT 0;
