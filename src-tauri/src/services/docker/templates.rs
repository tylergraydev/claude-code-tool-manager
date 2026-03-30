use crate::db::models::{ContainerTemplate, PortMapping};

pub fn get_builtin_templates() -> Vec<ContainerTemplate> {
    vec![
        ContainerTemplate {
            id: "claude-code-node".to_string(),
            name: "Claude Code + Node.js 20".to_string(),
            description: "Node.js 20 LTS with Claude Code CLI pre-installed".to_string(),
            category: "claude-code".to_string(),
            icon: "⬡".to_string(),
            image: "node:20-bookworm".to_string(),
            dockerfile: Some(
                r#"FROM node:20-bookworm
RUN apt-get update && apt-get install -y curl git && \
    npm install -g @anthropic-ai/claude-code && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace"#
                    .to_string(),
            ),
            env: None,
            ports: None,
            volumes: None,
            features: None,
            post_create_command: Some("claude --version".to_string()),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "claude-code-python".to_string(),
            name: "Claude Code + Python".to_string(),
            description: "Python 3.12 with Claude Code CLI pre-installed".to_string(),
            category: "claude-code".to_string(),
            icon: "🐍".to_string(),
            image: "python:3.12-bookworm".to_string(),
            dockerfile: Some(
                r#"FROM python:3.12-bookworm
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs git && \
    npm install -g @anthropic-ai/claude-code && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace"#
                    .to_string(),
            ),
            env: None,
            ports: None,
            volumes: None,
            features: None,
            post_create_command: Some("claude --version && python --version".to_string()),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "rust-dev".to_string(),
            name: "Rust Dev Environment".to_string(),
            description: "Rust stable with cargo tools and common dev utilities".to_string(),
            category: "language".to_string(),
            icon: "🦀".to_string(),
            image: "rust:bookworm".to_string(),
            dockerfile: Some(
                r#"FROM rust:bookworm
RUN apt-get update && apt-get install -y git pkg-config libssl-dev && \
    rustup component add clippy rustfmt && \
    cargo install cargo-watch cargo-edit && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace"#
                    .to_string(),
            ),
            env: None,
            ports: None,
            volumes: None,
            features: None,
            post_create_command: Some("rustc --version && cargo --version".to_string()),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "python-ml".to_string(),
            name: "Python ML".to_string(),
            description: "Python 3.12 with PyTorch, NumPy, and Jupyter".to_string(),
            category: "data-science".to_string(),
            icon: "🧠".to_string(),
            image: "python:3.12-bookworm".to_string(),
            dockerfile: Some(
                r#"FROM python:3.12-bookworm
RUN pip install --no-cache-dir torch numpy pandas jupyter matplotlib scikit-learn
WORKDIR /workspace"#
                    .to_string(),
            ),
            env: None,
            ports: Some(vec![PortMapping {
                host_port: 8888,
                container_port: 8888,
                protocol: Some("tcp".to_string()),
            }]),
            volumes: None,
            features: None,
            post_create_command: Some(
                "python -c 'import torch; print(f\"PyTorch {torch.__version__}\")'".to_string(),
            ),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "fullstack".to_string(),
            name: "Full Stack".to_string(),
            description: "Node.js + Python + PostgreSQL client for full-stack development"
                .to_string(),
            category: "fullstack".to_string(),
            icon: "🚀".to_string(),
            image: "node:20-bookworm".to_string(),
            dockerfile: Some(
                r#"FROM node:20-bookworm
RUN apt-get update && apt-get install -y python3 python3-pip python3-venv postgresql-client git && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace"#
                    .to_string(),
            ),
            env: None,
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
            post_create_command: Some("node --version && python3 --version".to_string()),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
        ContainerTemplate {
            id: "claude-code-base".to_string(),
            name: "Claude Code Base".to_string(),
            description: "Minimal Ubuntu with Claude Code CLI only".to_string(),
            category: "claude-code".to_string(),
            icon: "🤖".to_string(),
            image: "ubuntu:24.04".to_string(),
            dockerfile: Some(
                r#"FROM ubuntu:24.04
RUN apt-get update && apt-get install -y curl git && \
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g @anthropic-ai/claude-code && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace"#
                    .to_string(),
            ),
            env: None,
            ports: None,
            volumes: None,
            features: None,
            post_create_command: Some("claude --version".to_string()),
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_count() {
        let templates = get_builtin_templates();
        assert_eq!(templates.len(), 6);
    }

    #[test]
    fn test_all_templates_have_required_fields() {
        let templates = get_builtin_templates();
        for t in &templates {
            assert!(!t.id.is_empty(), "Template id must not be empty");
            assert!(!t.name.is_empty(), "Template name must not be empty");
            assert!(
                !t.description.is_empty(),
                "Template description must not be empty"
            );
            assert!(
                !t.category.is_empty(),
                "Template category must not be empty"
            );
            assert!(!t.icon.is_empty(), "Template icon must not be empty");
            assert!(!t.image.is_empty(), "Template image must not be empty");
        }
    }

    #[test]
    fn test_all_template_ids_unique() {
        let templates = get_builtin_templates();
        let mut ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), templates.len(), "Template IDs must be unique");
    }

    #[test]
    fn test_templates_with_ports_have_valid_mappings() {
        let templates = get_builtin_templates();
        for t in &templates {
            if let Some(ref ports) = t.ports {
                assert!(!ports.is_empty(), "Ports list should not be empty if Some");
                for pm in ports {
                    assert!(pm.host_port > 0, "Host port must be > 0");
                    assert!(pm.container_port > 0, "Container port must be > 0");
                    if let Some(ref proto) = pm.protocol {
                        assert!(
                            proto == "tcp" || proto == "udp",
                            "Protocol must be tcp or udp, got: {}",
                            proto
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_templates_with_dockerfiles_have_content() {
        let templates = get_builtin_templates();
        for t in &templates {
            if let Some(ref dockerfile) = t.dockerfile {
                assert!(
                    !dockerfile.is_empty(),
                    "Dockerfile content must not be empty for template {}",
                    t.id
                );
                assert!(
                    dockerfile.contains("FROM"),
                    "Dockerfile should contain FROM directive for template {}",
                    t.id
                );
            }
        }
    }

    #[test]
    fn test_python_ml_template_has_jupyter_port() {
        let templates = get_builtin_templates();
        let ml = templates.iter().find(|t| t.id == "python-ml").unwrap();
        let ports = ml.ports.as_ref().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].host_port, 8888);
        assert_eq!(ports[0].container_port, 8888);
    }

    #[test]
    fn test_fullstack_template_has_multiple_ports() {
        let templates = get_builtin_templates();
        let fs = templates.iter().find(|t| t.id == "fullstack").unwrap();
        let ports = fs.ports.as_ref().unwrap();
        assert_eq!(ports.len(), 2);
        let host_ports: Vec<u16> = ports.iter().map(|p| p.host_port).collect();
        assert!(host_ports.contains(&3000));
        assert!(host_ports.contains(&5173));
    }

    #[test]
    fn test_all_templates_have_working_dir() {
        let templates = get_builtin_templates();
        for t in &templates {
            assert_eq!(
                t.working_dir.as_deref(),
                Some("/workspace"),
                "Template {} should have /workspace as working_dir",
                t.id
            );
        }
    }

    #[test]
    fn test_claude_code_category_templates() {
        let templates = get_builtin_templates();
        let cc_templates: Vec<&ContainerTemplate> = templates
            .iter()
            .filter(|t| t.category == "claude-code")
            .collect();
        assert_eq!(cc_templates.len(), 3);
        let ids: Vec<&str> = cc_templates.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"claude-code-node"));
        assert!(ids.contains(&"claude-code-python"));
        assert!(ids.contains(&"claude-code-base"));
    }

    #[test]
    fn test_all_templates_have_post_create_command() {
        let templates = get_builtin_templates();
        for t in &templates {
            assert!(
                t.post_create_command.is_some(),
                "Template {} should have a post_create_command",
                t.id
            );
        }
    }
}
