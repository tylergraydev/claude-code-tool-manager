use crate::db::models::StatusLineSegment;
use anyhow::Result;
use directories::BaseDirs;
use serde_json::{json, Value};
use std::path::Path;

/// Read an existing settings.json file or return an empty object
fn read_settings_file(path: &Path) -> Result<Value> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or(json!({})))
    } else {
        Ok(json!({}))
    }
}

/// Write settings.json file, preserving other settings
fn write_settings_file(path: &Path, settings: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Write the statusLine key to ~/.claude/settings.json
pub fn write_statusline_to_settings(command: &str, padding: i32) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings = read_settings_file(&settings_path)?;

    let mut sl_config = serde_json::Map::new();
    sl_config.insert("type".to_string(), json!("command"));
    sl_config.insert("command".to_string(), json!(command));
    if padding > 0 {
        sl_config.insert("padding".to_string(), json!(padding));
    }

    settings["statusLine"] = Value::Object(sl_config);

    write_settings_file(&settings_path, &settings)
}

/// Remove the statusLine key from ~/.claude/settings.json
pub fn remove_statusline_from_settings() -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings = read_settings_file(&settings_path)?;

    if let Some(obj) = settings.as_object_mut() {
        obj.remove("statusLine");
    }

    write_settings_file(&settings_path, &settings)
}

/// Read the current statusLine config from ~/.claude/settings.json
pub fn read_current_statusline_config() -> Result<Option<Value>> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    let settings_path = home.join(".claude").join("settings.json");

    let settings = read_settings_file(&settings_path)?;
    Ok(settings.get("statusLine").cloned())
}

/// Get the path to the generated statusline script
pub fn get_statusline_script_path() -> Result<std::path::PathBuf> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    Ok(home.join(".claude").join("statusline.py"))
}

/// Write the generated Python script to ~/.claude/statusline.py
pub fn write_statusline_script(script_content: &str) -> Result<std::path::PathBuf> {
    let script_path = get_statusline_script_path()?;
    if let Some(parent) = script_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&script_path, script_content)?;
    Ok(script_path)
}

/// Generate a Python 3 script from status line segments
pub fn generate_script_from_segments(segments: &[StatusLineSegment]) -> String {
    generate_script_from_segments_with_theme(segments, "default")
}

/// Generate a Python 3 script from status line segments with a specific theme
pub fn generate_script_from_segments_with_theme(
    segments: &[StatusLineSegment],
    theme: &str,
) -> String {
    if theme == "powerline" || theme == "powerline_round" {
        return generate_powerline_script(segments, theme);
    }

    let enabled: Vec<&StatusLineSegment> = segments.iter().filter(|s| s.enabled).collect();

    let mut parts_code = String::new();
    let mut needs_usage_api = false;
    let has_line_breaks = enabled.iter().any(|s| s.segment_type == "line_break");

    for seg in &enabled {
        if seg.segment_type == "line_break" {
            parts_code.push_str("    lines.append(\" \".join(parts))\n    parts = []\n");
            continue;
        }

        let color_start = build_color_code(
            seg.color.as_deref().unwrap_or("white"),
            seg.bg_color.as_deref(),
        );
        let color_end = "\\033[0m";
        let label = seg.label.as_deref().unwrap_or("");
        let label_prefix = if label.is_empty() {
            String::new()
        } else {
            format!("{} ", label)
        };

        match seg.segment_type.as_str() {
            "model" => {
                let format = seg.format.as_deref().unwrap_or("short");
                let extract = if format == "short" {
                    r#"name = data.get("model", {}).get("display_name", "") or data.get("model", {}).get("name", "")
    # If no display_name, extract short name: claude-opus-4-6 -> opus
    if "-" in name and len(name) > 10:
        parts_list = name.split("-")
        if len(parts_list) >= 2:
            name = parts_list[1]"#
                } else {
                    r#"name = data.get("model", {}).get("id", "") or data.get("model", {}).get("name", "")"#
                };
                parts_code.push_str(&format!(
                    r#"    {extract}
    if name:
        parts.append(f"{color_start}{label_prefix}{{name}}{color_end}")
"#,
                    extract = extract,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "cost" => {
                let fmt = seg.format.as_deref().unwrap_or("$0.00");
                let decimals = if fmt.contains("0000") || fmt.contains("4") {
                    4
                } else {
                    2
                };
                parts_code.push_str(&format!(
                    r#"    cost = data.get("cost", {{}}).get("total_cost_usd", 0)
    parts.append(f"{color_start}{label_prefix}${{cost:.{decimals}f}}{color_end}")
"#,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    decimals = decimals,
                    color_end = color_end
                ));
            }
            "context" => {
                let fmt = seg.format.as_deref().unwrap_or("percentage");
                match fmt {
                    "fraction" => {
                        parts_code.push_str(&format!(
                            r#"    ctx = data.get("context_window", {{}})
    used = ctx.get("total_input_tokens", 0) + ctx.get("total_output_tokens", 0)
    cap = ctx.get("context_window_size", 200000)
    def fmt_tokens(n):
        if n >= 1000000: return f"{{n/1000000:.1f}}M"
        if n >= 1000: return f"{{n/1000:.0f}}k"
        return str(n)
    parts.append(f"{color_start}{label_prefix}{{fmt_tokens(used)}}/{{fmt_tokens(cap)}}{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                    "bar" => {
                        parts_code.push_str(&format!(
                            r#"    ctx = data.get("context_window", {{}})
    pct = ctx.get("used_percentage", 0)
    filled = int(pct / 100 * 6)
    filled_bar = "\u2588" * filled
    unfilled_bar = "\u2591" * (6 - filled)
    parts.append(f"{color_start}{label_prefix}[{{filled_bar}}\033[38;2;128;128;128m{{unfilled_bar}}{color_start}] {{pct:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                    _ => {
                        // percentage
                        parts_code.push_str(&format!(
                            r#"    pct = data.get("context_window", {{}}).get("used_percentage", 0)
    parts.append(f"{color_start}{label_prefix}{{pct:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                }
            }
            "cwd" => {
                let fmt = seg.format.as_deref().unwrap_or("basename");
                let extract = match fmt {
                    "full" => {
                        r#"cwd = data.get("workspace", {}).get("current_dir", "") or data.get("cwd", "")"#
                    }
                    "short" => {
                        r#"import os
    cwd = data.get("workspace", {}).get("current_dir", "") or data.get("cwd", "")
    home = os.path.expanduser("~")
    if cwd.startswith(home):
        cwd = "~" + cwd[len(home):]"#
                    }
                    _ => {
                        // basename
                        r#"import os
    cwd = os.path.basename(data.get("workspace", {}).get("current_dir", "") or data.get("cwd", ""))"#
                    }
                };
                parts_code.push_str(&format!(
                    r#"    {extract}
    if cwd:
        parts.append(f"{color_start}{label_prefix}{{cwd}}{color_end}")
"#,
                    extract = extract,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "tokens_in" => {
                let fmt = seg.format.as_deref().unwrap_or("compact");
                let format_fn = if fmt == "full" {
                    "str(tokens)"
                } else {
                    r#"f"{tokens/1000:.0f}k" if tokens >= 1000 else str(tokens)"#
                };
                parts_code.push_str(&format!(
                    r#"    tokens = data.get("context_window", {{}}).get("total_input_tokens", 0)
    formatted = {format_fn}
    parts.append(f"{color_start}{label_prefix}{{formatted}}{color_end}")
"#,
                    format_fn = format_fn,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "tokens_out" => {
                let fmt = seg.format.as_deref().unwrap_or("compact");
                let format_fn = if fmt == "full" {
                    "str(tokens)"
                } else {
                    r#"f"{tokens/1000:.0f}k" if tokens >= 1000 else str(tokens)"#
                };
                parts_code.push_str(&format!(
                    r#"    tokens = data.get("context_window", {{}}).get("total_output_tokens", 0)
    formatted = {format_fn}
    parts.append(f"{color_start}{label_prefix}{{formatted}}{color_end}")
"#,
                    format_fn = format_fn,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "vim_mode" => {
                parts_code.push_str(&format!(
                    r#"    vim = data.get("vim", {{}})
    mode = vim.get("mode", "")
    if mode:
        parts.append(f"{color_start}{label_prefix}{{mode.upper()}}{color_end}")
"#,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "separator" => {
                let ch = seg.separator_char.as_deref().unwrap_or("|");
                parts_code.push_str(&format!(
                    "    parts.append(f\"{color_start}{sep}{color_end}\")\n",
                    color_start = color_start,
                    sep = ch,
                    color_end = color_end
                ));
            }
            "custom_text" => {
                let text = seg.custom_text.as_deref().unwrap_or("");
                parts_code.push_str(&format!(
                    "    parts.append(f\"{color_start}{text}{color_end}\")\n",
                    color_start = color_start,
                    text = text,
                    color_end = color_end
                ));
            }
            "context_remaining" => {
                let fmt = seg.format.as_deref().unwrap_or("percentage");
                match fmt {
                    "bar" => {
                        parts_code.push_str(&format!(
                            r#"    ctx = data.get("context_window", {{}})
    rem = ctx.get("remaining_percentage", 0) or 0
    filled = int(rem / 100 * 6)
    unfilled_bar = "\u2591" * (6 - filled)
    filled_bar = "\u2588" * filled
    parts.append(f"{color_start}{label_prefix}[\033[38;2;128;128;128m{{unfilled_bar}}{color_start}{{filled_bar}}] {{rem:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                    _ => {
                        parts_code.push_str(&format!(
                            r#"    rem = data.get("context_window", {{}}).get("remaining_percentage", 0) or 0
    parts.append(f"{color_start}{label_prefix}{{rem:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                }
            }
            "project_dir" => {
                let fmt = seg.format.as_deref().unwrap_or("basename");
                let extract = match fmt {
                    "full" => r#"pdir = data.get("workspace", {}).get("project_dir", "")"#,
                    _ => {
                        r#"import os
    pdir = os.path.basename(data.get("workspace", {}).get("project_dir", ""))"#
                    }
                };
                parts_code.push_str(&format!(
                    r#"    {extract}
    if pdir:
        parts.append(f"{color_start}{label_prefix}{{pdir}}{color_end}")
"#,
                    extract = extract,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "duration" => {
                let fmt = seg.format.as_deref().unwrap_or("short");
                let format_code = if fmt == "hms" {
                    r#"h, rem = divmod(secs, 3600)
    m, s = divmod(rem, 60)
    formatted = f"{h}:{m:02d}:{s:02d}""#
                } else {
                    r#"m, s = divmod(secs, 60)
    formatted = f"{m}m {s}s""#
                };
                parts_code.push_str(&format!(
                    r#"    dur_ms = data.get("cost", {{}}).get("total_duration_ms", 0) or 0
    secs = dur_ms // 1000
    {format_code}
    parts.append(f"{color_start}{label_prefix}{{formatted}}{color_end}")
"#,
                    format_code = format_code,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "api_duration" => {
                let fmt = seg.format.as_deref().unwrap_or("short");
                let format_code = if fmt == "hms" {
                    r#"h, rem = divmod(secs, 3600)
    m, s = divmod(rem, 60)
    formatted = f"{h}:{m:02d}:{s:02d}""#
                } else {
                    r#"m, s = divmod(secs, 60)
    formatted = f"{m}m {s}s""#
                };
                parts_code.push_str(&format!(
                    r#"    api_ms = data.get("cost", {{}}).get("total_api_duration_ms", 0) or 0
    secs = api_ms // 1000
    {format_code}
    parts.append(f"{color_start}{label_prefix}{{formatted}}{color_end}")
"#,
                    format_code = format_code,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "lines_changed" => {
                let fmt = seg.format.as_deref().unwrap_or("both");
                let format_code = if fmt == "net" {
                    r#"net = added - removed
    sign = "+" if net >= 0 else ""
    formatted = f"{sign}{net}""#
                } else {
                    r#"formatted = f"+{added} -{removed}""#
                };
                parts_code.push_str(&format!(
                    r#"    added = data.get("cost", {{}}).get("total_lines_added", 0) or 0
    removed = data.get("cost", {{}}).get("total_lines_removed", 0) or 0
    {format_code}
    parts.append(f"{color_start}{label_prefix}{{formatted}}{color_end}")
"#,
                    format_code = format_code,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "git_branch" => {
                parts_code.push_str(&format!(
                    r#"    import subprocess
    try:
        branch = subprocess.check_output(["git", "branch", "--show-current"], text=True, stderr=subprocess.DEVNULL).strip()
        if branch:
            parts.append(f"{color_start}{label_prefix}{{branch}}{color_end}")
    except Exception:
        pass
"#,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "git_status" => {
                let fmt = seg.format.as_deref().unwrap_or("compact");
                let format_code = if fmt == "verbose" {
                    r#"parts_git = []
        if staged: parts_git.append(f"{staged} staged")
        if modified: parts_git.append(f"{modified} modified")
        formatted = ", ".join(parts_git) if parts_git else "clean""#
                } else {
                    r#"parts_git = []
        if staged: parts_git.append(f"+{staged}")
        if modified: parts_git.append(f"~{modified}")
        formatted = " ".join(parts_git) if parts_git else "clean""#
                };
                parts_code.push_str(&format!(
                    r#"    import subprocess
    try:
        subprocess.check_output(["git", "rev-parse", "--git-dir"], stderr=subprocess.DEVNULL)
        staged_out = subprocess.check_output(["git", "diff", "--cached", "--numstat"], text=True, stderr=subprocess.DEVNULL).strip()
        modified_out = subprocess.check_output(["git", "diff", "--numstat"], text=True, stderr=subprocess.DEVNULL).strip()
        staged = len(staged_out.split("\n")) if staged_out else 0
        modified = len(modified_out.split("\n")) if modified_out else 0
        {format_code}
        parts.append(f"{color_start}{label_prefix}{{formatted}}{color_end}")
    except Exception:
        pass
"#,
                    format_code = format_code,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "session_id" => {
                let fmt = seg.format.as_deref().unwrap_or("short");
                let extract = if fmt == "full" {
                    r#"sid = data.get("session_id", "")"#
                } else {
                    r#"sid = data.get("session_id", "")[:8]"#
                };
                parts_code.push_str(&format!(
                    r#"    {extract}
    if sid:
        parts.append(f"{color_start}{label_prefix}{{sid}}{color_end}")
"#,
                    extract = extract,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "version" => {
                parts_code.push_str(&format!(
                    r#"    ver = data.get("version", "")
    if ver:
        parts.append(f"{color_start}{label_prefix}v{{ver}}{color_end}")
"#,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "agent_name" => {
                parts_code.push_str(&format!(
                    r#"    agent = data.get("agent", {{}})
    aname = agent.get("name", "") if agent else ""
    if aname:
        parts.append(f"{color_start}{label_prefix}{{aname}}{color_end}")
"#,
                    color_start = color_start,
                    label_prefix = label_prefix,
                    color_end = color_end
                ));
            }
            "five_hour_usage" => {
                needs_usage_api = true;
                let fmt = seg.format.as_deref().unwrap_or("text");
                match fmt {
                    "bar" => {
                        parts_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    if usage and usage.get("five_hour"):
        pct = usage["five_hour"].get("utilization", 0) or 0
        filled = int(pct / 100 * 6)
        filled_bar = "\u2588" * filled
        unfilled_bar = "\u2591" * (6 - filled)
        parts.append(f"{color_start}{label_prefix}[{{filled_bar}}\033[38;2;128;128;128m{{unfilled_bar}}{color_start}] {{pct:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                    "percent_only" => {
                        parts_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    if usage and usage.get("five_hour"):
        pct = usage["five_hour"].get("utilization", 0) or 0
        parts.append(f"{color_start}{label_prefix}{{pct:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                    _ => {
                        // text: "12% 3h20m"
                        parts_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    if usage and usage.get("five_hour"):
        fh = usage["five_hour"]
        pct = fh.get("utilization", 0) or 0
        reset_str = ""
        resets_at = fh.get("resets_at", "")
        if resets_at:
            try:
                from datetime import datetime, timezone
                reset_dt = datetime.fromisoformat(resets_at.replace("Z", "+00:00"))
                now = datetime.now(timezone.utc)
                remaining = max(0, int((reset_dt - now).total_seconds() / 60))
                h, m = divmod(remaining, 60)
                reset_str = f" {{h}}h{{m:02d}}m" if h else f" {{m}}m"
            except Exception:
                pass
        parts.append(f"{color_start}{label_prefix}{{pct:.0f}}%{{reset_str}}{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                }
            }
            "weekly_usage" => {
                needs_usage_api = true;
                let fmt = seg.format.as_deref().unwrap_or("text");
                match fmt {
                    "bar" => {
                        parts_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    if usage and usage.get("seven_day"):
        pct = usage["seven_day"].get("utilization", 0) or 0
        filled = int(pct / 100 * 6)
        filled_bar = "\u2588" * filled
        unfilled_bar = "\u2591" * (6 - filled)
        parts.append(f"{color_start}{label_prefix}[{{filled_bar}}\033[38;2;128;128;128m{{unfilled_bar}}{color_start}] {{pct:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                    "percent_only" => {
                        parts_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    if usage and usage.get("seven_day"):
        pct = usage["seven_day"].get("utilization", 0) or 0
        parts.append(f"{color_start}{label_prefix}{{pct:.0f}}%{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                    _ => {
                        // text: "45% wk 85%"
                        parts_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    if usage and usage.get("seven_day"):
        sd = usage["seven_day"]
        pct = sd.get("utilization", 0) or 0
        wk_str = ""
        resets_at = sd.get("resets_at", "")
        if resets_at:
            try:
                from datetime import datetime, timezone
                reset_dt = datetime.fromisoformat(resets_at.replace("Z", "+00:00"))
                now = datetime.now(timezone.utc)
                period_start = reset_dt.timestamp() - 7*24*3600
                elapsed = now.timestamp() - period_start
                wk_pct = min(100, max(0, int(elapsed / (7*24*3600) * 100)))
                wk_str = f" wk {{wk_pct}}%"
            except Exception:
                pass
        parts.append(f"{color_start}{label_prefix}{{pct:.0f}}%{{wk_str}}{color_end}")
"#,
                            color_start = color_start,
                            label_prefix = label_prefix,
                            color_end = color_end
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    let (lines_init, print_code) = if has_line_breaks {
        (
            "    lines = []",
            r#"    lines.append(" ".join(parts))
    print("\n".join(lines))"#,
        )
    } else {
        ("", r#"    print(" ".join(parts))"#)
    };

    let usage_api_code = if needs_usage_api {
        get_usage_api_code()
    } else {
        ""
    };

    format!(
        r#"#!/usr/bin/env python3
"""Auto-generated status line script by Claude Code Tool Manager."""
import sys
import json

# Ensure UTF-8 output on Windows (default cp1252 can't encode Unicode glyphs)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
{usage_api_code}
def main():
    try:
        data = json.loads(sys.stdin.read())
    except (json.JSONDecodeError, EOFError):
        data = {{}}

    parts = []
{lines_init}
{parts_code}
{print_code}

if __name__ == "__main__":
    main()
"#,
        usage_api_code = usage_api_code,
        lines_init = lines_init,
        parts_code = parts_code,
        print_code = print_code
    )
}

/// Get the default bg color for a segment type in powerline mode
fn get_powerline_default_bg(segment_type: &str) -> &'static str {
    match segment_type {
        "model" => "blue",
        "cost" => "green",
        "context" | "git_status" | "vim_mode" => "yellow",
        "context_remaining" | "lines_changed" | "weekly_usage" | "git_branch" => "green",
        "cwd" | "project_dir" => "blue",
        "tokens_in" | "tokens_out" => "magenta",
        "duration" | "api_duration" | "agent_name" | "five_hour_usage" => "cyan",
        "session_id" | "version" | "custom_text" => "gray",
        _ => "gray",
    }
}

/// Generate a Powerline-themed Python 3 script
fn generate_powerline_script(segments: &[StatusLineSegment], theme: &str) -> String {
    let enabled: Vec<&StatusLineSegment> = segments
        .iter()
        .filter(|s| s.enabled && s.segment_type != "separator" && s.segment_type != "line_break")
        .collect();

    let arrow = if theme == "powerline_round" {
        "\u{E0B4}"
    } else {
        "\u{E0B0}"
    };

    let mut needs_usage_api = false;

    // Build the segment data extraction code
    let mut extract_code = String::new();
    for (i, seg) in enabled.iter().enumerate() {
        let fg = seg.color.as_deref().unwrap_or("white");
        let bg = seg
            .bg_color
            .as_deref()
            .unwrap_or_else(|| get_powerline_default_bg(&seg.segment_type));
        let fg_num = get_ansi_fg_color_num(fg);
        let bg_num = get_ansi_bg_color_num(bg);
        let label = seg.label.as_deref().unwrap_or("");
        let label_prefix = if label.is_empty() {
            String::new()
        } else {
            format!("{} ", label)
        };

        let var_name = format!("seg_{}", i);
        match seg.segment_type.as_str() {
            "model" => {
                let format = seg.format.as_deref().unwrap_or("short");
                let extract = if format == "short" {
                    r#"name = data.get("model", {}).get("display_name", "") or data.get("model", {}).get("name", "")
    if "-" in name and len(name) > 10:
        parts_list = name.split("-")
        if len(parts_list) >= 2:
            name = parts_list[1]"#
                } else {
                    r#"name = data.get("model", {}).get("id", "") or data.get("model", {}).get("name", "")"#
                };
                extract_code.push_str(&format!(
                    r#"    {extract}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{name}}") if name else None
"#,
                    extract = extract,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "cost" => {
                let fmt = seg.format.as_deref().unwrap_or("$0.00");
                let decimals = if fmt.contains("0000") || fmt.contains("4") {
                    4
                } else {
                    2
                };
                extract_code.push_str(&format!(
                    r#"    cost = data.get("cost", {{}}).get("total_cost_usd", 0)
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}${{cost:.{decimals}f}}")
"#,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix,
                    decimals = decimals
                ));
            }
            "context" => {
                let fmt = seg.format.as_deref().unwrap_or("percentage");
                match fmt {
                    "fraction" => {
                        extract_code.push_str(&format!(
                            r#"    ctx = data.get("context_window", {{}})
    used = ctx.get("total_input_tokens", 0) + ctx.get("total_output_tokens", 0)
    cap = ctx.get("context_window_size", 200000)
    def fmt_tokens(n):
        if n >= 1000000: return f"{{n/1000000:.1f}}M"
        if n >= 1000: return f"{{n/1000:.0f}}k"
        return str(n)
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{fmt_tokens(used)}}/{{fmt_tokens(cap)}}")
"#,
                            var_name = var_name,
                            fg_num = fg_num,
                            bg_num = bg_num,
                            label_prefix = label_prefix
                        ));
                    }
                    "bar" => {
                        extract_code.push_str(&format!(
                            r#"    ctx = data.get("context_window", {{}})
    pct = ctx.get("used_percentage", 0)
    filled = int(pct / 100 * 6)
    filled_bar = "\u2588" * filled
    unfilled_bar = "\u2591" * (6 - filled)
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}[{{filled_bar}}\033[38;2;128;128;128;{bg_num}m{{unfilled_bar}}\033[{fg_num};{bg_num}m] {{pct:.0f}}%")
"#,
                            var_name = var_name, fg_num = fg_num, bg_num = bg_num, label_prefix = label_prefix
                        ));
                    }
                    _ => {
                        extract_code.push_str(&format!(
                            r#"    pct = data.get("context_window", {{}}).get("used_percentage", 0)
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{pct:.0f}}%")
"#,
                            var_name = var_name,
                            fg_num = fg_num,
                            bg_num = bg_num,
                            label_prefix = label_prefix
                        ));
                    }
                }
            }
            "context_remaining" => {
                let fmt = seg.format.as_deref().unwrap_or("percentage");
                match fmt {
                    "bar" => {
                        extract_code.push_str(&format!(
                            r#"    ctx = data.get("context_window", {{}})
    rem = ctx.get("remaining_percentage", 0) or 0
    filled = int(rem / 100 * 6)
    unfilled_bar = "\u2591" * (6 - filled)
    filled_bar = "\u2588" * filled
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}[\033[38;2;128;128;128;{bg_num}m{{unfilled_bar}}\033[{fg_num};{bg_num}m{{filled_bar}}] {{rem:.0f}}%")
"#,
                            var_name = var_name, fg_num = fg_num, bg_num = bg_num, label_prefix = label_prefix
                        ));
                    }
                    _ => {
                        extract_code.push_str(&format!(
                            r#"    rem = data.get("context_window", {{}}).get("remaining_percentage", 0) or 0
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{rem:.0f}}%")
"#,
                            var_name = var_name, fg_num = fg_num, bg_num = bg_num, label_prefix = label_prefix
                        ));
                    }
                }
            }
            "cwd" => {
                let fmt = seg.format.as_deref().unwrap_or("basename");
                let (import_line, extract) = match fmt {
                    "full" => (
                        "",
                        r#"cwd = data.get("workspace", {}).get("current_dir", "") or data.get("cwd", "")"#,
                    ),
                    "short" => (
                        "import os",
                        r#"cwd = data.get("workspace", {}).get("current_dir", "") or data.get("cwd", "")
    home = os.path.expanduser("~")
    if cwd.startswith(home):
        cwd = "~" + cwd[len(home):]"#,
                    ),
                    _ => (
                        "import os",
                        r#"cwd = os.path.basename(data.get("workspace", {}).get("current_dir", "") or data.get("cwd", ""))"#,
                    ),
                };
                if !import_line.is_empty() {
                    extract_code.push_str(&format!("    {}\n", import_line));
                }
                extract_code.push_str(&format!(
                    r#"    {extract}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{cwd}}") if cwd else None
"#,
                    extract = extract,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "project_dir" => {
                let fmt = seg.format.as_deref().unwrap_or("basename");
                let (import_line, extract) = match fmt {
                    "full" => (
                        "",
                        r#"pdir = data.get("workspace", {}).get("project_dir", "")"#,
                    ),
                    _ => (
                        "import os",
                        r#"pdir = os.path.basename(data.get("workspace", {}).get("project_dir", ""))"#,
                    ),
                };
                if !import_line.is_empty() {
                    extract_code.push_str(&format!("    {}\n", import_line));
                }
                extract_code.push_str(&format!(
                    r#"    {extract}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{pdir}}") if pdir else None
"#,
                    extract = extract,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "tokens_in" => {
                let fmt = seg.format.as_deref().unwrap_or("compact");
                let format_fn = if fmt == "full" {
                    "str(tokens)"
                } else {
                    r#"f"{tokens/1000:.0f}k" if tokens >= 1000 else str(tokens)"#
                };
                extract_code.push_str(&format!(
                    r#"    tokens = data.get("context_window", {{}}).get("total_input_tokens", 0)
    formatted = {format_fn}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{formatted}}")
"#,
                    format_fn = format_fn,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "tokens_out" => {
                let fmt = seg.format.as_deref().unwrap_or("compact");
                let format_fn = if fmt == "full" {
                    "str(tokens)"
                } else {
                    r#"f"{tokens/1000:.0f}k" if tokens >= 1000 else str(tokens)"#
                };
                extract_code.push_str(&format!(
                    r#"    tokens = data.get("context_window", {{}}).get("total_output_tokens", 0)
    formatted = {format_fn}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{formatted}}")
"#,
                    format_fn = format_fn,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "duration" => {
                let fmt = seg.format.as_deref().unwrap_or("short");
                let format_code = if fmt == "hms" {
                    r#"h, rem = divmod(secs, 3600)
    m, s = divmod(rem, 60)
    formatted = f"{h}:{m:02d}:{s:02d}""#
                } else {
                    r#"m, s = divmod(secs, 60)
    formatted = f"{m}m {s}s""#
                };
                extract_code.push_str(&format!(
                    r#"    dur_ms = data.get("cost", {{}}).get("total_duration_ms", 0) or 0
    secs = dur_ms // 1000
    {format_code}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{formatted}}")
"#,
                    format_code = format_code,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "api_duration" => {
                let fmt = seg.format.as_deref().unwrap_or("short");
                let format_code = if fmt == "hms" {
                    r#"h, rem = divmod(secs, 3600)
    m, s = divmod(rem, 60)
    formatted = f"{h}:{m:02d}:{s:02d}""#
                } else {
                    r#"m, s = divmod(secs, 60)
    formatted = f"{m}m {s}s""#
                };
                extract_code.push_str(&format!(
                    r#"    api_ms = data.get("cost", {{}}).get("total_api_duration_ms", 0) or 0
    secs = api_ms // 1000
    {format_code}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{formatted}}")
"#,
                    format_code = format_code,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "lines_changed" => {
                let fmt = seg.format.as_deref().unwrap_or("both");
                let format_code = if fmt == "net" {
                    r#"net = added - removed
    sign = "+" if net >= 0 else ""
    formatted = f"{sign}{net}""#
                } else {
                    r#"formatted = f"+{added} -{removed}""#
                };
                extract_code.push_str(&format!(
                    r#"    added = data.get("cost", {{}}).get("total_lines_added", 0) or 0
    removed = data.get("cost", {{}}).get("total_lines_removed", 0) or 0
    {format_code}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{formatted}}")
"#,
                    format_code = format_code,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "git_branch" => {
                extract_code.push_str(&format!(
                    r#"    import subprocess
    {var_name} = None
    try:
        branch = subprocess.check_output(["git", "branch", "--show-current"], text=True, stderr=subprocess.DEVNULL).strip()
        if branch:
            {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{branch}}")
    except Exception:
        pass
"#,
                    var_name = var_name, fg_num = fg_num, bg_num = bg_num, label_prefix = label_prefix
                ));
            }
            "git_status" => {
                let fmt = seg.format.as_deref().unwrap_or("compact");
                let format_code = if fmt == "verbose" {
                    r#"parts_git = []
        if staged: parts_git.append(f"{staged} staged")
        if modified: parts_git.append(f"{modified} modified")
        formatted = ", ".join(parts_git) if parts_git else "clean""#
                } else {
                    r#"parts_git = []
        if staged: parts_git.append(f"+{staged}")
        if modified: parts_git.append(f"~{modified}")
        formatted = " ".join(parts_git) if parts_git else "clean""#
                };
                extract_code.push_str(&format!(
                    r#"    import subprocess
    {var_name} = None
    try:
        subprocess.check_output(["git", "rev-parse", "--git-dir"], stderr=subprocess.DEVNULL)
        staged_out = subprocess.check_output(["git", "diff", "--cached", "--numstat"], text=True, stderr=subprocess.DEVNULL).strip()
        modified_out = subprocess.check_output(["git", "diff", "--numstat"], text=True, stderr=subprocess.DEVNULL).strip()
        staged = len(staged_out.split("\n")) if staged_out else 0
        modified = len(modified_out.split("\n")) if modified_out else 0
        {format_code}
        {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{formatted}}")
    except Exception:
        pass
"#,
                    format_code = format_code, var_name = var_name, fg_num = fg_num, bg_num = bg_num, label_prefix = label_prefix
                ));
            }
            "session_id" => {
                let fmt = seg.format.as_deref().unwrap_or("short");
                let extract = if fmt == "full" {
                    r#"sid = data.get("session_id", "")"#
                } else {
                    r#"sid = data.get("session_id", "")[:8]"#
                };
                extract_code.push_str(&format!(
                    r#"    {extract}
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{sid}}") if sid else None
"#,
                    extract = extract,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "version" => {
                extract_code.push_str(&format!(
                    r#"    ver = data.get("version", "")
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}v{{ver}}") if ver else None
"#,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "agent_name" => {
                extract_code.push_str(&format!(
                    r#"    agent = data.get("agent", {{}})
    aname = agent.get("name", "") if agent else ""
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{aname}}") if aname else None
"#,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "vim_mode" => {
                extract_code.push_str(&format!(
                    r#"    vim = data.get("vim", {{}})
    mode = vim.get("mode", "")
    {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{mode.upper()}}") if mode else None
"#,
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    label_prefix = label_prefix
                ));
            }
            "custom_text" => {
                let text = seg.custom_text.as_deref().unwrap_or("");
                extract_code.push_str(&format!(
                    "    {var_name} = (\"{fg_num}\", \"{bg_num}\", \"{text}\")\n",
                    var_name = var_name,
                    fg_num = fg_num,
                    bg_num = bg_num,
                    text = text
                ));
            }
            "five_hour_usage" => {
                needs_usage_api = true;
                let fmt = seg.format.as_deref().unwrap_or("text");
                match fmt {
                    "bar" => {
                        extract_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    {var_name} = None
    if usage and usage.get("five_hour"):
        pct = usage["five_hour"].get("utilization", 0) or 0
        filled = int(pct / 100 * 6)
        filled_bar = "\u2588" * filled
        unfilled_bar = "\u2591" * (6 - filled)
        {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}[{{filled_bar}}\033[38;2;128;128;128;{bg_num}m{{unfilled_bar}}\033[{fg_num};{bg_num}m] {{pct:.0f}}%")
"#,
                            var_name = var_name, fg_num = fg_num, bg_num = bg_num, label_prefix = label_prefix
                        ));
                    }
                    "percent_only" => {
                        extract_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    {var_name} = None
    if usage and usage.get("five_hour"):
        pct = usage["five_hour"].get("utilization", 0) or 0
        {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{pct:.0f}}%")
"#,
                            var_name = var_name,
                            fg_num = fg_num,
                            bg_num = bg_num,
                            label_prefix = label_prefix
                        ));
                    }
                    _ => {
                        extract_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    {var_name} = None
    if usage and usage.get("five_hour"):
        fh = usage["five_hour"]
        pct = fh.get("utilization", 0) or 0
        reset_str = ""
        resets_at = fh.get("resets_at", "")
        if resets_at:
            try:
                from datetime import datetime, timezone
                reset_dt = datetime.fromisoformat(resets_at.replace("Z", "+00:00"))
                now = datetime.now(timezone.utc)
                remaining = max(0, int((reset_dt - now).total_seconds() / 60))
                h, m = divmod(remaining, 60)
                reset_str = f" {{h}}h{{m:02d}}m" if h else f" {{m}}m"
            except Exception:
                pass
        {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{pct:.0f}}%{{reset_str}}")
"#,
                            var_name = var_name,
                            fg_num = fg_num,
                            bg_num = bg_num,
                            label_prefix = label_prefix
                        ));
                    }
                }
            }
            "weekly_usage" => {
                needs_usage_api = true;
                let fmt = seg.format.as_deref().unwrap_or("text");
                match fmt {
                    "bar" => {
                        extract_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    {var_name} = None
    if usage and usage.get("seven_day"):
        pct = usage["seven_day"].get("utilization", 0) or 0
        filled = int(pct / 100 * 6)
        filled_bar = "\u2588" * filled
        unfilled_bar = "\u2591" * (6 - filled)
        {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}[{{filled_bar}}\033[38;2;128;128;128;{bg_num}m{{unfilled_bar}}\033[{fg_num};{bg_num}m] {{pct:.0f}}%")
"#,
                            var_name = var_name, fg_num = fg_num, bg_num = bg_num, label_prefix = label_prefix
                        ));
                    }
                    "percent_only" => {
                        extract_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    {var_name} = None
    if usage and usage.get("seven_day"):
        pct = usage["seven_day"].get("utilization", 0) or 0
        {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{pct:.0f}}%")
"#,
                            var_name = var_name,
                            fg_num = fg_num,
                            bg_num = bg_num,
                            label_prefix = label_prefix
                        ));
                    }
                    _ => {
                        extract_code.push_str(&format!(
                            r#"    usage = _get_usage_data()
    {var_name} = None
    if usage and usage.get("seven_day"):
        sd = usage["seven_day"]
        pct = sd.get("utilization", 0) or 0
        wk_str = ""
        resets_at = sd.get("resets_at", "")
        if resets_at:
            try:
                from datetime import datetime, timezone
                reset_dt = datetime.fromisoformat(resets_at.replace("Z", "+00:00"))
                now = datetime.now(timezone.utc)
                period_start = reset_dt.timestamp() - 7*24*3600
                elapsed = now.timestamp() - period_start
                wk_pct = min(100, max(0, int(elapsed / (7*24*3600) * 100)))
                wk_str = f" wk {{wk_pct}}%"
            except Exception:
                pass
        {var_name} = ("{fg_num}", "{bg_num}", f"{label_prefix}{{pct:.0f}}%{{wk_str}}")
"#,
                            var_name = var_name,
                            fg_num = fg_num,
                            bg_num = bg_num,
                            label_prefix = label_prefix
                        ));
                    }
                }
            }
            _ => {
                extract_code.push_str(&format!("    {var_name} = None\n", var_name = var_name));
            }
        }
    }

    // Build the segment list
    let seg_vars: Vec<String> = (0..enabled.len()).map(|i| format!("seg_{}", i)).collect();
    let seg_list = seg_vars.join(", ");

    let usage_api_code = if needs_usage_api {
        get_usage_api_code()
    } else {
        ""
    };

    format!(
        r#"#!/usr/bin/env python3
"""Auto-generated Powerline status line script by Claude Code Tool Manager."""
import sys
import json

# Ensure UTF-8 output on Windows (default cp1252 can't encode Unicode glyphs)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
{usage_api_code}
ARROW = "{arrow}"

def render_powerline(segments):
    """Render segments as a Powerline-style status bar.
    Each segment is (fg_code, bg_code, text). None segments are skipped.
    fg_code is like '38;2;R;G;B', bg_code is like '48;2;R;G;B'."""
    active = [s for s in segments if s is not None]
    if not active:
        return ""
    output = ""
    for i, (fg, bg, text) in enumerate(active):
        output += f"\033[{{fg}};{{bg}}m {{text}} "
        if i < len(active) - 1:
            next_bg = active[i + 1][1]
            # Arrow: fg = current bg color as foreground, bg = next bg
            arrow_fg = bg.replace("48;2;", "38;2;")
            output += f"\033[{{arrow_fg}};{{next_bg}}m{{ARROW}}"
        else:
            # Last segment: arrow with current bg as fg, reset bg
            arrow_fg = bg.replace("48;2;", "38;2;")
            output += f"\033[0m\033[{{arrow_fg}}m{{ARROW}}\033[0m"
    return output

def main():
    try:
        data = json.loads(sys.stdin.read())
    except (json.JSONDecodeError, EOFError):
        data = {{}}

{extract_code}
    segments = [{seg_list}]
    print(render_powerline(segments))

if __name__ == "__main__":
    main()
"#,
        usage_api_code = usage_api_code,
        arrow = arrow,
        extract_code = extract_code,
        seg_list = seg_list
    )
}

/// Get the usage API helper code
fn get_usage_api_code() -> &'static str {
    r#"
import os
import time
import urllib.request

_USAGE_CACHE_PATH = os.path.join(os.environ.get("TMPDIR", os.environ.get("TEMP", "/tmp")), "cctm-usage-cache.json")
_USAGE_CACHE_MAX_AGE = 900  # 15 minutes

def _get_oauth_token():
    """Read OAuth token from Claude Code credentials."""
    creds_path = os.path.join(os.path.expanduser("~"), ".claude", ".credentials.json")
    try:
        with open(creds_path) as f:
            creds = json.load(f)
        oauth = creds.get("claudeAiOauth", {})
        token = oauth.get("accessToken", "") if isinstance(oauth, dict) else ""
        if token and token.startswith("sk-ant-oat"):
            return token
    except Exception:
        pass
    return None

def _get_usage_data():
    """Fetch usage data from Anthropic OAuth API with file-based caching."""
    # Check cache
    try:
        if os.path.exists(_USAGE_CACHE_PATH):
            age = time.time() - os.path.getmtime(_USAGE_CACHE_PATH)
            if age < _USAGE_CACHE_MAX_AGE:
                with open(_USAGE_CACHE_PATH) as f:
                    return json.load(f)
    except Exception:
        pass

    # Fetch fresh
    token = _get_oauth_token()
    if not token:
        return None
    try:
        req = urllib.request.Request(
            "https://api.anthropic.com/api/oauth/usage",
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
                "anthropic-beta": "oauth-2025-04-20",
            },
        )
        with urllib.request.urlopen(req, timeout=5) as resp:
            data = json.loads(resp.read())
        # Cache it
        with open(_USAGE_CACHE_PATH, "w") as f:
            json.dump(data, f)
        return data
    except Exception:
        # Return stale cache if available
        try:
            if os.path.exists(_USAGE_CACHE_PATH):
                with open(_USAGE_CACHE_PATH) as f:
                    return json.load(f)
        except Exception:
            pass
        return None

"#
}

/// Map a color name to its RGB values (matches SEGMENT_COLORS hex in TypeScript)
fn color_name_to_rgb(color: &str) -> (u8, u8, u8) {
    match color {
        "red" => (205, 49, 49),
        "green" => (13, 188, 121),
        "yellow" => (229, 229, 16),
        "blue" => (36, 114, 200),
        "magenta" => (188, 63, 188),
        "cyan" => (17, 168, 205),
        "white" => (229, 229, 229),
        "bright_red" => (241, 76, 76),
        "bright_green" => (35, 209, 139),
        "bright_yellow" => (245, 245, 67),
        "bright_blue" => (59, 142, 234),
        "bright_magenta" => (214, 112, 214),
        "bright_cyan" => (41, 184, 219),
        "bright_white" => (255, 255, 255),
        "gray" | "grey" => (128, 128, 128),
        _ => (229, 229, 229), // default white
    }
}

/// Get ANSI 24-bit true color foreground escape code for a color name
fn get_ansi_color_code(color: &str) -> String {
    let (r, g, b) = color_name_to_rgb(color);
    format!("\\033[38;2;{};{};{}m", r, g, b)
}

/// Get ANSI 24-bit true color background code parameter for a color name (e.g. "48;2;R;G;B")
fn get_ansi_bg_color_num(color: &str) -> String {
    let (r, g, b) = color_name_to_rgb(color);
    format!("48;2;{};{};{}", r, g, b)
}

/// Get ANSI 24-bit true color foreground code parameter for a color name (e.g. "38;2;R;G;B")
fn get_ansi_fg_color_num(color: &str) -> String {
    let (r, g, b) = color_name_to_rgb(color);
    format!("38;2;{};{};{}", r, g, b)
}

/// Build combined ANSI 24-bit true color fg+bg escape code
fn build_color_code(fg: &str, bg: Option<&str>) -> String {
    let (fr, fg_g, fb) = color_name_to_rgb(fg);
    match bg {
        Some(bg_color) => {
            let (br, bg_g, bb) = color_name_to_rgb(bg_color);
            format!(
                "\\033[38;2;{};{};{};48;2;{};{};{}m",
                fr, fg_g, fb, br, bg_g, bb
            )
        }
        None => format!("\\033[38;2;{};{};{}m", fr, fg_g, fb),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_script_from_segments() {
        let segments = vec![
            StatusLineSegment {
                id: "1".to_string(),
                segment_type: "model".to_string(),
                enabled: true,
                label: None,
                format: Some("short".to_string()),
                color: Some("cyan".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 0,
            },
            StatusLineSegment {
                id: "2".to_string(),
                segment_type: "separator".to_string(),
                enabled: true,
                label: None,
                format: None,
                color: Some("gray".to_string()),
                bg_color: None,
                separator_char: Some("|".to_string()),
                custom_text: None,
                position: 1,
            },
            StatusLineSegment {
                id: "3".to_string(),
                segment_type: "cost".to_string(),
                enabled: true,
                label: Some("$".to_string()),
                format: Some("$0.00".to_string()),
                color: Some("green".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 2,
            },
        ];

        let script = generate_script_from_segments(&segments);
        assert!(script.contains("#!/usr/bin/env python3"));
        assert!(script.contains("json.loads"));
        assert!(script.contains("model"));
        assert!(script.contains("total_cost_usd"));
    }

    #[test]
    fn test_generate_script_disabled_segments() {
        let segments = vec![StatusLineSegment {
            id: "1".to_string(),
            segment_type: "model".to_string(),
            enabled: false,
            label: None,
            format: None,
            color: None,
            bg_color: None,
            separator_char: None,
            custom_text: None,
            position: 0,
        }];

        let script = generate_script_from_segments(&segments);
        assert!(!script.contains("model"));
    }

    #[test]
    fn test_get_ansi_color_code() {
        assert_eq!(get_ansi_color_code("red"), "\\033[38;2;205;49;49m");
        assert_eq!(get_ansi_color_code("cyan"), "\\033[38;2;17;168;205m");
        assert_eq!(get_ansi_color_code("unknown"), "\\033[38;2;229;229;229m");
    }

    #[test]
    fn test_color_name_to_rgb() {
        assert_eq!(color_name_to_rgb("blue"), (36, 114, 200));
        assert_eq!(color_name_to_rgb("gray"), (128, 128, 128));
    }

    #[test]
    fn test_generate_script_multiple_segments() {
        let segments = vec![
            StatusLineSegment {
                id: "1".to_string(),
                segment_type: "model".to_string(),
                enabled: true,
                label: Some("Model".to_string()),
                format: Some("short".to_string()),
                color: Some("cyan".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 0,
            },
            StatusLineSegment {
                id: "2".to_string(),
                segment_type: "context".to_string(),
                enabled: true,
                label: Some("Ctx".to_string()),
                format: Some("percentage".to_string()),
                color: Some("yellow".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 1,
            },
            StatusLineSegment {
                id: "3".to_string(),
                segment_type: "cost".to_string(),
                enabled: true,
                label: None,
                format: Some("$0.0000".to_string()),
                color: Some("green".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 2,
            },
        ];

        let script = generate_script_from_segments(&segments);
        assert!(script.contains("model"));
        assert!(script.contains("used_percentage"));
        assert!(script.contains("total_cost_usd"));
        // Should have 4 decimal places for cost
        assert!(script.contains(":.4f"));
    }

    #[test]
    fn test_generate_script_with_powerline_theme() {
        let segments = vec![StatusLineSegment {
            id: "1".to_string(),
            segment_type: "model".to_string(),
            enabled: true,
            label: None,
            format: Some("short".to_string()),
            color: Some("white".to_string()),
            bg_color: Some("blue".to_string()),
            separator_char: None,
            custom_text: None,
            position: 0,
        }];

        let script = generate_script_from_segments_with_theme(&segments, "powerline");
        assert!(script.contains("Powerline"));
        assert!(script.contains("render_powerline"));
        assert!(script.contains("ARROW"));
    }

    #[test]
    fn test_generate_script_empty_segments_produces_valid_output() {
        let segments: Vec<StatusLineSegment> = vec![];
        let script = generate_script_from_segments(&segments);
        assert!(script.contains("#!/usr/bin/env python3"));
        assert!(script.contains("def main():"));
    }

    #[test]
    fn test_build_color_code_fg_only() {
        let code = build_color_code("red", None);
        assert!(code.contains("205;49;49"));
        assert!(!code.contains("48;2;"));
    }

    #[test]
    fn test_build_color_code_fg_and_bg() {
        let code = build_color_code("red", Some("blue"));
        assert!(code.contains("205;49;49"));
        assert!(code.contains("48;2;36;114;200"));
    }

    #[test]
    fn test_get_ansi_bg_color_num() {
        let bg = get_ansi_bg_color_num("green");
        assert_eq!(bg, "48;2;13;188;121");
    }

    #[test]
    fn test_get_ansi_fg_color_num() {
        let fg = get_ansi_fg_color_num("magenta");
        assert_eq!(fg, "38;2;188;63;188");
    }
}
