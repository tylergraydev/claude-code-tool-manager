use anyhow::Result;
use log::{info, warn};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Information about a WSL distro with Claude Code installed
#[derive(Debug, Clone)]
pub struct WslClaudeInfo {
    /// The WSL distro name (e.g., "Ubuntu", "Debian")
    pub distro: String,
    /// Whether Claude Code's .claude/ directory exists in this distro
    pub is_installed: bool,
    /// The home directory path inside WSL (e.g., "/home/user")
    pub wsl_home: Option<String>,
}

/// Check if WSL2 is available on this Windows system
#[cfg(target_os = "windows")]
pub fn is_wsl_available() -> bool {
    Command::new("wsl.exe")
        .args(["--status"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
pub fn is_wsl_available() -> bool {
    false
}

/// List all installed WSL distros
#[cfg(target_os = "windows")]
pub fn list_wsl_distros() -> Result<Vec<String>> {
    let output = Command::new("wsl.exe")
        .args(["--list", "--quiet"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run wsl.exe: {}", e))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    // WSL outputs UTF-16LE, decode it
    let stdout = decode_wsl_output(&output.stdout);

    let distros: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(distros)
}

#[cfg(not(target_os = "windows"))]
pub fn list_wsl_distros() -> Result<Vec<String>> {
    Ok(Vec::new())
}

/// Decode WSL output which may be UTF-16LE (Windows) or UTF-8
fn decode_wsl_output(bytes: &[u8]) -> String {
    // Check for UTF-16LE BOM (0xFF 0xFE)
    let has_bom = bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE;

    // Also detect UTF-16LE by checking for null bytes in alternating positions
    // (common for ASCII text encoded as UTF-16LE)
    let looks_like_utf16 = has_bom
        || (bytes.len() >= 4
            && bytes.len() % 2 == 0
            && bytes[1] == 0
            && bytes[3] == 0
            && bytes[0].is_ascii_alphanumeric());

    if looks_like_utf16 {
        let u16_iter: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        if let Ok(decoded) = String::from_utf16(&u16_iter) {
            // Strip BOM if present
            let result = decoded.strip_prefix('\u{feff}').unwrap_or(&decoded);
            return result.to_string();
        }
    }

    // Fall back to UTF-8
    String::from_utf8_lossy(bytes).to_string()
}

/// Check if Claude Code is installed in a specific WSL distro
#[cfg(target_os = "windows")]
pub fn check_claude_in_distro(distro: &str) -> WslClaudeInfo {
    // Get the home directory
    let wsl_home = get_wsl_home(distro);

    // Check if .claude/ directory exists
    let is_installed = Command::new("wsl.exe")
        .args(["-d", distro, "--", "test", "-d", "$HOME/.claude"])
        .creation_flags(0x08000000)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    WslClaudeInfo {
        distro: distro.to_string(),
        is_installed,
        wsl_home,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn check_claude_in_distro(distro: &str) -> WslClaudeInfo {
    WslClaudeInfo {
        distro: distro.to_string(),
        is_installed: false,
        wsl_home: None,
    }
}

/// Get the home directory path inside a WSL distro
#[cfg(target_os = "windows")]
fn get_wsl_home(distro: &str) -> Option<String> {
    let output = Command::new("wsl.exe")
        .args(["-d", distro, "--", "echo", "$HOME"])
        .creation_flags(0x08000000)
        .output()
        .ok()?;

    if output.status.success() {
        let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !home.is_empty() {
            return Some(home);
        }
    }
    None
}

/// Detect all WSL distros that have Claude Code installed
pub fn detect_wsl_claude_installations() -> Vec<WslClaudeInfo> {
    if !is_wsl_available() {
        return Vec::new();
    }

    let distros = match list_wsl_distros() {
        Ok(d) => d,
        Err(e) => {
            warn!("[WSL] Failed to list distros: {}", e);
            return Vec::new();
        }
    };

    info!("[WSL] Found {} distro(s): {:?}", distros.len(), distros);

    distros
        .iter()
        .map(|distro| {
            let info = check_claude_in_distro(distro);
            info!(
                "[WSL] Distro '{}': claude_installed={}, home={:?}",
                distro, info.is_installed, info.wsl_home
            );
            info
        })
        .collect()
}

/// Read a file from inside a WSL distro
#[cfg(target_os = "windows")]
pub fn read_wsl_file(distro: &str, path: &str) -> Result<String> {
    let output = Command::new("wsl.exe")
        .args(["-d", distro, "--", "cat", path])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to read file from WSL distro '{}': {}", distro, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to read '{}' in WSL distro '{}': {}",
            path,
            distro,
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn read_wsl_file(_distro: &str, _path: &str) -> Result<String> {
    Err(anyhow::anyhow!("WSL is only available on Windows"))
}

/// Write a file inside a WSL distro
#[cfg(target_os = "windows")]
pub fn write_wsl_file(distro: &str, path: &str, content: &str) -> Result<()> {
    use std::process::Stdio;

    // Use a shell command to write via stdin to avoid escaping issues
    let mut child = Command::new("wsl.exe")
        .args(["-d", distro, "--", "sh", "-c", &format!("cat > '{}'", path)])
        .creation_flags(0x08000000)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to write file to WSL distro '{}': {}", distro, e))?;

    {
        use std::io::Write;
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to open stdin for WSL write"))?;
        stdin.write_all(content.as_bytes()).map_err(|e| {
            anyhow::anyhow!("Failed to write content to WSL distro '{}': {}", distro, e)
        })?;
    }

    let output = child.wait_with_output().map_err(|e| {
        anyhow::anyhow!("Failed to wait for WSL write in distro '{}': {}", distro, e)
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to write '{}' in WSL distro '{}': {}",
            path,
            distro,
            stderr
        ));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn write_wsl_file(_distro: &str, _path: &str, _content: &str) -> Result<()> {
    Err(anyhow::anyhow!("WSL is only available on Windows"))
}

/// Create a backup of a file inside a WSL distro
#[cfg(target_os = "windows")]
pub fn backup_wsl_file(distro: &str, path: &str) -> Result<()> {
    let backup_path = format!("{}.bak", path);

    let output = Command::new("wsl.exe")
        .args([
            "-d",
            distro,
            "--",
            "sh",
            "-c",
            &format!("[ -f '{}' ] && cp '{}' '{}'", path, path, backup_path),
        ])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to backup file in WSL distro '{}': {}", distro, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(
            "[WSL] Backup of '{}' in '{}' may have failed: {}",
            path, distro, stderr
        );
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn backup_wsl_file(_distro: &str, _path: &str) -> Result<()> {
    Err(anyhow::anyhow!("WSL is only available on Windows"))
}

/// Create a directory inside a WSL distro (mkdir -p)
#[cfg(target_os = "windows")]
pub fn mkdir_wsl(distro: &str, path: &str) -> Result<()> {
    let output = Command::new("wsl.exe")
        .args(["-d", distro, "--", "mkdir", "-p", path])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to create directory in WSL distro '{}': {}",
                distro,
                e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to create '{}' in WSL distro '{}': {}",
            path,
            distro,
            stderr
        ));
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn mkdir_wsl(_distro: &str, _path: &str) -> Result<()> {
    Err(anyhow::anyhow!("WSL is only available on Windows"))
}

/// Generate the editor ID for a WSL distro
pub fn wsl_editor_id(distro: &str) -> String {
    format!("wsl_{}", distro.to_lowercase().replace(' ', "_"))
}

/// Check if an editor ID represents a WSL distro
pub fn is_wsl_editor(editor_id: &str) -> bool {
    editor_id.starts_with("wsl_")
}

/// Extract the distro name from a WSL editor ID
pub fn distro_from_editor_id(editor_id: &str) -> Option<String> {
    if !is_wsl_editor(editor_id) {
        return None;
    }

    // We need to find the original distro name - try listing distros
    let distros = list_wsl_distros().unwrap_or_default();
    let normalized_id = &editor_id[4..]; // strip "wsl_"

    distros
        .into_iter()
        .find(|d| d.to_lowercase().replace(' ', "_") == normalized_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsl_editor_id() {
        assert_eq!(wsl_editor_id("Ubuntu"), "wsl_ubuntu");
        assert_eq!(wsl_editor_id("Debian"), "wsl_debian");
        assert_eq!(wsl_editor_id("Ubuntu 22.04"), "wsl_ubuntu_22.04");
    }

    #[test]
    fn test_is_wsl_editor() {
        assert!(is_wsl_editor("wsl_ubuntu"));
        assert!(is_wsl_editor("wsl_debian"));
        assert!(!is_wsl_editor("claude_code"));
        assert!(!is_wsl_editor("opencode"));
    }

    #[test]
    fn test_decode_wsl_output_utf8() {
        let input = b"Ubuntu\nDebian\n";
        let result = decode_wsl_output(input);
        assert!(result.contains("Ubuntu"));
        assert!(result.contains("Debian"));
    }

    #[test]
    fn test_decode_wsl_output_utf16le() {
        // "Ubuntu\r\n" in UTF-16LE with BOM
        let mut bytes: Vec<u8> = Vec::new();
        // BOM
        bytes.extend_from_slice(&[0xFF, 0xFE]);
        // "Ubuntu\r\n"
        for c in "Ubuntu\r\n".encode_utf16() {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        let result = decode_wsl_output(&bytes);
        assert!(result.contains("Ubuntu"));
    }

    #[test]
    fn test_decode_wsl_output_empty() {
        let result = decode_wsl_output(b"");
        assert!(result.is_empty());
    }
}
