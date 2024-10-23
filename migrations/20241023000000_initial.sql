CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    category_id INTEGER,
    log_channel_id INTEGER
);

INSERT OR IGNORE INTO config (key) VALUES ('main');
