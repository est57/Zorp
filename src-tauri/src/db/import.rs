use rusqlite::{Connection, params};
use uuid::Uuid;

use super::{collections, folders, requests};
use crate::error::AppResult;
use crate::models::collection::NodeKind;
use crate::models::import::ImportReport;
use crate::services::importers::{ImportedCollection, ImportedNode};

fn insert_nodes(
    conn: &Connection,
    collection_id: Uuid,
    parent: Option<Uuid>,
    nodes: Vec<ImportedNode>,
    report: &mut ImportReport,
) -> AppResult<()> {
    for node in nodes {
        match node {
            ImportedNode::Folder {
                name,
                auth,
                children,
            } => {
                let folder = folders::create_folder(conn, collection_id, parent, &name)?;
                if let Some(auth) = auth {
                    collections::set_auth(conn, NodeKind::Folder, folder.id, &auth)?;
                }
                report.folders += 1;
                insert_nodes(conn, collection_id, Some(folder.id), children, report)?;
            }
            ImportedNode::Request(spec) => {
                requests::save(conn, *spec, collection_id, parent)?;
                report.requests += 1;
            }
        }
    }
    Ok(())
}

/// Stores an imported collection atomically: either everything is saved or nothing is.
pub fn save(
    conn: &Connection,
    workspace_id: Uuid,
    imported: ImportedCollection,
) -> AppResult<ImportReport> {
    let tx = conn.unchecked_transaction()?;
    let collection = collections::create_collection(&tx, workspace_id, &imported.name)?;
    if let Some(auth) = &imported.auth {
        collections::set_auth(&tx, NodeKind::Collection, collection.id, auth)?;
    }
    if !imported.variables.is_empty() {
        tx.execute(
            "UPDATE collections SET variables_json = ?1 WHERE id = ?2",
            params![
                serde_json::to_string(&imported.variables)?,
                collection.id.to_string()
            ],
        )?;
    }
    let mut report = ImportReport {
        collection_id: collection.id,
        collection_name: collection.name,
        folders: 0,
        requests: 0,
        warnings: imported.warnings,
    };
    insert_nodes(&tx, collection.id, None, imported.items, &mut report)?;
    tx.commit()?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_in_memory, workspace};
    use crate::services::importers::postman;

    #[test]
    fn saves_imported_collection_with_variables() {
        let conn = open_in_memory().unwrap();
        let ws = workspace::current(&conn).unwrap().id;
        let json = r#"{"info":{"name":"A","schema":"https://schema.getpostman.com/json/collection/v2.1.0/collection.json"},
            "variable":[{"key":"host","value":"h.test"}],
            "item":[{"name":"F","item":[{"name":"R","request":{"method":"GET","url":"{{host}}/x"}}]}]}"#;
        let report = save(&conn, ws, postman::parse(json).unwrap()).unwrap();
        assert_eq!((report.folders, report.requests), (1, 1));
        let vars = collections::variables(&conn, report.collection_id).unwrap();
        assert_eq!(vars, vec![("host".to_string(), "h.test".to_string())]);
    }
}
