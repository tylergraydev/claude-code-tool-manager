# Claude Code Hooks Reference

## Overview

Claude Code hooks are shell commands that execute in response to specific events. They allow you to customize behavior, play notifications, validate actions, and more.

## Hook Events

| Event | When It Fires | Use Case |
|-------|---------------|----------|
| `PreToolUse` | Before a tool runs | Validate/modify tool inputs |
| `PermissionRequest` | When Claude asks for permission to use a tool | **Play notification sound** |
| `PostToolUse` | After a tool completes | Log results, cleanup |
| `Notification` | System notifications (idle, auth, etc.) | Alert user |
| `UserPromptSubmit` | When user submits a prompt | Validate/modify input |
| `Stop` | When Claude finishes responding | **Play completion sound** |
| `SubagentStop` | When a background agent completes | **Play completion sound** |
| `PreCompact` | Before context compaction | Custom compaction logic |
| `SessionStart` | When a session begins | Setup tasks |
| `SessionEnd` | When a session ends | Cleanup tasks |

## Configuration Location

Hooks are configured in `~/.claude/settings.json` under the `hooks` key.

## JSON Payload Structure

### PermissionRequest / PreToolUse / PostToolUse
```json
{
  "session_id": "abc123",
  "transcript_path": "/Users/.../.claude/projects/.../session.jsonl",
  "cwd": "/Users/...",
  "hook_event_name": "PermissionRequest",
  "tool_name": "Bash",
  "tool_input": {
    "command": "ls -la",
    "description": "List files"
  }
}
```

### Notification
```json
{
  "session_id": "abc123",
  "transcript_path": "/Users/.../.claude/projects/.../session.jsonl",
  "cwd": "/Users/...",
  "hook_event_name": "Notification",
  "message": "Claude needs your permission to use Bash",
  "notification_type": "permission_prompt"
}
```

### Notification Types (for Notification event)
- `permission_prompt` - Permission requests
- `idle_prompt` - Claude waiting for input (60+ seconds idle)
- `auth_success` - Authentication successful
- `elicitation_dialog` - MCP tool needs input

### Stop / SubagentStop
```json
{
  "session_id": "abc123",
  "transcript_path": "/Users/.../.claude/projects/.../session.jsonl",
  "cwd": "/Users/...",
  "hook_event_name": "Stop",
  "stop_reason": "end_turn"
}
```

## Example Configuration

```json
{
  "hooks": {
    "PermissionRequest": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 ~/.claude/hooks/notification-hook.py",
            "timeout": 5
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 ~/.claude/hooks/notification-hook.py",
            "timeout": 5
          }
        ]
      }
    ],
    "SubagentStop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 ~/.claude/hooks/notification-hook.py",
            "timeout": 5
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "permission_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "python3 ~/.claude/hooks/notification-hook.py",
            "timeout": 5
          }
        ]
      },
      {
        "matcher": "idle_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "python3 ~/.claude/hooks/notification-hook.py",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

## Hook Configuration Options

```json
{
  "matcher": "optional_regex_pattern",  // Filter when hook runs (for Notification types)
  "hooks": [
    {
      "type": "command",
      "command": "path/to/script.py",
      "timeout": 5  // Seconds before hook times out
    }
  ]
}
```

## Example Notification Hook Script

```python
#!/usr/bin/env python3
"""
Notification hook for Claude Code that plays different sounds based on event type.
"""
import json
import sys
import subprocess
import os

# Hook event sounds (PermissionRequest, Stop, SubagentStop, etc.)
EVENT_SOUNDS = {
    "PermissionRequest": "~/.claude/sounds/permission.wav",
    "Stop": "~/.claude/sounds/success.wav",
    "SubagentStop": "~/.claude/sounds/success.wav",
}

# Notification type sounds (for Notification hook events)
NOTIFICATION_SOUNDS = {
    "permission_prompt": "~/.claude/sounds/permission.wav",
    "idle_prompt": "~/.claude/sounds/idle.wav",
    "auth_success": "~/.claude/sounds/success.wav",
    "elicitation_dialog": "~/.claude/sounds/input.wav",
}

DEFAULT_SOUND = "~/.claude/sounds/notification.wav"

def play_sound(sound_path: str) -> bool:
    sound_path = os.path.expanduser(sound_path)
    if not os.path.exists(sound_path):
        return False

    try:
        if sys.platform == "darwin":
            subprocess.run(["afplay", sound_path], check=True, capture_output=True)
        elif sys.platform.startswith("linux"):
            try:
                subprocess.run(["paplay", sound_path], check=True, capture_output=True)
            except FileNotFoundError:
                subprocess.run(["aplay", sound_path], check=True, capture_output=True)
        elif sys.platform == "win32":
            subprocess.run([
                "powershell", "-c",
                f"(New-Object Media.SoundPlayer '{sound_path}').PlaySync()"
            ], check=True, capture_output=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False

def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError:
        sys.exit(1)

    hook_event = input_data.get("hook_event_name", "")
    notification_type = input_data.get("notification_type", "")

    if hook_event in EVENT_SOUNDS:
        sound_file = EVENT_SOUNDS[hook_event]
    elif notification_type in NOTIFICATION_SOUNDS:
        sound_file = NOTIFICATION_SOUNDS[notification_type]
    else:
        sound_file = DEFAULT_SOUND

    if play_sound(sound_file):
        print(json.dumps({"suppressOutput": True}))
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()
```

## macOS System Sounds

Available at `/System/Library/Sounds/`:
- Basso.aiff - Deep bass alert
- Blow.aiff
- Bottle.aiff
- Frog.aiff
- Funk.aiff - Funky notification
- Glass.aiff - Clear ding
- Hero.aiff - Triumphant
- Morse.aiff
- Ping.aiff
- Pop.aiff
- Purr.aiff
- Sosumi.aiff - Classic Mac alert
- Submarine.aiff
- Tink.aiff

## Hook Exit Codes

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success - stdout shown in transcript mode (CTRL-R) |
| 1 | Non-blocking error - logged but doesn't stop Claude |
| 2 | Blocking error - stderr fed back to Claude to handle |

## Hook Output

Hooks can return JSON to control behavior:
```json
{
  "suppressOutput": true  // Don't show hook output in transcript
}
```

## Key Learnings

1. **PermissionRequest vs Notification**: The `PermissionRequest` hook fires when Claude asks to use a tool. The `Notification` hook with `permission_prompt` matcher is for system-level notifications, which may not always fire.

2. **Auto-approved commands don't trigger PermissionRequest**: If a command is in the user's allowed list, no permission is requested, so no hook fires.

3. **Reload required**: After modifying `~/.claude/settings.json`, you may need to reload Claude Code for hooks to take effect.

4. **Timeout**: Hooks have a timeout (default varies). Set explicitly with `"timeout": 5` for 5 seconds.

5. **Input is JSON via stdin**: Hooks receive event data as JSON on stdin.

## Management Feature Ideas

A hook management tool could:
1. List all configured hooks
2. Add/remove hooks for specific events
3. Enable/disable hooks without deleting them
4. Test hooks with sample payloads
5. Manage sound file associations
6. Preview available system sounds
7. Configure per-project vs global hooks
8. Show hook execution logs

## Documentation Links

- https://code.claude.com/docs/en/hooks
- https://docs.claude.com/en/docs/claude-code/hooks
