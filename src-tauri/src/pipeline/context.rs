use std::collections::HashMap;

use uuid::Uuid;

use crate::db;
use crate::error::AppResult;
use crate::models::request::{Auth, RequestSpec};
use crate::services::resolver::Variables;
use crate::state::AppState;

/// Where a request lives and the variable layers that apply to it.
pub struct RequestContext {
    pub collection_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    /// Workspace-wide values, the lowest layer.
    pub globals: Vec<(String, String)>,
    pub environment: Vec<(String, String)>,
    pub collection: Vec<(String, String)>,
    /// Values of the current runner iteration; they override the environment.
    pub data: Vec<(String, String)>,
    pub has_environment: bool,
}

impl RequestContext {
    pub fn load(state: &AppState, spec: &RequestSpec) -> AppResult<Self> {
        let location = db::requests::location(&*state.db()?, spec.id)?;
        Self::for_location(
            state,
            location.map(|(c, _)| c),
            location.and_then(|(_, f)| f),
        )
    }

    pub fn for_location(
        state: &AppState,
        collection_id: Option<Uuid>,
        folder_id: Option<Uuid>,
    ) -> AppResult<Self> {
        let conn = state.db()?;
        let environment =
            db::environments::active_values(&conn, state.workspace_id(), state.secrets.as_ref())?;
        let globals = db::globals::values(&conn, state.workspace_id(), state.secrets.as_ref())?;
        let has_environment = db::environments::list(&conn, state.workspace_id())?
            .iter()
            .any(|e| e.is_active);
        let collection = match collection_id {
            Some(id) => db::collections::variables(&conn, id)?,
            None => Vec::new(),
        };
        Ok(Self {
            collection_id,
            folder_id,
            globals,
            environment,
            collection,
            data: Vec::new(),
            has_environment,
        })
    }

    /// Reloads values from storage after a script changed them, keeping the iteration data.
    pub fn reload(&mut self, state: &AppState) -> AppResult<()> {
        let data = std::mem::take(&mut self.data);
        *self = Self::for_location(state, self.collection_id, self.folder_id)?;
        self.data = data;
        Ok(())
    }

    /// Globals, overridden by collection values, the environment, iteration data and
    /// script locals, in that order (Postman precedence).
    pub fn variables(&self, locals: &HashMap<String, String>) -> Variables {
        let locals: Vec<(String, String)> =
            locals.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        Variables::from_layers(vec![
            self.globals.clone(),
            self.collection.clone(),
            self.environment.clone(),
            self.data.clone(),
            locals,
        ])
    }

    pub fn variable_map(&self, locals: &HashMap<String, String>) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = self.globals.iter().cloned().collect();
        map.extend(self.collection.iter().cloned());
        map.extend(self.environment.iter().cloned());
        map.extend(self.data.iter().cloned());
        map.extend(locals.clone());
        map
    }

    pub fn effective_auth(&self, state: &AppState, spec: &RequestSpec) -> AppResult<Auth> {
        Ok(match (&spec.auth, self.collection_id) {
            (Auth::Inherit, Some(collection_id)) => {
                db::collections::inherited_auth(&*state.db()?, collection_id, self.folder_id)?
                    .unwrap_or(Auth::None)
            }
            (Auth::Inherit, None) => Auth::None,
            (other, _) => other.clone(),
        })
    }
}
