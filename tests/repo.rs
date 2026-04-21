use local_backlog::config::PriorityOrder;
use local_backlog::db;
use local_backlog::db::repo::{project_repo, tag_repo, task_repo};

fn fresh() -> (rusqlite::Connection, i64) {
    let conn = db::open_in_memory().unwrap();
    let project = project_repo::insert(&conn, "proj", "/tmp/proj", None).unwrap();
    (conn, project.id)
}

fn new_task(title: &str, priority: i64) -> task_repo::NewTask {
    task_repo::NewTask {
        title: title.to_string(),
        body: None,
        status: "todo".to_string(),
        priority,
        task_type: None,
        parent_id: None,
    }
}

#[test]
fn task_insert_and_get_are_tenant_scoped() {
    let (conn, pid) = fresh();
    let other = project_repo::insert(&conn, "other", "/tmp/other", None).unwrap();

    let t = task_repo::insert(&conn, pid, &new_task("first", 100)).unwrap();

    assert!(task_repo::get(&conn, pid, t.id).unwrap().is_some());
    // tenant-leak policy: outra tenant vê None (não TaskNotFound diferenciado).
    assert!(task_repo::get(&conn, other.id, t.id).unwrap().is_none());
}

#[test]
fn task_list_applies_filters_and_order() {
    let (conn, pid) = fresh();
    let hi = task_repo::insert(&conn, pid, &new_task("a", 50)).unwrap();
    let lo = task_repo::insert(&conn, pid, &new_task("b", 200)).unwrap();

    let asc = task_repo::list(
        &conn,
        pid,
        &task_repo::ListFilter {
            priority_order: Some(PriorityOrder::Asc),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(asc[0].id, hi.id, "ASC → menor priority primeiro");
    assert_eq!(asc[1].id, lo.id);

    let desc = task_repo::list(
        &conn,
        pid,
        &task_repo::ListFilter {
            priority_order: Some(PriorityOrder::Desc),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(desc[0].id, lo.id);
}

#[test]
fn task_list_excludes_archived_by_default() {
    let (conn, pid) = fresh();
    let keep = task_repo::insert(&conn, pid, &new_task("a", 100)).unwrap();
    let gone = task_repo::insert(&conn, pid, &new_task("b", 100)).unwrap();
    task_repo::set_archived(&conn, pid, gone.id).unwrap();

    let default = task_repo::list(&conn, pid, &task_repo::ListFilter::default()).unwrap();
    assert_eq!(default.len(), 1);
    assert_eq!(default[0].id, keep.id);

    let all = task_repo::list(
        &conn,
        pid,
        &task_repo::ListFilter {
            include_archived: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn set_status_is_idempotent_and_signals_change() {
    let (conn, pid) = fresh();
    let t = task_repo::insert(&conn, pid, &new_task("a", 100)).unwrap();

    let (_, changed) = task_repo::set_status(&conn, pid, t.id, "done", true).unwrap();
    assert!(changed);
    let (t2, changed2) = task_repo::set_status(&conn, pid, t.id, "done", true).unwrap();
    assert!(!changed2);
    assert!(t2.completed_at.is_some());
}

#[test]
fn set_archived_is_idempotent() {
    let (conn, pid) = fresh();
    let t = task_repo::insert(&conn, pid, &new_task("a", 100)).unwrap();
    assert!(task_repo::set_archived(&conn, pid, t.id).unwrap());
    assert!(!task_repo::set_archived(&conn, pid, t.id).unwrap());
}

#[test]
fn tag_ensure_is_idempotent_and_scoped() {
    let (conn, pid) = fresh();
    let other = project_repo::insert(&conn, "other", "/tmp/other", None).unwrap();

    let t1 = tag_repo::ensure(&conn, pid, "urgent").unwrap();
    let t2 = tag_repo::ensure(&conn, pid, "urgent").unwrap();
    assert_eq!(t1.id, t2.id);

    // mesma tag em outro tenant → id distinto
    let t_other = tag_repo::ensure(&conn, other.id, "urgent").unwrap();
    assert_ne!(t1.id, t_other.id);
}

#[test]
fn tag_attach_detach_list() {
    let (conn, pid) = fresh();
    let task = task_repo::insert(&conn, pid, &new_task("a", 100)).unwrap();
    let tag = tag_repo::ensure(&conn, pid, "x").unwrap();

    tag_repo::attach(&conn, pid, task.id, tag.id).unwrap();
    // idempotência (INSERT OR IGNORE)
    tag_repo::attach(&conn, pid, task.id, tag.id).unwrap();
    assert_eq!(
        tag_repo::list_for_task(&conn, pid, task.id).unwrap().len(),
        1
    );

    tag_repo::detach(&conn, pid, task.id, tag.id).unwrap();
    assert!(tag_repo::list_for_task(&conn, pid, task.id)
        .unwrap()
        .is_empty());
}
