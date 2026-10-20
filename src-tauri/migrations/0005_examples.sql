CREATE TABLE request_examples (
  id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL REFERENCES requests(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  status INTEGER NOT NULL,
  status_text TEXT NOT NULL,
  headers_json TEXT NOT NULL,
  body TEXT NOT NULL,
  sort_order REAL NOT NULL
);

CREATE INDEX idx_examples_request ON request_examples(request_id);
