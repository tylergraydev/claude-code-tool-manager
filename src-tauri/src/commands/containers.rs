use crate::db::models::{
    Container, ContainerLog, ContainerStats, ContainerStatus, ContainerTemplate,
    ContainerWithStatus, CreateContainerRequest, ExecResult, PortMapping, ProjectContainer,
    VolumeMapping,
};
use crate::db::Database;
use crate::services::docker::client::DockerClientManager;
use log::{error, info};
use rusqlite::params;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

// ============================================================================
// JSON parsing helpers
// ============================================================================

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn parse_json_map(s: Option<String>) -> Option<HashMap<String, String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn parse_json_port_mappings(s: Option<String>) -> Option<Vec<PortMapping>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn parse_json_volume_mappings(s: Option<String>) -> Option<Vec<VolumeMapping>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

// ============================================================================
// Row mapper
// ============================================================================

fn row_to_container(row: &rusqlite::Row) -> rusqlite::Result<Container> {
    Ok(Container {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        container_type: row.get(3)?,
        docker_host_id: row.get(4)?,
        docker_container_id: row.get(5)?,
        image: row.get(6)?,
        dockerfile: row.get(7)?,
        devcontainer_json: row.get(8)?,
        env: parse_json_map(row.get(9)?),
        ports: parse_json_port_mappings(row.get(10)?),
        volumes: parse_json_volume_mappings(row.get(11)?),
        mounts: parse_json_array(row.get(12)?),
        features: parse_json_array(row.get(13)?),
        post_create_command: row.get(14)?,
        post_start_command: row.get(15)?,
        working_dir: row.get(16)?,
        template_id: row.get(17)?,
        repo_url: row.get(18)?,
        icon: row.get(19)?,
        tags: parse_json_array(row.get(20)?),
        is_favorite: row.get::<_, i32>(21)? != 0,
        created_at: row.get(22)?,
        updated_at: row.get(23)?,
    })
}

// ============================================================================
// Tauri command wrappers
// ============================================================================

#[tauri::command]
pub fn get_all_containers(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Container>, String> {
    info!("[Container] Loading all containers");
    let db = db.lock().map_err(|e| e.to_string())?;
    get_all_containers_impl(&db)
}

#[tauri::command]
pub fn get_container(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<Container, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    get_container_impl(&db, id)
}

#[tauri::command]
pub fn create_container(
    db: State<'_, Arc<Mutex<Database>>>,
    container: CreateContainerRequest,
) -> Result<Container, String> {
    info!("[Container] Creating container: {}", container.name);
    let db = db.lock().map_err(|e| e.to_string())?;
    let result = create_container_impl(&db, &container);
    if let Err(ref e) = result {
        error!(
            "[Container] Failed to create container '{}': {}",
            container.name, e
        );
    }
    result
}

#[tauri::command]
pub fn update_container(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    container: CreateContainerRequest,
) -> Result<Container, String> {
    info!("[Container] Updating container id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    update_container_impl(&db, id, &container)
}

#[tauri::command]
pub async fn delete_container(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<(), String> {
    info!("[Container] Deleting container id={}", id);
    // Try to remove the Docker container first
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    if let Some(ref docker_id) = container.docker_container_id {
        // Force remove (stop + remove) — ignore errors if container doesn't exist
        let _ = docker_mgr
            .stop_container(docker_id, container.docker_host_id)
            .await;
        let _ = docker_mgr
            .remove_container(docker_id, container.docker_host_id)
            .await;
    }
    let db = db.lock().map_err(|e| e.to_string())?;
    delete_container_impl(&db, id)
}

#[tauri::command]
pub fn toggle_container_favorite(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
) -> Result<Container, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    toggle_container_favorite_impl(&db, id)
}

#[tauri::command]
pub async fn check_docker_available(
    docker_mgr: State<'_, Arc<DockerClientManager>>,
) -> Result<bool, String> {
    Ok(docker_mgr.is_docker_available().await)
}

#[tauri::command]
pub async fn build_container_image(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<String, String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };

    let image = container
        .image
        .as_deref()
        .ok_or("No image specified for container")?;
    docker_mgr
        .pull_image(image, container.docker_host_id)
        .await?;
    Ok(format!("Pulled image: {}", image))
}

#[tauri::command]
pub async fn start_container_cmd(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<(), String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };

    // Load Claude settings for container creation
    let claude_settings = {
        let db = db.lock().map_err(|e| e.to_string())?;
        crate::commands::settings::get_container_claude_settings_from_db(&db)
    };

    // If no docker_container_id, pull image and create the container first
    let is_first_start;
    let docker_id = if let Some(ref did) = container.docker_container_id {
        is_first_start = false;
        did.clone()
    } else {
        is_first_start = true;
        // Pull the image if specified
        if let Some(ref image) = container.image {
            info!("[Container] Pulling image before first start: {}", image);
            docker_mgr
                .pull_image(image, container.docker_host_id)
                .await?;
        }

        let created_id = docker_mgr
            .create_docker_container(&container, Some(&claude_settings))
            .await?;
        // Save the docker_container_id back to DB
        let db = db.lock().map_err(|e| e.to_string())?;
        db.conn()
            .execute(
                "UPDATE containers SET docker_container_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                params![created_id, id],
            )
            .map_err(|e| e.to_string())?;
        created_id
    };

    docker_mgr
        .start_container(&docker_id, container.docker_host_id)
        .await?;

    // On first start, clone repo if repo_url is set
    if is_first_start {
        if let Some(ref repo_url) = container.repo_url {
            let working_dir = container.working_dir.as_deref().unwrap_or("/workspace");
            info!("[Container] Cloning repo {} into {}", repo_url, working_dir);
            let clone_cmd = vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("git clone {} {}", repo_url, working_dir),
            ];
            match docker_mgr
                .exec(&docker_id, container.docker_host_id, clone_cmd)
                .await
            {
                Ok(result) => {
                    if result.exit_code != 0 {
                        // If clone fails because dir isn't empty, try cloning into it
                        let retry_cmd = vec![
                            "sh".to_string(),
                            "-c".to_string(),
                            format!("cd {} && git init && git remote add origin {} && git fetch origin && git checkout -t origin/main || git checkout -t origin/master", working_dir, repo_url),
                        ];
                        let _ = docker_mgr
                            .exec(&docker_id, container.docker_host_id, retry_cmd)
                            .await;
                    }
                    // Run post_create_command if set
                    if let Some(ref post_cmd) = container.post_create_command {
                        info!("[Container] Running post-create command: {}", post_cmd);
                        let post_exec = vec![
                            "sh".to_string(),
                            "-c".to_string(),
                            format!("cd {} && {}", working_dir, post_cmd),
                        ];
                        let _ = docker_mgr
                            .exec(&docker_id, container.docker_host_id, post_exec)
                            .await;
                    }
                }
                Err(e) => {
                    error!("[Container] Failed to clone repo: {}", e);
                }
            }
        } else if let Some(ref post_cmd) = container.post_create_command {
            // No repo, but still run post_create_command
            let working_dir = container.working_dir.as_deref().unwrap_or("/workspace");
            info!("[Container] Running post-create command: {}", post_cmd);
            let post_exec = vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("cd {} && {}", working_dir, post_cmd),
            ];
            let _ = docker_mgr
                .exec(&docker_id, container.docker_host_id, post_exec)
                .await;
        }

        // Auto-install Claude Code if enabled
        if claude_settings.auto_install {
            info!("[Container] Auto-installing Claude Code");
            let install_cmd = vec![
                "sh".to_string(),
                "-c".to_string(),
                "npm install -g @anthropic-ai/claude-code 2>/dev/null || true".to_string(),
            ];
            let _ = docker_mgr
                .exec(&docker_id, container.docker_host_id, install_cmd)
                .await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_container_cmd(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<(), String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    let docker_id = container
        .docker_container_id
        .ok_or("Container has no Docker ID")?;
    docker_mgr
        .stop_container(&docker_id, container.docker_host_id)
        .await
}

#[tauri::command]
pub async fn restart_container_cmd(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<(), String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    let docker_id = container
        .docker_container_id
        .ok_or("Container has no Docker ID")?;
    docker_mgr
        .restart_container(&docker_id, container.docker_host_id)
        .await
}

#[tauri::command]
pub async fn remove_container_cmd(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<(), String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    if let Some(ref docker_id) = container.docker_container_id {
        docker_mgr
            .remove_container(docker_id, container.docker_host_id)
            .await?;
    }
    // Clear the docker_container_id in DB
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "UPDATE containers SET docker_container_id = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_container_status(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<ContainerStatus, String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };

    if let Some(ref docker_id) = container.docker_container_id {
        docker_mgr
            .inspect_container_status(id, docker_id, container.docker_host_id)
            .await
    } else {
        Ok(ContainerStatus {
            container_id: id,
            docker_status: "not_created".to_string(),
            docker_container_id: None,
            started_at: None,
            finished_at: None,
            exit_code: None,
            health: None,
            cpu_percent: None,
            memory_usage: None,
            memory_limit: None,
        })
    }
}

#[tauri::command]
pub async fn get_all_container_statuses(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
) -> Result<Vec<ContainerWithStatus>, String> {
    let containers = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_all_containers_impl(&db)?
    };

    let mut results = Vec::new();
    for container in containers {
        let status = if let Some(ref docker_id) = container.docker_container_id {
            docker_mgr
                .inspect_container_status(container.id, docker_id, container.docker_host_id)
                .await
                .unwrap_or(ContainerStatus {
                    container_id: container.id,
                    docker_status: "unknown".to_string(),
                    docker_container_id: container.docker_container_id.clone(),
                    started_at: None,
                    finished_at: None,
                    exit_code: None,
                    health: None,
                    cpu_percent: None,
                    memory_usage: None,
                    memory_limit: None,
                })
        } else {
            ContainerStatus {
                container_id: container.id,
                docker_status: "not_created".to_string(),
                docker_container_id: None,
                started_at: None,
                finished_at: None,
                exit_code: None,
                health: None,
                cpu_percent: None,
                memory_usage: None,
                memory_limit: None,
            }
        };
        results.push(ContainerWithStatus { container, status });
    }
    Ok(results)
}

#[tauri::command]
pub async fn get_container_logs_cmd(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
    tail: Option<u32>,
    since: Option<i64>,
) -> Result<Vec<ContainerLog>, String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    let docker_id = container
        .docker_container_id
        .ok_or("Container has no Docker ID")?;
    docker_mgr
        .get_logs(&docker_id, container.docker_host_id, tail, since)
        .await
}

#[tauri::command]
pub async fn get_container_stats_cmd(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
) -> Result<ContainerStats, String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    let docker_id = container
        .docker_container_id
        .ok_or("Container has no Docker ID")?;
    docker_mgr
        .get_stats(id, &docker_id, container.docker_host_id)
        .await
}

#[tauri::command]
pub async fn exec_in_container_cmd(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    id: i64,
    command: Vec<String>,
) -> Result<ExecResult, String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    let docker_id = container
        .docker_container_id
        .ok_or("Container has no Docker ID")?;
    docker_mgr
        .exec(&docker_id, container.docker_host_id, command)
        .await
}

#[tauri::command]
pub async fn start_container_shell(
    db: State<'_, Arc<Mutex<Database>>>,
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    app_handle: tauri::AppHandle,
    id: i64,
    session_id: String,
) -> Result<String, String> {
    let container = {
        let db = db.lock().map_err(|e| e.to_string())?;
        get_container_impl(&db, id)?
    };
    let docker_id = container
        .docker_container_id
        .ok_or("Container has no Docker ID")?;
    docker_mgr
        .start_interactive_shell(&docker_id, container.docker_host_id, app_handle, session_id)
        .await
}

#[tauri::command]
pub async fn send_shell_input(session_id: String, data: String) -> Result<(), String> {
    crate::services::docker::client::send_shell_input(&session_id, data).await
}

#[tauri::command]
pub async fn resize_shell(
    docker_mgr: State<'_, Arc<DockerClientManager>>,
    exec_id: String,
    host_id: i64,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    docker_mgr.resize_shell(&exec_id, host_id, rows, cols).await
}

#[tauri::command]
pub fn get_container_templates() -> Result<Vec<ContainerTemplate>, String> {
    Ok(get_builtin_templates())
}

#[tauri::command]
pub fn create_container_from_template(
    db: State<'_, Arc<Mutex<Database>>>,
    template_id: String,
    name: String,
) -> Result<Container, String> {
    let templates = get_builtin_templates();
    let template = templates
        .iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| format!("Template '{}' not found", template_id))?;

    let request = CreateContainerRequest {
        name,
        description: Some(template.description.clone()),
        container_type: "docker".to_string(),
        docker_host_id: None,
        image: Some(template.image.clone()),
        dockerfile: template.dockerfile.clone(),
        devcontainer_json: None,
        env: template.env.clone(),
        ports: template.ports.clone(),
        volumes: template.volumes.clone(),
        mounts: None,
        features: template.features.clone(),
        post_create_command: template.post_create_command.clone(),
        post_start_command: template.post_start_command.clone(),
        working_dir: template.working_dir.clone(),
        template_id: Some(template.id.clone()),
        repo_url: None,
        icon: Some(template.icon.clone()),
        tags: None,
    };

    let db = db.lock().map_err(|e| e.to_string())?;
    create_container_impl(&db, &request)
}

#[tauri::command]
pub fn assign_container_to_project(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    container_id: i64,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    assign_container_to_project_impl(&db, project_id, container_id)
}

#[tauri::command]
pub fn remove_container_from_project(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    container_id: i64,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    remove_container_from_project_impl(&db, project_id, container_id)
}

#[tauri::command]
pub fn get_project_containers(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
) -> Result<Vec<ProjectContainer>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    get_project_containers_impl(&db, project_id)
}

#[tauri::command]
pub fn set_default_project_container(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    container_id: i64,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    set_default_project_container_impl(&db, project_id, container_id)
}

// ============================================================================
// Business logic implementations
// ============================================================================

pub(crate) fn get_all_containers_impl(db: &Database) -> Result<Vec<Container>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, container_type, docker_host_id, docker_container_id,
                    image, dockerfile, devcontainer_json, env, ports, volumes, mounts, features,
                    post_create_command, post_start_command, working_dir, template_id, repo_url, icon, tags,
                    is_favorite, created_at, updated_at
             FROM containers ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let containers: Vec<Container> = stmt
        .query_map([], row_to_container)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(containers)
}

pub(crate) fn get_container_impl(db: &Database, id: i64) -> Result<Container, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, container_type, docker_host_id, docker_container_id,
                    image, dockerfile, devcontainer_json, env, ports, volumes, mounts, features,
                    post_create_command, post_start_command, working_dir, template_id, repo_url, icon, tags,
                    is_favorite, created_at, updated_at
             FROM containers WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_container)
        .map_err(|e| format!("Container not found: {}", e))
}

pub(crate) fn create_container_impl(
    db: &Database,
    req: &CreateContainerRequest,
) -> Result<Container, String> {
    let env_json = req.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
    let ports_json = req
        .ports
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap());
    let volumes_json = req
        .volumes
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap());
    let mounts_json = req
        .mounts
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap());
    let features_json = req
        .features
        .as_ref()
        .map(|f| serde_json::to_string(f).unwrap());
    let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let docker_host_id = req.docker_host_id.unwrap_or(1);

    db.conn()
        .execute(
            "INSERT INTO containers (name, description, container_type, docker_host_id, image,
             dockerfile, devcontainer_json, env, ports, volumes, mounts, features,
             post_create_command, post_start_command, working_dir, template_id, repo_url, icon, tags)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                req.name,
                req.description,
                req.container_type,
                docker_host_id,
                req.image,
                req.dockerfile,
                req.devcontainer_json,
                env_json,
                ports_json,
                volumes_json,
                mounts_json,
                features_json,
                req.post_create_command,
                req.post_start_command,
                req.working_dir,
                req.template_id,
                req.repo_url,
                req.icon,
                tags_json,
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_container_impl(db, id)
}

pub(crate) fn update_container_impl(
    db: &Database,
    id: i64,
    req: &CreateContainerRequest,
) -> Result<Container, String> {
    let env_json = req.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
    let ports_json = req
        .ports
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap());
    let volumes_json = req
        .volumes
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap());
    let mounts_json = req
        .mounts
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap());
    let features_json = req
        .features
        .as_ref()
        .map(|f| serde_json::to_string(f).unwrap());
    let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let docker_host_id = req.docker_host_id.unwrap_or(1);

    db.conn()
        .execute(
            "UPDATE containers SET name = ?, description = ?, container_type = ?,
             docker_host_id = ?, image = ?, dockerfile = ?, devcontainer_json = ?,
             env = ?, ports = ?, volumes = ?, mounts = ?, features = ?,
             post_create_command = ?, post_start_command = ?, working_dir = ?,
             template_id = ?, repo_url = ?, icon = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![
                req.name,
                req.description,
                req.container_type,
                docker_host_id,
                req.image,
                req.dockerfile,
                req.devcontainer_json,
                env_json,
                ports_json,
                volumes_json,
                mounts_json,
                features_json,
                req.post_create_command,
                req.post_start_command,
                req.working_dir,
                req.template_id,
                req.repo_url,
                req.icon,
                tags_json,
                id,
            ],
        )
        .map_err(|e| e.to_string())?;

    get_container_impl(db, id)
}

pub(crate) fn delete_container_impl(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM containers WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn toggle_container_favorite_impl(db: &Database, id: i64) -> Result<Container, String> {
    db.conn()
        .execute(
            "UPDATE containers SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            [id],
        )
        .map_err(|e| e.to_string())?;
    get_container_impl(db, id)
}

// ============================================================================
// Project-container junction CRUD
// ============================================================================

fn assign_container_to_project_impl(
    db: &Database,
    project_id: i64,
    container_id: i64,
) -> Result<(), String> {
    db.conn()
        .execute(
            "INSERT OR IGNORE INTO project_containers (project_id, container_id) VALUES (?, ?)",
            params![project_id, container_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn remove_container_from_project_impl(
    db: &Database,
    project_id: i64,
    container_id: i64,
) -> Result<(), String> {
    db.conn()
        .execute(
            "DELETE FROM project_containers WHERE project_id = ? AND container_id = ?",
            params![project_id, container_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_project_containers_impl(
    db: &Database,
    project_id: i64,
) -> Result<Vec<ProjectContainer>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT pc.id, pc.project_id, pc.container_id, pc.is_default, pc.created_at,
                    c.id, c.name, c.description, c.container_type, c.docker_host_id, c.docker_container_id,
                    c.image, c.dockerfile, c.devcontainer_json, c.env, c.ports, c.volumes, c.mounts,
                    c.features, c.post_create_command, c.post_start_command, c.working_dir,
                    c.template_id, c.repo_url, c.icon, c.tags, c.is_favorite, c.created_at, c.updated_at
             FROM project_containers pc
             JOIN containers c ON c.id = pc.container_id
             WHERE pc.project_id = ?
             ORDER BY pc.is_default DESC, c.name",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([project_id], |row| {
            let container = Container {
                id: row.get(5)?,
                name: row.get(6)?,
                description: row.get(7)?,
                container_type: row.get(8)?,
                docker_host_id: row.get(9)?,
                docker_container_id: row.get(10)?,
                image: row.get(11)?,
                dockerfile: row.get(12)?,
                devcontainer_json: row.get(13)?,
                env: parse_json_map(row.get(14)?),
                ports: parse_json_port_mappings(row.get(15)?),
                volumes: parse_json_volume_mappings(row.get(16)?),
                mounts: parse_json_array(row.get(17)?),
                features: parse_json_array(row.get(18)?),
                post_create_command: row.get(19)?,
                post_start_command: row.get(20)?,
                working_dir: row.get(21)?,
                template_id: row.get(22)?,
                repo_url: row.get(23)?,
                icon: row.get(24)?,
                tags: parse_json_array(row.get(25)?),
                is_favorite: row.get::<_, i32>(26)? != 0,
                created_at: row.get(27)?,
                updated_at: row.get(28)?,
            };

            Ok(ProjectContainer {
                id: row.get(0)?,
                project_id: row.get(1)?,
                container_id: row.get(2)?,
                container,
                is_default: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

fn set_default_project_container_impl(
    db: &Database,
    project_id: i64,
    container_id: i64,
) -> Result<(), String> {
    // Reset all defaults for this project
    db.conn()
        .execute(
            "UPDATE project_containers SET is_default = 0 WHERE project_id = ?",
            [project_id],
        )
        .map_err(|e| e.to_string())?;

    // Set the new default
    db.conn()
        .execute(
            "UPDATE project_containers SET is_default = 1 WHERE project_id = ? AND container_id = ?",
            params![project_id, container_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Built-in container templates
// ============================================================================

fn get_builtin_templates() -> Vec<ContainerTemplate> {
    vec![
        // === General ===
        ContainerTemplate {
            id: "ubuntu-dev".to_string(),
            name: "Ubuntu Development".to_string(),
            description: "General-purpose Linux dev environment with git, curl, and common tools"
                .to_string(),
            category: "General".to_string(),
            icon: "\u{1F427}".to_string(),
            image: "mcr.microsoft.com/devcontainers/base:ubuntu".to_string(),
            dockerfile: None,
            env: None,
            ports: None,
            volumes: None,
            features: Some(vec![
                "git".to_string(),
                "curl".to_string(),
                "wget".to_string(),
            ]),
            post_create_command: None,
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        // === Languages ===
        ContainerTemplate {
            id: "node-dev".to_string(),
            name: "Node.js".to_string(),
            description: "Node.js 20 with npm, git, and common dev tools".to_string(),
            category: "Languages".to_string(),
            icon: "\u{1F7E2}".to_string(),
            image: "mcr.microsoft.com/devcontainers/javascript-node:20".to_string(),
            dockerfile: None,
            env: Some(HashMap::from([(
                "NODE_ENV".to_string(),
                "development".to_string(),
            )])),
            ports: Some(vec![PortMapping {
                host_port: 3000,
                container_port: 3000,
                protocol: Some("tcp".to_string()),
            }]),
            volumes: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "typescript-fullstack".to_string(),
            name: "TypeScript Full-Stack".to_string(),
            description: "TypeScript/Node.js with pnpm, ideal for full-stack web apps".to_string(),
            category: "Languages".to_string(),
            icon: "\u{1F535}".to_string(),
            image: "mcr.microsoft.com/devcontainers/typescript-node:20".to_string(),
            dockerfile: None,
            env: Some(HashMap::from([(
                "NODE_ENV".to_string(),
                "development".to_string(),
            )])),
            ports: Some(vec![
                PortMapping {
                    host_port: 3000,
                    container_port: 3000,
                    protocol: Some("tcp".to_string()),
                },
                PortMapping {
                    host_port: 5173,
                    container_port: 5173,
                    protocol: Some("tcp".to_string()),
                },
            ]),
            volumes: None,
            features: None,
            post_create_command: Some("npm install -g pnpm typescript".to_string()),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "rust-tauri-dev".to_string(),
            name: "Rust / Tauri".to_string(),
            description:
                "Rust with Cargo, clippy, rustfmt, and Tauri CLI for desktop app development"
                    .to_string(),
            category: "Languages".to_string(),
            icon: "\u{1F980}".to_string(),
            image: "mcr.microsoft.com/devcontainers/rust:latest".to_string(),
            dockerfile: None,
            env: None,
            ports: Some(vec![PortMapping {
                host_port: 1420,
                container_port: 1420,
                protocol: Some("tcp".to_string()),
            }]),
            volumes: None,
            features: None,
            post_create_command: Some(
                "rustup component add clippy rustfmt && cargo install tauri-cli".to_string(),
            ),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "python-dev".to_string(),
            name: "Python".to_string(),
            description: "Python 3.12 with pip, venv support, and common dev tools".to_string(),
            category: "Languages".to_string(),
            icon: "\u{1F40D}".to_string(),
            image: "mcr.microsoft.com/devcontainers/python:3.12".to_string(),
            dockerfile: None,
            env: Some(HashMap::from([(
                "PYTHONDONTWRITEBYTECODE".to_string(),
                "1".to_string(),
            )])),
            ports: Some(vec![PortMapping {
                host_port: 8000,
                container_port: 8000,
                protocol: Some("tcp".to_string()),
            }]),
            volumes: None,
            features: None,
            post_create_command: Some("pip install --upgrade pip".to_string()),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "go-dev".to_string(),
            name: "Go".to_string(),
            description: "Go 1.22 with standard toolchain and common dev tools".to_string(),
            category: "Languages".to_string(),
            icon: "\u{1F439}".to_string(),
            image: "mcr.microsoft.com/devcontainers/go:1.22".to_string(),
            dockerfile: None,
            env: Some(HashMap::from([("GOPATH".to_string(), "/go".to_string())])),
            ports: Some(vec![PortMapping {
                host_port: 8080,
                container_port: 8080,
                protocol: Some("tcp".to_string()),
            }]),
            volumes: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "dotnet-dev".to_string(),
            name: ".NET".to_string(),
            description: ".NET 8 SDK with C# support for web APIs and apps".to_string(),
            category: "Languages".to_string(),
            icon: "\u{1F7E3}".to_string(),
            image: "mcr.microsoft.com/devcontainers/dotnet:8.0".to_string(),
            dockerfile: None,
            env: Some(HashMap::from([(
                "ASPNETCORE_ENVIRONMENT".to_string(),
                "Development".to_string(),
            )])),
            ports: Some(vec![
                PortMapping {
                    host_port: 5000,
                    container_port: 5000,
                    protocol: Some("tcp".to_string()),
                },
                PortMapping {
                    host_port: 5001,
                    container_port: 5001,
                    protocol: Some("tcp".to_string()),
                },
            ]),
            volumes: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        // === Databases ===
        ContainerTemplate {
            id: "postgres".to_string(),
            name: "PostgreSQL".to_string(),
            description: "PostgreSQL 16 database server".to_string(),
            category: "Databases".to_string(),
            icon: "\u{1F418}".to_string(),
            image: "postgres:16-alpine".to_string(),
            dockerfile: None,
            env: Some(HashMap::from([
                ("POSTGRES_PASSWORD".to_string(), "postgres".to_string()),
                ("POSTGRES_DB".to_string(), "devdb".to_string()),
            ])),
            ports: Some(vec![PortMapping {
                host_port: 5432,
                container_port: 5432,
                protocol: Some("tcp".to_string()),
            }]),
            volumes: Some(vec![VolumeMapping {
                host_path: "pgdata".to_string(),
                container_path: "/var/lib/postgresql/data".to_string(),
                read_only: Some(false),
            }]),
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: None,
        },
        ContainerTemplate {
            id: "redis".to_string(),
            name: "Redis".to_string(),
            description: "Redis 7 in-memory data store".to_string(),
            category: "Databases".to_string(),
            icon: "\u{1F534}".to_string(),
            image: "redis:7-alpine".to_string(),
            dockerfile: None,
            env: None,
            ports: Some(vec![PortMapping {
                host_port: 6379,
                container_port: 6379,
                protocol: Some("tcp".to_string()),
            }]),
            volumes: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: None,
        },
    ]
}
