//! `zorp run`: runs a collection from a workspace folder in the terminal or in CI, through
//! the same pipeline as the app. Nothing is stored; the folder is loaded into memory.
mod args;
mod report;

use std::path::Path;
use std::sync::Mutex;

use tokio::sync::oneshot;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::runner::RunnerOptions;
use crate::pipeline::runner::run_items;
use crate::services::cookies::CookieJar;
use crate::services::run_data;
use crate::services::secrets::MemoryStore;
use crate::services::workspace_sync::parse;
use crate::state::AppState;
use args::{Reporter, RunArgs};

/// Entry point of the zorp-cli binary.
pub fn main() -> i32 {
    try_run().unwrap_or_else(|| {
        eprintln!("{}", args::USAGE);
        2
    })
}

/// Runs the CLI when the first argument is `run`; returns the exit code, or `None` to start
/// the app normally.
pub fn try_run() -> Option<i32> {
    let mut argv = std::env::args().skip(1);
    if argv.next().as_deref() != Some("run") {
        return None;
    }
    attach_console();
    Some(match args::parse(argv) {
        Ok(run_args) => match runtime().and_then(|rt| rt.block_on(run(run_args))) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("zorp: {e}");
                2
            }
        },
        Err(message) => {
            eprintln!("{message}");
            2
        }
    })
}

fn runtime() -> AppResult<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(AppError::from)
}

/// Release builds of the app on Windows have no console of their own; reuse the one that
/// started us. The separate zorp-cli binary is a console program and needs nothing here.
#[cfg(all(windows, feature = "gui"))]
fn attach_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    // SAFETY: AttachConsole has no preconditions; failure only means output is not shown.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(all(windows, feature = "gui")))]
fn attach_console() {}

fn not_found(what: &str, name: &str, options: Vec<String>) -> AppError {
    AppError::Validation(format!(
        "{what} \"{name}\" tidak ada. Pilihan: {}",
        options.join(", ")
    ))
}

async fn run(a: RunArgs) -> AppResult<i32> {
    let snapshot = parse::read(Path::new(&a.folder)).map_err(|p| {
        AppError::Validation(format!("Folder tidak bisa dibaca:\n{}", p.0.join("\n")))
    })?;
    let conn = db::open_in_memory()?;
    let ws = db::workspace::current(&conn)?.id;
    let secrets = MemoryStore::default();
    db::snapshot::replace(&conn, ws, &snapshot, &secrets)?;

    let env_name = match &a.environment {
        Some(name) => {
            let envs = db::environments::list(&conn, ws)?;
            let env = envs
                .iter()
                .find(|e| e.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    not_found(
                        "Environment",
                        name,
                        envs.iter().map(|e| e.name.clone()).collect(),
                    )
                })?;
            db::environments::set_active(&conn, ws, Some(env.id))?;
            env.name.clone()
        }
        None => "tanpa environment".into(),
    };
    let collections = db::collections::list_collections(&conn, ws)?;
    let collection = match &a.collection {
        Some(name) => collections
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name)),
        None if collections.len() == 1 => collections.first(),
        None => None,
    }
    .ok_or_else(|| {
        not_found(
            "Koleksi",
            a.collection.as_deref().unwrap_or("?"),
            collections.iter().map(|c| c.name.clone()).collect(),
        )
    })?
    .clone();
    let folder_id = match &a.folder_name {
        Some(name) => {
            let folders: Vec<_> = db::folders::list_folders(&conn, ws)?
                .into_iter()
                .filter(|f| f.collection_id == collection.id)
                .collect();
            Some(
                folders
                    .iter()
                    .find(|f| f.name.eq_ignore_ascii_case(name))
                    .ok_or_else(|| {
                        not_found(
                            "Folder",
                            name,
                            folders.iter().map(|f| f.name.clone()).collect(),
                        )
                    })?
                    .id,
            )
        }
        None => None,
    };
    let items = db::run_order::ordered_requests(&conn, ws, collection.id, folder_id)?;
    if items.is_empty() {
        return Err(AppError::Validation(
            "Tidak ada request untuk dijalankan".into(),
        ));
    }

    // Secret values never live in the folder, so CI passes them in instead.
    let mut overrides: Vec<(String, String)> = std::env::vars()
        .filter_map(|(k, v)| {
            k.strip_prefix("ZORP_VAR_")
                .map(|name| (name.to_string(), v))
        })
        .collect();
    overrides.extend(a.vars.iter().cloned());
    let mut data = match &a.data {
        Some(path) => run_data::load(path)?.rows,
        None => Vec::new(),
    };
    if data.is_empty() && !overrides.is_empty() {
        data.push(Vec::new());
    }
    for row in &mut data {
        row.extend(overrides.iter().cloned());
    }

    let state = AppState::new(
        conn,
        ws,
        Box::new(secrets),
        std::env::temp_dir().join("zorp-cli"),
        CookieJar::default(),
        Default::default(),
    );
    let iterations = data.len().max(1) as u32;
    if a.reporter == Reporter::Text || a.out.is_some() {
        println!(
            "Zorp CLI {} · koleksi {} · {env_name} · {iterations} iterasi\n",
            env!("CARGO_PKG_VERSION"),
            collection.name
        );
    }
    let results = Mutex::new(Vec::new());
    let last = Mutex::new(0);
    let options = RunnerOptions {
        delay_ms: a.delay_ms,
        stop_on_failure: a.bail,
        data_path: None,
    };
    let (_keep, never) = oneshot::channel();
    let summary = run_items(&state, &items, &data, &options, never, |item| {
        if a.reporter == Reporter::Text || a.out.is_some() {
            let mut last = last.lock().unwrap_or_else(|e| e.into_inner());
            println!("{}", report::text_line(&item, &mut last, iterations));
        }
        results.lock().unwrap_or_else(|e| e.into_inner()).push(item);
    })
    .await;
    let results = results.into_inner().unwrap_or_else(|e| e.into_inner());
    let output = match a.reporter {
        Reporter::Text => None,
        Reporter::Json => Some(report::json(&collection.name, &summary, &results)),
        Reporter::Junit => Some(report::junit(&collection.name, &summary, &results)),
    };
    if a.reporter == Reporter::Text || a.out.is_some() {
        println!("{}", report::summary_line(&summary));
    }
    match (output, &a.out) {
        (Some(text), Some(path)) => {
            std::fs::write(path, text)?;
            println!("Laporan ditulis ke {path}");
        }
        (Some(text), None) => println!("{text}"),
        _ => {}
    }
    Ok(if summary.failed > 0 || summary.cancelled {
        1
    } else {
        0
    })
}

#[cfg(test)]
mod tests;
