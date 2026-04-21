CREATE TABLE projects (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT NOT NULL,
    root_path    TEXT NOT NULL UNIQUE,
    description  TEXT,
    archived_at  TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE tasks (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id    INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    body          TEXT,
    status        TEXT NOT NULL,
    priority      INTEGER,
    type          TEXT,
    parent_id     INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    archived_at   TEXT,
    completed_at  TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_tasks_project_status ON tasks(project_id, status) WHERE archived_at IS NULL;
CREATE INDEX idx_tasks_parent ON tasks(parent_id);

CREATE TRIGGER tasks_parent_same_project_insert
BEFORE INSERT ON tasks
WHEN NEW.parent_id IS NOT NULL
BEGIN
    SELECT CASE WHEN (
        NEW.project_id <> (SELECT project_id FROM tasks WHERE id = NEW.parent_id)
    ) THEN RAISE(ABORT, 'parent e child de projetos diferentes') END;
END;

CREATE TRIGGER tasks_parent_same_project_update
BEFORE UPDATE OF project_id, parent_id ON tasks
WHEN NEW.parent_id IS NOT NULL
BEGIN
    SELECT CASE WHEN (
        NEW.project_id <> (SELECT project_id FROM tasks WHERE id = NEW.parent_id)
    ) THEN RAISE(ABORT, 'parent e child de projetos diferentes') END;
END;

CREATE TABLE tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    UNIQUE (project_id, name)
);

CREATE TABLE task_tags (
    task_id  INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id   INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

CREATE TRIGGER task_tags_same_project_insert
BEFORE INSERT ON task_tags
BEGIN
    SELECT CASE WHEN (
        (SELECT project_id FROM tasks WHERE id = NEW.task_id)
        <> (SELECT project_id FROM tags WHERE id = NEW.tag_id)
    ) THEN RAISE(ABORT, 'tag e task de projetos diferentes') END;
END;

CREATE TRIGGER task_tags_same_project_update
BEFORE UPDATE OF task_id, tag_id ON task_tags
BEGIN
    SELECT CASE WHEN (
        (SELECT project_id FROM tasks WHERE id = NEW.task_id)
        <> (SELECT project_id FROM tags WHERE id = NEW.tag_id)
    ) THEN RAISE(ABORT, 'tag e task de projetos diferentes') END;
END;

CREATE TABLE task_attributes (
    task_id  INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    key      TEXT NOT NULL,
    value    TEXT NOT NULL,
    PRIMARY KEY (task_id, key)
);

CREATE TABLE task_links (
    from_id  INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    to_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    kind     TEXT NOT NULL,
    PRIMARY KEY (from_id, to_id, kind)
);

CREATE TRIGGER task_links_same_project_insert
BEFORE INSERT ON task_links
BEGIN
    SELECT CASE WHEN (
        (SELECT project_id FROM tasks WHERE id = NEW.from_id)
        <> (SELECT project_id FROM tasks WHERE id = NEW.to_id)
    ) THEN RAISE(ABORT, 'links entre projetos não são permitidos') END;
END;

CREATE TRIGGER task_links_same_project_update
BEFORE UPDATE OF from_id, to_id ON task_links
BEGIN
    SELECT CASE WHEN (
        (SELECT project_id FROM tasks WHERE id = NEW.from_id)
        <> (SELECT project_id FROM tasks WHERE id = NEW.to_id)
    ) THEN RAISE(ABORT, 'links entre projetos não são permitidos') END;
END;

CREATE TABLE task_events (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    ts         TEXT NOT NULL DEFAULT (datetime('now')),
    kind       TEXT NOT NULL,
    payload    TEXT
);

CREATE INDEX idx_task_events_task_ts ON task_events(task_id, ts);
