use local_backlog::db;

#[test]
fn schema_after_migrations_matches_snapshot() {
    let conn = db::open_in_memory().expect("migrations aplicadas com sucesso");

    let mut stmt = conn
        .prepare(
            "SELECT type, name, sql FROM sqlite_master \
             WHERE name NOT LIKE 'sqlite_%' \
             ORDER BY type, name",
        )
        .unwrap();

    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ))
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    let rendered = rows
        .into_iter()
        .map(|(kind, name, sql)| format!("-- {kind} {name}\n{sql}"))
        .collect::<Vec<_>>()
        .join("\n\n");

    insta::assert_snapshot!("schema", rendered);
}
