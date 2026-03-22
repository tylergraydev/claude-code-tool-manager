use crate::db::models::{CreateDockerHostRequest, DockerHost};
use crate::db::Database;
use crate::services::docker::client::DockerClientManager;
use log::{error, info};
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::State;

// ============================================================================
// Row mapper
// ============================================================================

fn row_to_docker_host(row: &rusqlite::Row) -> rusqlite::Result<DockerHost> {
    Ok(DockerHost {
        id: row.get(0)?,
        name: row.get(1)?,
        host_type: row.get(2)?,
        connection_uri: row.get(3)?,
        ssh_key_path: row.get(4)?,
        tls_ca_cert: row.get(5)?,
        tls_cert: row.get(6)?,
        tls_key: row.get(7)?,
        is_default: row.get::<_, i32>(8)? != 0,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

// ============================================================================
// Tauri command wrappers
// ============================================================================

#[tauri::command]
pub fn get_all_docker_hosts(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<DockerHost>, String> {
    info!("[DockerHost] Loading all docker hosts");
    let db = db.lock().map_err(|e| e.to_string())?;
    get_all_docker_hosts_impl(&db)
}

#[tauri::command]
pub fn create_docker_host(
    db: State<'_, Arc<Mutex<Database>>>,
    host: CreateDockerHostRequest,
) -> Result<DockerHost, String> {
    info!("[DockerHost] Creating docker host: {}", host.name);
    let db = db.lock().map_err(|e| e.to_string())?;
    let result = create_docker_host_impl(&db, &host);
    if let Err(ref e) = result {
        error!(
            "[DockerHost] Failed to create docker host '{}': {}",
            host.name, e
        );
    }
    result
}

#[tauri::command]
pub fn update_docker_host(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    host: CreateDockerHostRequest,
) -> Result<DockerHost, String> {
    info!("[DockerHost] Updating docker host id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    update_docker_host_impl(&db, id, &host)
}

#[tauri::command]
pub fn delete_docker_host(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    if id == 1 {
        return Err("Cannot delete the default local Docker host".to_string());
    }
    info!("[DockerHost] Deleting docker host id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    delete_docker_host_impl(&db, id)
}

#[tauri::command]
pub async fn test_docker_host(
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    host_type: String,
    connection_uri: String,
    ssh_key_path: String,
) -> Result<bool, String> {
    info!(
        "[DockerHost] Testing connection: type={}, uri={}",
        host_type, connection_uri
    );
    docker_mgr
        .ping_host(
            &host_type,
            if connection_uri.is_empty() {
                None
            } else {
                Some(&connection_uri)
            },
            if ssh_key_path.is_empty() {
                None
            } else {
                Some(&ssh_key_path)
            },
        )
        .await
}

// ============================================================================
// Business logic implementations
// ============================================================================

pub(crate) fn get_all_docker_hosts_impl(db: &Database) -> Result<Vec<DockerHost>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, host_type, connection_uri, ssh_key_path,
                    tls_ca_cert, tls_cert, tls_key, is_default, created_at, updated_at
             FROM docker_hosts ORDER BY is_default DESC, name",
        )
        .map_err(|e| e.to_string())?;

    let hosts: Vec<DockerHost> = stmt
        .query_map([], row_to_docker_host)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hosts)
}

pub(crate) fn get_docker_host_impl(db: &Database, id: i64) -> Result<DockerHost, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, host_type, connection_uri, ssh_key_path,
                    tls_ca_cert, tls_cert, tls_key, is_default, created_at, updated_at
             FROM docker_hosts WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_docker_host)
        .map_err(|e| format!("Docker host not found: {}", e))
}

fn create_docker_host_impl(
    db: &Database,
    req: &CreateDockerHostRequest,
) -> Result<DockerHost, String> {
    let is_default = req.is_default.unwrap_or(false);

    // If setting as default, clear other defaults first
    if is_default {
        db.conn()
            .execute("UPDATE docker_hosts SET is_default = 0", [])
            .map_err(|e| e.to_string())?;
    }

    db.conn()
        .execute(
            "INSERT INTO docker_hosts (name, host_type, connection_uri, ssh_key_path,
             tls_ca_cert, tls_cert, tls_key, is_default)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                req.name,
                req.host_type,
                req.connection_uri,
                req.ssh_key_path,
                req.tls_ca_cert,
                req.tls_cert,
                req.tls_key,
                is_default as i32,
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_docker_host_impl(db, id)
}

fn update_docker_host_impl(
    db: &Database,
    id: i64,
    req: &CreateDockerHostRequest,
) -> Result<DockerHost, String> {
    let is_default = req.is_default.unwrap_or(false);

    if is_default {
        db.conn()
            .execute("UPDATE docker_hosts SET is_default = 0", [])
            .map_err(|e| e.to_string())?;
    }

    db.conn()
        .execute(
            "UPDATE docker_hosts SET name = ?, host_type = ?, connection_uri = ?,
             ssh_key_path = ?, tls_ca_cert = ?, tls_cert = ?, tls_key = ?,
             is_default = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![
                req.name,
                req.host_type,
                req.connection_uri,
                req.ssh_key_path,
                req.tls_ca_cert,
                req.tls_cert,
                req.tls_key,
                is_default as i32,
                id,
            ],
        )
        .map_err(|e| e.to_string())?;

    get_docker_host_impl(db, id)
}

fn delete_docker_host_impl(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM docker_hosts WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
