use std::path::PathBuf;

use local_backlog::db;
use local_backlog::db::repo::project_repo;
use local_backlog::error::BacklogError;
use local_backlog::registry::{Registry, RegistryEntry};
use local_backlog::resolver;

fn register(
    conn: &rusqlite::Connection,
    registry: &mut Registry,
    name: &str,
    root: &std::path::Path,
) -> i64 {
    let project = project_repo::insert(conn, name, root.to_str().unwrap(), None).unwrap();
    registry.upsert(RegistryEntry {
        id: project.id,
        name: name.to_string(),
        root_path: root.to_path_buf(),
    });
    project.id
}

#[test]
fn resolves_simple_match() {
    let conn = db::open_in_memory().unwrap();
    let mut registry = Registry::default();

    let tmp = tempfile::tempdir().unwrap();
    let canon_root = std::fs::canonicalize(tmp.path()).unwrap();
    let id = register(&conn, &mut registry, "proj-a", &canon_root);

    let cwd = canon_root.clone();
    let out = resolver::resolve(&cwd, &conn, &registry).unwrap();
    assert_eq!(out.project_id, id);
    assert_eq!(out.name, "proj-a");
}

#[test]
fn resolves_ancestor_from_nested_cwd() {
    let conn = db::open_in_memory().unwrap();
    let mut registry = Registry::default();

    let tmp = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(tmp.path()).unwrap();
    let nested = root.join("src/lib");
    std::fs::create_dir_all(&nested).unwrap();
    register(&conn, &mut registry, "proj-a", &root);

    let canon_cwd = std::fs::canonicalize(&nested).unwrap();
    let out = resolver::resolve(&canon_cwd, &conn, &registry).unwrap();
    assert_eq!(out.name, "proj-a");
}

#[test]
fn greater_depth_wins_on_ambiguity() {
    let conn = db::open_in_memory().unwrap();
    let mut registry = Registry::default();

    let tmp = tempfile::tempdir().unwrap();
    let mono = std::fs::canonicalize(tmp.path()).unwrap();
    let pkg_api = mono.join("packages/api");
    let pkg_api_src = pkg_api.join("src");
    std::fs::create_dir_all(&pkg_api_src).unwrap();

    register(&conn, &mut registry, "mono", &mono);
    register(&conn, &mut registry, "api", &pkg_api);

    let out = resolver::resolve(&pkg_api_src, &conn, &registry).unwrap();
    assert_eq!(out.name, "api", "match de maior profundidade deve vencer");
}

#[test]
fn canonicalizes_cwd_through_symlink() {
    use std::os::unix::fs::symlink;

    let conn = db::open_in_memory().unwrap();
    let mut registry = Registry::default();

    let tmp = tempfile::tempdir().unwrap();
    let real = std::fs::canonicalize(tmp.path()).unwrap().join("real");
    std::fs::create_dir_all(&real).unwrap();
    let link = std::fs::canonicalize(tmp.path()).unwrap().join("link");
    symlink(&real, &link).unwrap();

    register(&conn, &mut registry, "proj-a", &real);

    let out = resolver::resolve(&link, &conn, &registry).unwrap();
    assert_eq!(out.name, "proj-a");
}

#[test]
fn errors_when_not_registered() {
    let conn = db::open_in_memory().unwrap();
    let registry = Registry::default();

    let tmp = tempfile::tempdir().unwrap();
    let cwd = std::fs::canonicalize(tmp.path()).unwrap();

    match resolver::resolve(&cwd, &conn, &registry).unwrap_err() {
        BacklogError::ProjectNotRegistered { .. } => {}
        e => panic!("variante errada: {e:?}"),
    }
}

#[test]
fn errors_when_project_archived() {
    let conn = db::open_in_memory().unwrap();
    let mut registry = Registry::default();

    let tmp = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(tmp.path()).unwrap();
    let id = register(&conn, &mut registry, "proj-a", &root);
    project_repo::archive(&conn, id).unwrap();

    match resolver::resolve(&root, &conn, &registry).unwrap_err() {
        BacklogError::ProjectArchived { id: got, name } => {
            assert_eq!(got, id);
            assert_eq!(name, "proj-a");
        }
        e => panic!("variante errada: {e:?}"),
    }
}

#[test]
fn registry_roundtrips_atomically() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("registry.toml");

    let mut reg = Registry::default();
    reg.upsert(RegistryEntry {
        id: 1,
        name: "proj-a".to_string(),
        root_path: PathBuf::from("/tmp/proj-a"),
    });
    reg.save_atomic(&path).unwrap();

    let loaded = Registry::load(&path).unwrap();
    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(loaded.entries[0].name, "proj-a");
}

#[test]
fn registry_load_missing_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("no-such.toml");
    let reg = Registry::load(&path).unwrap();
    assert!(reg.entries.is_empty());
}

#[test]
fn registry_load_corrupt_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("registry.toml");
    std::fs::write(&path, "not valid toml ===").unwrap();
    match Registry::load(&path).unwrap_err() {
        BacklogError::RegistryCorrupt { .. } => {}
        e => panic!("variante errada: {e:?}"),
    }
}
