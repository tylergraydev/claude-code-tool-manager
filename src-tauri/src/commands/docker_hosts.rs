use crate::db::models::{CreateDockerHostRequest, DockerHost};
use crate::db::Database;
use std::sync::{Arc, Mutex};
use tauri::State;

#[tauri::command]
pub fn get_all_docker_hosts(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<DockerHost>, String> {
    let _db = db.lock().map_err(|e| e.to_string())?;
    Ok(vec![])
}

#[tauri::command]
pub fn create_docker_host(
    db: State<'_, Arc<Mutex<Database>>>,
    request: CreateDockerHostRequest,
) -> Result<DockerHost, String> {
    let _db = db.lock().map_err(|e| e.to_string())?;
    Err("Docker host feature not yet implemented".to_string())
}

#[tauri::command]
pub fn update_docker_host(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    request: CreateDockerHostRequest,
) -> Result<DockerHost, String> {
    let _db = db.lock().map_err(|e| e.to_string())?;
    Err("Docker host feature not yet implemented".to_string())
}

#[tauri::command]
pub fn delete_docker_host(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    let _db = db.lock().map_err(|e| e.to_string())?;
    Err("Docker host feature not yet implemented".to_string())
}

#[tauri::command]
pub fn test_docker_host(id: i64) -> Result<bool, String> {
    Err("Docker host feature not yet implemented".to_string())
}
