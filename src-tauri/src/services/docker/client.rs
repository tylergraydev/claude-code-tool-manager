use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, LogOutput, LogsOptions,
    RemoveContainerOptions, RestartContainerOptions, StartContainerOptions, StatsOptions,
    StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults};
use bollard::image::{BuildImageOptions, CreateImageOptions};
use bollard::Docker;
use bytes::Bytes;
use futures::StreamExt;
use log::{error, info, warn};
use std::collections::HashMap;
use std::default::Default;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use crate::db::models::{Container, ContainerLog, ContainerStats, ContainerStatus, ExecResult};

/// Manages Docker client connections for container operations.
pub struct DockerClientManager {
    clients: RwLock<HashMap<i64, Docker>>,
}

impl DockerClientManager {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a Docker client for a given host ID.
    /// Host ID 1 is always the local Docker daemon.
    async fn get_client(&self, host_id: i64) -> Result<Docker, String> {
        // Check cache first
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&host_id) {
                return Ok(client.clone());
            }
        }

        // Create new client - for now only local is supported
        let client = Docker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

        let mut clients = self.clients.write().await;
        clients.insert(host_id, client.clone());
        Ok(client)
    }

    /// Connect to Docker based on host parameters (for testing connections)
    fn connect_with_params(
        host_type: &str,
        connection_uri: Option<&str>,
        _ssh_key_path: Option<&str>,
    ) -> Result<Docker, String> {
        match host_type {
            "local" => Docker::connect_with_local_defaults()
                .map_err(|e| format!("Failed to connect to local Docker: {}", e)),
            "tcp" => {
                let uri = connection_uri.ok_or("TCP connection requires a connection URI")?;
                Docker::connect_with_http(uri, 30, bollard::API_DEFAULT_VERSION)
                    .map_err(|e| format!("Failed to connect to Docker at {}: {}", uri, e))
            }
            _ => Err(format!("Unsupported host type: {}", host_type)),
        }
    }

    /// Check if the local Docker daemon is available
    pub async fn is_docker_available(&self) -> bool {
        match self.get_client(1).await {
            Ok(client) => client.ping().await.is_ok(),
            Err(_) => false,
        }
    }

    /// Ping a Docker host by connection parameters
    pub async fn ping_host(
        &self,
        host_type: &str,
        connection_uri: Option<&str>,
        ssh_key_path: Option<&str>,
    ) -> Result<bool, String> {
        let client = Self::connect_with_params(host_type, connection_uri, ssh_key_path)?;
        match client.ping().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("[Docker] Ping failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Pull a Docker image
    pub async fn pull_image(&self, image: &str, host_id: i64) -> Result<(), String> {
        let client = self.get_client(host_id).await?;
        info!("[Docker] Pulling image: {}", image);

        let (repo, tag) = if let Some(pos) = image.rfind(':') {
            (&image[..pos], &image[pos + 1..])
        } else {
            (image, "latest")
        };

        let options = CreateImageOptions {
            from_image: repo,
            tag,
            ..Default::default()
        };

        let mut stream = client.create_image(Some(options), None, None);
        while let Some(result) = stream.next().await {
            match result {
                Ok(_info) => {} // Progress update, could log if needed
                Err(e) => return Err(format!("Failed to pull image {}: {}", image, e)),
            }
        }

        info!("[Docker] Successfully pulled image: {}", image);
        Ok(())
    }

    /// Build a Docker image from a Dockerfile string.
    /// Returns the tag of the built image.
    pub async fn build_from_dockerfile(
        &self,
        dockerfile_content: &str,
        tag: &str,
        host_id: i64,
    ) -> Result<String, String> {
        let client = self.get_client(host_id).await?;
        info!("[Docker] Building image '{}' from Dockerfile", tag);

        // Create a tar archive containing the Dockerfile
        let mut header = tar::Header::new_gnu();
        let dockerfile_bytes = dockerfile_content.as_bytes();
        header.set_size(dockerfile_bytes.len() as u64);
        header
            .set_path("Dockerfile")
            .map_err(|e| format!("Failed to set tar path: {}", e))?;
        header.set_mode(0o644);
        header.set_cksum();

        let mut tar_buf = Vec::new();
        {
            let mut tar_builder = tar::Builder::new(&mut tar_buf);
            tar_builder
                .append(&header, dockerfile_bytes)
                .map_err(|e| format!("Failed to append Dockerfile to tar: {}", e))?;
            tar_builder
                .finish()
                .map_err(|e| format!("Failed to finish tar: {}", e))?;
        }

        let options = BuildImageOptions {
            dockerfile: "Dockerfile".to_string(),
            t: tag.to_string(),
            pull: true,
            rm: true,
            ..Default::default()
        };

        let mut stream = client.build_image(options, None, Some(Bytes::from(tar_buf)));
        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(ref error) = info.error {
                        return Err(format!("Docker build error: {}", error));
                    }
                }
                Err(e) => return Err(format!("Failed to build image: {}", e)),
            }
        }

        info!("[Docker] Successfully built image: {}", tag);
        Ok(tag.to_string())
    }

    /// Create a Docker container from a Container model
    pub async fn create_docker_container(
        &self,
        container: &Container,
        claude_settings: Option<&crate::commands::settings::ContainerClaudeSettings>,
    ) -> Result<String, String> {
        let client = self.get_client(container.docker_host_id).await?;

        let image = container
            .image
            .as_deref()
            .ok_or("Container has no image specified")?;

        // Build exposed ports and port bindings
        let mut exposed_ports = HashMap::new();
        let mut port_bindings = HashMap::new();

        if let Some(ref ports) = container.ports {
            for pm in ports {
                let proto = pm.protocol.as_deref().unwrap_or("tcp");
                let container_port = format!("{}/{}", pm.container_port, proto);
                exposed_ports.insert(container_port.clone(), HashMap::new());
                port_bindings.insert(
                    container_port,
                    Some(vec![bollard::models::PortBinding {
                        host_ip: Some("127.0.0.1".to_string()),
                        host_port: Some(pm.host_port.to_string()),
                    }]),
                );
            }
        }

        // Build volume binds (with host path validation)
        let mut binds = Vec::new();
        if let Some(ref volumes) = container.volumes {
            for vm in volumes {
                crate::commands::containers::validate_volume_host_path(&vm.host_path)?;
                let bind = if vm.read_only.unwrap_or(false) {
                    format!("{}:{}:ro", vm.host_path, vm.container_path)
                } else {
                    format!("{}:{}", vm.host_path, vm.container_path)
                };
                binds.push(bind);
            }
        }
        if let Some(ref mounts) = container.mounts {
            binds.extend(mounts.iter().cloned());
        }

        // Build env vars
        let mut env: Vec<String> = container
            .env
            .as_ref()
            .map(|e| e.iter().map(|(k, v)| format!("{}={}", k, v)).collect())
            .unwrap_or_default();

        // Apply Claude Code settings
        if let Some(cs) = claude_settings {
            // Mount ~/.claude/ for Max plan auth
            if cs.auto_mount_claude_dir {
                if let Some(host_claude_dir) = crate::commands::settings::get_host_claude_dir() {
                    let path = std::path::Path::new(&host_claude_dir);
                    if path.exists() {
                        // Mount as /root/.claude and /home/node/.claude to cover common user setups (read-only)
                        binds.push(format!("{}:/root/.claude:ro", host_claude_dir));
                        binds.push(format!("{}:/home/node/.claude:ro", host_claude_dir));
                        info!(
                            "[Docker] Auto-mounting Claude auth dir: {}",
                            host_claude_dir
                        );
                    }
                }
            }

            // Inject API key if using api_key mode
            if cs.auth_mode == "api_key" {
                if let Some(ref key) = cs.api_key {
                    env.push(format!("ANTHROPIC_API_KEY={}", key));
                }
            }
        }

        let host_config = bollard::models::HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            binds: if binds.is_empty() { None } else { Some(binds) },
            ..Default::default()
        };

        let config = Config {
            image: Some(image.to_string()),
            env: if env.is_empty() { None } else { Some(env) },
            exposed_ports: if exposed_ports.is_empty() {
                None
            } else {
                Some(exposed_ports)
            },
            working_dir: container.working_dir.as_deref().map(String::from),
            host_config: Some(host_config),
            // Keep container alive as a dev environment
            cmd: Some(vec!["sleep".to_string(), "infinity".to_string()]),
            tty: Some(true),
            open_stdin: Some(true),
            ..Default::default()
        };

        let container_name = format!("cctm-{}", container.name.to_lowercase().replace(' ', "-"));
        let options = CreateContainerOptions {
            name: &container_name,
            platform: None,
        };

        let response = client
            .create_container(Some(options), config)
            .await
            .map_err(|e| format!("Failed to create container: {}", e))?;

        info!("[Docker] Created container: {}", response.id);
        Ok(response.id)
    }

    /// Start a Docker container
    pub async fn start_container(&self, docker_id: &str, host_id: i64) -> Result<(), String> {
        let client = self.get_client(host_id).await?;
        client
            .start_container(docker_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| format!("Failed to start container: {}", e))?;
        info!("[Docker] Started container: {}", docker_id);
        Ok(())
    }

    /// Stop a Docker container
    pub async fn stop_container(&self, docker_id: &str, host_id: i64) -> Result<(), String> {
        let client = self.get_client(host_id).await?;
        let options = StopContainerOptions { t: 10 };
        client
            .stop_container(docker_id, Some(options))
            .await
            .map_err(|e| format!("Failed to stop container: {}", e))?;
        info!("[Docker] Stopped container: {}", docker_id);
        Ok(())
    }

    /// Restart a Docker container
    pub async fn restart_container(&self, docker_id: &str, host_id: i64) -> Result<(), String> {
        let client = self.get_client(host_id).await?;
        let options = RestartContainerOptions { t: 10 };
        client
            .restart_container(docker_id, Some(options))
            .await
            .map_err(|e| format!("Failed to restart container: {}", e))?;
        info!("[Docker] Restarted container: {}", docker_id);
        Ok(())
    }

    /// Remove a Docker container
    pub async fn remove_container(&self, docker_id: &str, host_id: i64) -> Result<(), String> {
        let client = self.get_client(host_id).await?;
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        client
            .remove_container(docker_id, Some(options))
            .await
            .map_err(|e| format!("Failed to remove container: {}", e))?;
        info!("[Docker] Removed container: {}", docker_id);
        Ok(())
    }

    /// Inspect a container and return its status
    pub async fn inspect_container_status(
        &self,
        container_id: i64,
        docker_id: &str,
        host_id: i64,
    ) -> Result<ContainerStatus, String> {
        let client = self.get_client(host_id).await?;
        let inspect = client
            .inspect_container(docker_id, None::<InspectContainerOptions>)
            .await
            .map_err(|e| {
                // If container not found, return not_created status
                if e.to_string().contains("404") || e.to_string().contains("No such container") {
                    return format!("not_found:{}", e);
                }
                format!("Failed to inspect container: {}", e)
            });

        match inspect {
            Ok(info) => {
                let state = info.state.as_ref();
                let docker_status = state
                    .and_then(|s| s.status)
                    .map(|s| format!("{:?}", s).to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());

                Ok(ContainerStatus {
                    container_id,
                    docker_status,
                    docker_container_id: Some(docker_id.to_string()),
                    started_at: state.and_then(|s| s.started_at.clone()),
                    finished_at: state.and_then(|s| s.finished_at.clone()),
                    exit_code: state.and_then(|s| s.exit_code),
                    health: state
                        .and_then(|s| s.health.as_ref())
                        .and_then(|h| h.status)
                        .map(|s| format!("{:?}", s).to_lowercase()),
                    cpu_percent: None,
                    memory_usage: None,
                    memory_limit: None,
                })
            }
            Err(e) if e.starts_with("not_found:") => Ok(ContainerStatus {
                container_id,
                docker_status: "not_created".to_string(),
                docker_container_id: Some(docker_id.to_string()),
                started_at: None,
                finished_at: None,
                exit_code: None,
                health: None,
                cpu_percent: None,
                memory_usage: None,
                memory_limit: None,
            }),
            Err(e) => Err(e),
        }
    }

    /// Get container logs
    pub async fn get_logs(
        &self,
        docker_id: &str,
        host_id: i64,
        tail: Option<u32>,
        since: Option<i64>,
    ) -> Result<Vec<ContainerLog>, String> {
        let client = self.get_client(host_id).await?;

        let tail_str = tail
            .map(|t| t.to_string())
            .unwrap_or_else(|| "100".to_string());

        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail_str,
            since: since.unwrap_or(0),
            timestamps: true,
            ..Default::default()
        };

        let mut stream = client.logs(docker_id, Some(options));
        let mut logs = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    let (stream_type, message) = match output {
                        LogOutput::StdOut { message } => {
                            ("stdout", String::from_utf8_lossy(&message).to_string())
                        }
                        LogOutput::StdErr { message } => {
                            ("stderr", String::from_utf8_lossy(&message).to_string())
                        }
                        LogOutput::Console { message } => {
                            ("stdout", String::from_utf8_lossy(&message).to_string())
                        }
                        LogOutput::StdIn { message: _ } => continue,
                    };

                    // Try to extract timestamp from message if timestamps enabled
                    let (timestamp, msg) = if let Some(space_pos) = message.find(' ') {
                        let potential_ts = &message[..space_pos];
                        if potential_ts.contains('T') || potential_ts.contains('-') {
                            (
                                Some(potential_ts.to_string()),
                                message[space_pos + 1..].to_string(),
                            )
                        } else {
                            (None, message)
                        }
                    } else {
                        (None, message)
                    };

                    logs.push(ContainerLog {
                        timestamp,
                        stream: stream_type.to_string(),
                        message: msg.trim_end().to_string(),
                    });
                }
                Err(e) => {
                    error!("[Docker] Error reading logs: {}", e);
                    break;
                }
            }
        }

        Ok(logs)
    }

    /// Get container stats (single snapshot)
    pub async fn get_stats(
        &self,
        container_id: i64,
        docker_id: &str,
        host_id: i64,
    ) -> Result<ContainerStats, String> {
        let client = self.get_client(host_id).await?;

        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stream = client.stats(docker_id, Some(options));

        if let Some(result) = stream.next().await {
            let stats: bollard::container::Stats =
                result.map_err(|e| format!("Failed to get stats: {}", e))?;

            // Calculate CPU percentage
            let cpu_percent = calculate_cpu_percent(&stats);

            // Memory stats
            let memory_usage = stats.memory_stats.usage.unwrap_or(0);
            let memory_limit = stats.memory_stats.limit.unwrap_or(0);
            let memory_percent = if memory_limit > 0 {
                (memory_usage as f64 / memory_limit as f64) * 100.0
            } else {
                0.0
            };

            // Network stats
            let (network_rx, network_tx) = if let Some(ref nets) = stats.networks {
                nets.values().fold((0u64, 0u64), |(rx, tx), net| {
                    (rx + net.rx_bytes, tx + net.tx_bytes)
                })
            } else {
                (0, 0)
            };

            // Block I/O stats
            let (block_read, block_write) =
                if let Some(ref entries) = stats.blkio_stats.io_service_bytes_recursive {
                    entries.iter().fold((0u64, 0u64), |(read, write), entry| {
                        match entry.op.as_str() {
                            "read" | "Read" => (read + entry.value, write),
                            "write" | "Write" => (read, write + entry.value),
                            _ => (read, write),
                        }
                    })
                } else {
                    (0, 0)
                };

            let pids = stats.pids_stats.current.unwrap_or(0);

            Ok(ContainerStats {
                container_id,
                cpu_percent,
                memory_usage,
                memory_limit,
                memory_percent,
                network_rx_bytes: network_rx,
                network_tx_bytes: network_tx,
                block_read_bytes: block_read,
                block_write_bytes: block_write,
                pids,
            })
        } else {
            Err("No stats returned".to_string())
        }
    }

    /// Execute a command in a container
    pub async fn exec(
        &self,
        docker_id: &str,
        host_id: i64,
        command: Vec<String>,
    ) -> Result<ExecResult, String> {
        let client = self.get_client(host_id).await?;

        let exec_options = CreateExecOptions {
            cmd: Some(command),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = client
            .create_exec(docker_id, exec_options)
            .await
            .map_err(|e| format!("Failed to create exec: {}", e))?;

        let start_result = client
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| format!("Failed to start exec: {}", e))?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let StartExecResults::Attached { mut output, .. } = start_result {
            while let Some(result) = output.next().await {
                match result {
                    Ok(log_output) => match log_output {
                        LogOutput::StdOut { message } => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        LogOutput::StdErr { message } => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        _ => {}
                    },
                    Err(e) => {
                        error!("[Docker] Exec output error: {}", e);
                        break;
                    }
                }
            }
        }

        // Get exit code
        let inspect = client
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| format!("Failed to inspect exec: {}", e))?;

        let exit_code = inspect.exit_code.unwrap_or(-1);

        Ok(ExecResult {
            exit_code,
            stdout,
            stderr,
        })
    }

    /// Start an interactive shell session in a container
    /// Returns the exec ID for resize operations
    pub async fn start_interactive_shell(
        &self,
        docker_id: &str,
        host_id: i64,
        app_handle: tauri::AppHandle,
        session_id: String,
    ) -> Result<String, String> {
        let client = self.get_client(host_id).await?;

        let exec_options = CreateExecOptions {
            cmd: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                "if command -v bash >/dev/null 2>&1; then exec bash; else exec sh; fi".to_string(),
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            attach_stdin: Some(true),
            tty: Some(true),
            ..Default::default()
        };

        let exec = client
            .create_exec(docker_id, exec_options)
            .await
            .map_err(|e| format!("Failed to create exec: {}", e))?;

        let exec_id = exec.id.clone();

        // Resize to reasonable default
        let _ = client
            .resize_exec(
                &exec_id,
                ResizeExecOptions {
                    height: 24,
                    width: 80,
                },
            )
            .await;

        let start_opts = StartExecOptions {
            detach: false,
            ..Default::default()
        };

        let start_result = client
            .start_exec(&exec_id, Some(start_opts))
            .await
            .map_err(|e| format!("Failed to start exec: {}", e))?;

        if let StartExecResults::Attached {
            mut output,
            mut input,
        } = start_result
        {
            let session_id_clone = session_id.clone();
            let app_clone = app_handle.clone();

            // Spawn task to read output and emit events
            tokio::spawn(async move {
                while let Some(result) = output.next().await {
                    match result {
                        Ok(log_output) => {
                            let data = match log_output {
                                LogOutput::StdOut { message } => message,
                                LogOutput::StdErr { message } => message,
                                LogOutput::Console { message } => message,
                                _ => continue,
                            };
                            let _ = app_clone.emit(
                                &format!("terminal-output-{}", session_id_clone),
                                String::from_utf8_lossy(&data).to_string(),
                            );
                        }
                        Err(e) => {
                            error!("[Docker] Shell output error: {}", e);
                            break;
                        }
                    }
                }
                let _ = app_clone.emit(&format!("terminal-exit-{}", session_id_clone), ());
            });

            // Spawn task to listen for input events
            let _app_clone2 = app_handle.clone();
            let session_id_clone2 = session_id.clone();
            tokio::spawn(async move {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(256);

                // Register the sender in a global map
                {
                    let mut senders = SHELL_SENDERS.lock().await;
                    senders.insert(session_id_clone2.clone(), tx);
                }

                while let Some(data) = rx.recv().await {
                    if let Err(e) = input.write_all(data.as_bytes()).await {
                        error!("[Docker] Shell input error: {}", e);
                        break;
                    }
                    if let Err(e) = input.flush().await {
                        error!("[Docker] Shell flush error: {}", e);
                        break;
                    }
                }

                // Cleanup
                {
                    let mut senders = SHELL_SENDERS.lock().await;
                    senders.remove(&session_id_clone2);
                }
            });
        }

        Ok(exec_id)
    }

    /// Resize a terminal session
    pub async fn resize_shell(
        &self,
        exec_id: &str,
        host_id: i64,
        rows: u16,
        cols: u16,
    ) -> Result<(), String> {
        let client = self.get_client(host_id).await?;
        client
            .resize_exec(
                exec_id,
                ResizeExecOptions {
                    height: rows,
                    width: cols,
                },
            )
            .await
            .map_err(|e| format!("Failed to resize: {}", e))?;
        Ok(())
    }
}

use once_cell::sync::Lazy;
/// Global map of shell input senders
use tokio::sync::Mutex as TokioMutex;
static SHELL_SENDERS: Lazy<TokioMutex<HashMap<String, tokio::sync::mpsc::Sender<String>>>> =
    Lazy::new(|| TokioMutex::new(HashMap::new()));

/// Send input to a shell session (called from Tauri command)
pub async fn send_shell_input(session_id: &str, data: String) -> Result<(), String> {
    let senders = SHELL_SENDERS.lock().await;
    if let Some(tx) = senders.get(session_id) {
        tx.send(data)
            .await
            .map_err(|e| format!("Failed to send input: {}", e))?;
        Ok(())
    } else {
        Err("Shell session not found".to_string())
    }
}

/// Calculate CPU percentage from Docker stats
fn calculate_cpu_percent(stats: &bollard::container::Stats) -> f64 {
    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
        - stats.precpu_stats.cpu_usage.total_usage as f64;

    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
        - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

    let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

    if system_delta > 0.0 && cpu_delta >= 0.0 {
        (cpu_delta / system_delta) * num_cpus * 100.0
    } else {
        0.0
    }
}
