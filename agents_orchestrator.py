"""
Multi-Agent Orchestrator for Rust Todo App
Starts 3 parallel conversations, each working on independent tasks.
"""
import json, os, sys, time, urllib.request, threading

AGENT_SERVER = os.environ.get("AGENT_SERVER_URL", "http://127.0.0.1:18000").rstrip("/")
SESSION_KEY = os.environ.get("SESSION_API_KEY", "")

def api(method, path, body=None):
    url = f"{AGENT_SERVER}{path}"
    data = json.dumps(body).encode() if body else None
    req = urllib.request.Request(url, data=data, method=method, headers={
        "X-Session-API-Key": SESSION_KEY,
        "Content-Type": "application/json",
    })
    try:
        with urllib.request.urlopen(req) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        return {"error": e.code, "body": e.read().decode()}

AGENTS = [
    {
        "name": "Tests",
        "task": (
            "You are working on the Rust TUI todo app at /home/openhands/workspace/project/rust-todo.\n\n"
            "Read src/main.rs to understand the Todo, Priority, TodoList, and App structures.\n"
            "Then create unit tests in src/main.rs (add a #[cfg(test)] module at the bottom).\n"
            "Test at least: Todo creation, priority cycling, add/delete from TodoList, toggle done.\n"
            "Run 'cargo test' to verify all tests pass.\n"
            "When done, reply with exactly: TASK_DONE"
        ),
    },
    {
        "name": "CI/CD",
        "task": (
            "Create a GitHub Actions CI workflow for the Rust project at /home/openhands/workspace/project/rust-todo.\n\n"
            "Create .github/workflows/rust.yml with:\n"
            "- Trigger on push to main and PRs to main\n"
            "- Build with cargo build --release\n"
            "- Run tests with cargo test\n"
            "- Check formatting with cargo fmt --check\n"
            "When done, reply with exactly: TASK_DONE"
        ),
    },
    {
        "name": "Docs",
        "task": (
            "Create comprehensive documentation for the Rust TUI todo app at /home/openhands/workspace/project/rust-todo.\n\n"
            "Read src/main.rs and Cargo.toml to understand the app.\n"
            "Create a docs/ directory with:\n"
            "- docs/README.md: full usage guide with all keyboard shortcuts\n"
            "- docs/ARCHITECTURE.md: code structure, data flow, design decisions\n"
            "When done, reply with exactly: TASK_DONE"
        ),
    },
]

def start_conversation(agent_info):
    """Start a new conversation with the given task."""
    body = {
        "agent": {
            "kind": "Agent",
        },
        "workspace": {"kind": "LocalWorkspace", "working_dir": "/home/openhands/workspace/project/rust-todo"},
        "initial_message": {
            "content": [{"type": "text", "text": agent_info["task"]}],
            "run": True,
        },
        "extra_data": {"automation_name": agent_info["name"]},
    }
    result = api("POST", "/api/conversations", body)
    if "id" in result:
        print(f"  ✅ [{agent_info['name']}] Started: {result['id'][:8]}...")
        return result["id"]
    print(f"  ❌ [{agent_info['name']}] Failed: {result}")
    return None

def check_conversation(conv_id):
    """Check if a conversation has finished."""
    result = api("GET", f"/api/conversations/{conv_id}")
    if "error" in result:
        return "error"
    status = result.get("execution_status", "unknown")
    return status

def main():
    print("=" * 60)
    print("🤖 Multi-Agent Orchestrator - Starting 3 Agents")
    print("=" * 60)

    # Step 1: Start all conversations
    print("\n[1/3] Starting conversations...")
    conv_ids = {}
    for agent in AGENTS:
        conv_id = start_conversation(agent)
        if conv_id:
            conv_ids[agent["name"]] = conv_id
        time.sleep(1)  # stagger starts

    if not conv_ids:
        print("\n❌ No conversations started. Aborting.")
        return

    print(f"\n[2/3] Monitoring {len(conv_ids)} agents...")

    # Step 2: Poll until all are done
    finished = set()
    start_time = time.time()
    timeout = 600  # 10 minutes max

    while len(finished) < len(conv_ids):
        elapsed = time.time() - start_time
        if elapsed > timeout:
            print(f"\n⏰ Timeout after {timeout}s. {len(conv_ids) - len(finished)} agents still running.")
            break

        for name, conv_id in list(conv_ids.items()):
            if name in finished:
                continue
            status = check_conversation(conv_id)
            if status in ("completed", "stopped", "finished"):
                print(f"  ✅ [{name}] Done ({elapsed:.0f}s)")
                finished.add(name)
            elif status == "error":
                print(f"  ❌ [{name}] Error")
                finished.add(name)

        time.sleep(5)

    # Step 3: Report
    print(f"\n[3/3] Summary")
    print("=" * 60)
    for name, conv_id in conv_ids.items():
        status = name in finished and "✅ Done" or "⏳ Timed out"
        print(f"  {status}: {name}")
    print(f"  Total time: {time.time() - start_time:.0f}s")
    print("=" * 60)

if __name__ == "__main__":
    main()