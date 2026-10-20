use super::*;
use crate::db::open_in_memory;
use crate::models::environment::Environment;
use crate::services::secrets::MemoryStore;

#[test]
fn replace_keeps_local_secrets_and_active_environment() {
    let conn = open_in_memory().unwrap();
    let store = MemoryStore::default();
    let ws = workspace::current(&conn).unwrap().id;
    let c = collections::create_collection(&conn, ws, "API").unwrap();
    let f = folders::create_folder(&conn, c.id, None, "Admin").unwrap();
    folders::create_folder(&conn, c.id, Some(f.id), "Deep").unwrap();
    let secret = Variable {
        key: "token".into(),
        value: "s3cret".into(),
        enabled: true,
        secret: true,
        has_value: false,
    };
    let env = Environment {
        id: Uuid::now_v7(),
        name: "Dev".into(),
        variables: vec![secret],
        is_active: false,
    };
    let env = environments::save(&conn, ws, env, &store).unwrap();
    environments::set_active(&conn, ws, Some(env.id)).unwrap();

    let mut snapshot = load(&conn, ws).unwrap();
    snapshot.folders.reverse();
    snapshot.name = "Renamed".into();
    replace(&conn, ws, &snapshot, &store).unwrap();
    let after = load(&conn, ws).unwrap();
    assert_eq!(after.name, "Renamed");
    assert_eq!(after.folders.len(), 2);
    assert!(after.environments[0].is_active && after.environments[0].variables[0].has_value);
    assert_eq!(
        store.get(&env_account(env.id, "token")).unwrap().as_deref(),
        Some("s3cret")
    );

    snapshot.environments.clear();
    replace(&conn, ws, &snapshot, &store).unwrap();
    assert!(store.get(&env_account(env.id, "token")).unwrap().is_none());

    let other = workspace::create(&conn, "Other", None).unwrap().id;
    assert!(matches!(
        replace(&conn, other, &snapshot, &store),
        Err(AppError::Validation(_))
    ));
}

#[test]
fn owner_of_finds_the_workspace_that_holds_an_id() {
    let conn = open_in_memory().unwrap();
    let ws = workspace::current(&conn).unwrap().id;
    let c = collections::create_collection(&conn, ws, "API").unwrap();
    let found = workspace::owner_of(&conn, &[Uuid::now_v7(), c.id]).unwrap();
    assert_eq!(found.map(|w| w.id), Some(ws));
    assert!(
        workspace::owner_of(&conn, &[Uuid::now_v7()])
            .unwrap()
            .is_none()
    );
}
