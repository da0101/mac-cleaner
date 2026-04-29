#!/usr/bin/env python3
"""
Mac Cleaner AI Assistant — interactive terminal chat that analyzes
your running processes, asks what you're working on, and intelligently
suggests what to kill to free RAM. You approve, it kills.

Usage: python3 ai_cleaner.py
"""

import os
import sys
import json
import subprocess
import signal
from pathlib import Path
from datetime import datetime
from dotenv import load_dotenv

# Load .env from same directory
load_dotenv(Path(__file__).parent / ".env")

from openai import OpenAI

# ── Config ──
HOME = Path.home()
client = OpenAI(api_key=os.getenv("OPENAI_API_KEY") or os.getenv("OPNEN_AI_API_KEY"))
MODEL = "gpt-4o-mini"  # Fast and cheap for this use case
PROTECTED_PROCESS_KEYWORDS = (
    "terminal", "iterm", "zsh", "bash", "claude", "codex", "gemini",
    "visual studio code", "code helper", "vscode", "cursor", "node", "python",
)

# ANSI colors
RED = "\033[91m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
BLUE = "\033[94m"
CYAN = "\033[96m"
MAGENTA = "\033[95m"
BOLD = "\033[1m"
DIM = "\033[2m"
RESET = "\033[0m"


def fmt_size(size_bytes):
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 ** 2:
        return f"{size_bytes / 1024:.1f} KB"
    elif size_bytes < 1024 ** 3:
        return f"{size_bytes / 1024**2:.1f} MB"
    else:
        return f"{size_bytes / 1024**3:.2f} GB"


def get_system_info():
    """Get RAM and disk info."""
    info = {}
    try:
        out = subprocess.run(["vm_stat"], capture_output=True, text=True, timeout=5)
        if out.returncode == 0:
            pages = {}
            for line in out.stdout.strip().split("\n")[1:]:
                if ":" in line:
                    key, val = line.split(":", 1)
                    try:
                        pages[key.strip()] = int(val.strip().rstrip("."))
                    except ValueError:
                        pass
            ps = 16384
            free = pages.get("Pages free", 0) * ps
            active = pages.get("Pages active", 0) * ps
            inactive = pages.get("Pages inactive", 0) * ps
            wired = pages.get("Pages wired down", 0) * ps
            compressed = pages.get("Pages occupied by compressor", 0) * ps
            speculative = pages.get("Pages speculative", 0) * ps
            total = free + active + inactive + wired + compressed + speculative
            info["free_ram"] = fmt_size(free)
            info["free_ram_bytes"] = free
            info["total_ram"] = fmt_size(total)
            info["app_memory"] = fmt_size(active)
            info["wired"] = fmt_size(wired)
            info["compressed"] = fmt_size(compressed)
            info["cached"] = fmt_size(inactive)
    except Exception:
        pass

    try:
        out = subprocess.run(["df", "-h", "/"], capture_output=True, text=True, timeout=5)
        if out.returncode == 0:
            parts = out.stdout.strip().split("\n")[1].split()
            info["disk_total"] = parts[1]
            info["disk_available"] = parts[3]
            info["disk_percent_used"] = parts[4]
    except Exception:
        pass

    return info


def get_all_processes():
    """Get all running processes with memory usage."""
    try:
        out = subprocess.run(
            ["ps", "aux", "-m"],
            capture_output=True, text=True, timeout=5
        )
        if out.returncode != 0:
            return []

        procs = []
        for line in out.stdout.strip().split("\n")[1:]:
            parts = line.split(None, 10)
            if len(parts) < 11:
                continue
            try:
                mem_pct = float(parts[3])
                cpu_pct = float(parts[2])
                rss_kb = int(parts[5])
                pid = parts[1]
                command = parts[10]
                name = command.split("/")[-1][:50]

                if rss_kb > 5120:  # > 5MB
                    procs.append({
                        "pid": pid,
                        "name": name,
                        "command": command[:100],
                        "mem_mb": round(rss_kb / 1024),
                        "mem_pct": mem_pct,
                        "cpu_pct": cpu_pct,
                    })
            except (ValueError, IndexError):
                continue

        # Group by name
        grouped = {}
        for p in procs:
            key = p["name"]
            if key not in grouped:
                grouped[key] = {
                    "name": key,
                    "total_mem_mb": 0,
                    "total_cpu_pct": 0,
                    "count": 0,
                    "pids": [],
                    "command": p["command"],
                }
            grouped[key]["total_mem_mb"] += p["mem_mb"]
            grouped[key]["total_cpu_pct"] += p["cpu_pct"]
            grouped[key]["count"] += 1
            grouped[key]["pids"].append(p["pid"])

        result = sorted(grouped.values(), key=lambda x: x["total_mem_mb"], reverse=True)
        return result[:30]
    except Exception:
        return []


def kill_processes(pids, name=""):
    """Kill processes by PID list."""
    if is_protected_name(name):
        return [], list(pids)

    killed = []
    failed = []
    for pid in pids:
        try:
            result = subprocess.run(["kill", "-9", str(pid)], capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                killed.append(pid)
            else:
                failed.append(pid)
        except Exception:
            failed.append(pid)
    return killed, failed


def purge_ram():
    """Purge inactive RAM."""
    try:
        result = subprocess.run(["purge"], capture_output=True, text=True, timeout=15)
        if result.returncode == 0:
            return True
        result = subprocess.run(["sudo", "-n", "purge"], capture_output=True, text=True, timeout=15)
        return result.returncode == 0
    except Exception:
        return False


def quit_app(app_name):
    """Gracefully quit a macOS app."""
    if is_protected_name(app_name):
        return False

    try:
        result = subprocess.run(
            ["osascript", "-e", f'tell application "{app_name}" to quit'],
            capture_output=True, text=True, timeout=10
        )
        return result.returncode == 0
    except Exception:
        return False


def is_protected_name(name):
    """Return True for developer tools and shells that require manual approval outside AI tools."""
    lowered = (name or "").lower()
    return any(keyword in lowered for keyword in PROTECTED_PROCESS_KEYWORDS)


# ── AI Tools ──
TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "get_system_status",
            "description": "Get current system RAM, disk usage, and memory breakdown",
            "parameters": {"type": "object", "properties": {}, "required": []},
        },
    },
    {
        "type": "function",
        "function": {
            "name": "get_running_processes",
            "description": "Get all running processes grouped by name, sorted by memory usage. Shows PID, name, memory MB, CPU %, and count.",
            "parameters": {"type": "object", "properties": {}, "required": []},
        },
    },
    {
        "type": "function",
        "function": {
            "name": "kill_process",
            "description": "Force kill a process by its PID(s). Use this when the user approves killing a process.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of PIDs to kill",
                    },
                    "process_name": {
                        "type": "string",
                        "description": "Name of the process being killed (for logging)",
                    },
                },
                "required": ["pids", "process_name"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "quit_application",
            "description": "Gracefully quit a macOS application by name (e.g. 'Google Chrome', 'Postman', 'Docker'). Prefer this over kill for GUI apps.",
            "parameters": {
                "type": "object",
                "properties": {
                    "app_name": {
                        "type": "string",
                        "description": "The macOS application name as it appears in the menu bar",
                    },
                },
                "required": ["app_name"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "purge_memory",
            "description": "Purge inactive/cached RAM to free up memory immediately",
            "parameters": {"type": "object", "properties": {}, "required": []},
        },
    },
]

SYSTEM_PROMPT = """You are Mac Cleaner AI — a friendly, opinionated system performance assistant running locally on the user's M3 MacBook Pro (18GB RAM).

Your job:
1. First, ALWAYS call get_system_status and get_running_processes to see the current state
2. Ask the user what they're currently working on (what apps/tools they need right now)
3. Based on their answer, classify every running process as:
   - 🟢 KEEP — needed for their current work
   - 🟡 CHECK — might not be needed, ask the user
   - 🔴 KILL — definitely not needed, wasting RAM

4. Present your findings in a clear, color-coded table format
5. Suggest what to close and how much RAM it would free
6. Wait for the user to approve before killing anything
7. After killing, purge RAM and show the before/after

Rules:
- NEVER kill system processes (kernel_task, WindowServer, launchd, mds, loginwindow, Finder, Dock, SystemUIServer, etc.)
- NEVER kill Terminal, iTerm, zsh, bash, this Python process, VS Code, Cursor, Claude Code, Codex, Gemini, node, or python jobs through tools
- For protected developer tools, explain the RAM impact and ask the user to close them manually if they agree
- For GUI apps (Chrome, Postman, Slack, etc.), prefer quit_application over kill_process — it's cleaner
- For background processes (node, python scripts, etc.), use kill_process
- Always show how much RAM will be freed
- Be direct and opinionated — if something is clearly wasting RAM, say so
- Group related processes (e.g., all Chrome helper processes = "Chrome")
- After any action, call get_system_status again to show the result
- Keep responses concise — this is a terminal, not an essay

When presenting the process table, format it like:
🔴 KILL   Chrome (5 procs)     542 MB   "You said you're not browsing"
🟡 CHECK  Docker (3 procs)     312 MB   "Are you running containers right now?"
🟢 KEEP   VS Code (2 procs)    180 MB   "You need this for coding"

Always end suggestions with a total: "Total freeable: ~1.2 GB — want me to clean up?"
"""


def handle_tool_call(tool_call):
    """Execute a tool call and return the result."""
    name = tool_call.function.name
    args = json.loads(tool_call.function.arguments) if tool_call.function.arguments else {}

    if name == "get_system_status":
        return json.dumps(get_system_info())

    elif name == "get_running_processes":
        procs = get_all_processes()
        return json.dumps(procs, indent=2)

    elif name == "kill_process":
        pids = args.get("pids", [])
        proc_name = args.get("process_name", "unknown")
        killed, failed = kill_processes(pids, proc_name)
        return json.dumps({
            "killed": killed,
            "failed": failed,
            "message": f"Killed {len(killed)}/{len(pids)} {proc_name} processes"
        })

    elif name == "quit_application":
        app_name = args.get("app_name", "")
        success = quit_app(app_name)
        return json.dumps({
            "success": success,
            "message": f"{'Quit' if success else 'Failed to quit'} {app_name}"
        })

    elif name == "purge_memory":
        success = purge_ram()
        info = get_system_info()
        return json.dumps({
            "success": success,
            "free_ram_after": info.get("free_ram", "unknown"),
            "message": "RAM purged" if success else "Purge failed — needs sudo"
        })

    return json.dumps({"error": f"Unknown tool: {name}"})


def print_ai(text):
    """Print AI response with formatting."""
    print(f"\n{CYAN}{BOLD}  AI:{RESET} ", end="")
    # Simple markdown-ish formatting for terminal
    for line in text.split("\n"):
        if line.startswith("🔴") or line.startswith("🟡") or line.startswith("🟢"):
            print(f"  {line}")
        elif line.startswith("#"):
            print(f"  {BOLD}{line.lstrip('#').strip()}{RESET}")
        elif line.startswith("---"):
            print(f"  {DIM}{'─' * 56}{RESET}")
        elif line.startswith(">"):
            print(f"  {DIM}{line.lstrip('> ')}{RESET}")
        elif "Total freeable" in line or "freed" in line.lower():
            print(f"  {GREEN}{BOLD}{line}{RESET}")
        else:
            print(f"  {line}")
    print()


def main():
    print(f"""
{CYAN}{BOLD}
  ╔══════════════════════════════════════════════════╗
  ║         MAC CLEANER AI ASSISTANT                 ║
  ║   Interactive memory manager powered by AI       ║
  ║                                                  ║
  ║   I'll analyze your processes, ask what you're   ║
  ║   working on, and help you free up RAM.          ║
  ║                                                  ║
  ║   Type 'quit' or Ctrl+C to exit                  ║
  ╚══════════════════════════════════════════════════╝
{RESET}""")

    # Conversation history
    messages = [{"role": "system", "content": SYSTEM_PROMPT}]

    # Start with AI analyzing the system
    messages.append({
        "role": "user",
        "content": "Hey! My Mac is running slow. Can you check what's eating my RAM and help me clean up?"
    })

    while True:
        try:
            # Call AI
            sys.stdout.write(f"  {DIM}Thinking...{RESET}\r")
            sys.stdout.flush()

            response = client.chat.completions.create(
                model=MODEL,
                messages=messages,
                tools=TOOLS,
                tool_choice="auto",
                temperature=0.3,
            )

            msg = response.choices[0].message
            messages.append(msg)

            # Handle tool calls
            while msg.tool_calls:
                for tool_call in msg.tool_calls:
                    fname = tool_call.function.name
                    sys.stdout.write(f"  {DIM}Running {fname}...{RESET}          \r")
                    sys.stdout.flush()

                    result = handle_tool_call(tool_call)
                    messages.append({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "content": result,
                    })

                # Get next response
                response = client.chat.completions.create(
                    model=MODEL,
                    messages=messages,
                    tools=TOOLS,
                    tool_choice="auto",
                    temperature=0.3,
                )
                msg = response.choices[0].message
                messages.append(msg)

            # Print AI response
            if msg.content:
                sys.stdout.write("                              \r")  # Clear "Thinking..."
                print_ai(msg.content)

            # Get user input
            try:
                user_input = input(f"  {BOLD}You:{RESET} ").strip()
            except EOFError:
                break

            if user_input.lower() in ("quit", "exit", "q", "bye"):
                print(f"\n  {DIM}Bye! Keep your Mac clean.{RESET}\n")
                break

            if not user_input:
                continue

            messages.append({"role": "user", "content": user_input})

        except KeyboardInterrupt:
            print(f"\n\n  {DIM}Bye! Keep your Mac clean.{RESET}\n")
            break
        except Exception as e:
            print(f"\n  {RED}Error: {e}{RESET}\n")
            # Continue conversation
            continue


if __name__ == "__main__":
    main()
