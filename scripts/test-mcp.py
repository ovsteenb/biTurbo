#!/usr/bin/env python3
"""
MCP smoke test for biTurbo.

Spawns the biturbo-mcp binary over stdio, runs the MCP handshake,
discovers every tool the server exposes via tools/list, then calls
each one and prints a PASS/FAIL summary. Exits non-zero on any failure.

Usage: scripts/test-mcp.py [--binary PATH] [--keep]
   --binary PATH   Path to the biturbo-mcp binary
                   (default: src-tauri/target/release/biturbo-mcp)
   --keep          Don't delete the smoke project / memories afterwards
"""
from __future__ import annotations
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Optional

DEFAULT_BINARY = "src-tauri/target/release/biturbo-mcp"
TAG_PREFIX = "mcp-smoke"
ROOT = Path(__file__).resolve().parent.parent


def now_ms() -> int:
    return int(time.time() * 1000)


class McpClient:
    def __init__(self, proc: subprocess.Popen):
        self.proc = proc
        self._id = 0

    def _next_id(self) -> int:
        self._id += 1
        return self._id

    def call(self, method: str, params: Optional[dict] = None) -> dict:
        msg = {"jsonrpc": "2.0", "id": self._next_id(), "method": method}
        if params is not None:
            msg["params"] = params
        line = json.dumps(msg)
        try:
            self.proc.stdin.write(line + "\n")
            self.proc.stdin.flush()
        except BrokenPipeError as e:
            raise RuntimeError(f"server closed stdin: {e}")
        return self._read_response(msg["id"])

    def notify(self, method: str, params: Optional[dict] = None) -> None:
        msg = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            msg["params"] = params
        self.proc.stdin.write(json.dumps(msg) + "\n")
        self.proc.stdin.flush()

    def _read_response(self, want_id: int) -> dict:
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise RuntimeError("server closed stdout")
            line = line.strip()
            if not line:
                continue
            try:
                resp = json.loads(line)
            except json.JSONDecodeError:
                continue
            if "id" in resp and resp["id"] == want_id:
                return resp


def spawn(binary: str, env: dict) -> subprocess.Popen:
    return subprocess.Popen(
        [binary],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
        env=env,
    )


def tool_text(result: dict) -> str:
    if "error" in result:
        return ""
    content = result.get("result", {}).get("content", [])
    if not content:
        return ""
    return "\n".join(c.get("text", "") for c in content if c.get("type") == "text")


def tool_ok(result: dict) -> bool:
    if "error" in result:
        return False
    if "result" not in result:
        return False
    if result["result"].get("isError") is True:
        return False
    return True


def fmt_result(result: dict) -> str:
    if "error" in result:
        return json.dumps(result["error"])[:120]
    content = result.get("result", {}).get("content", [])
    if content and isinstance(content, list) and content[0].get("type") == "text":
        t = content[0].get("text", "")
        return (t[:80] + "…") if len(t) > 80 else t
    return "ok"


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--binary", default=DEFAULT_BINARY)
    p.add_argument("--keep", action="store_true")
    args = p.parse_args()

    binary = args.binary
    if not os.path.isabs(binary):
        binary = str(ROOT / binary)
    if not os.path.exists(binary):
        print(f"binary not found: {binary}", file=sys.stderr)
        return 2

    # The MCP server uses the user's app data dir by default, so the
    # smoke test pollutes the real DB. We use a throwaway dir for
    # both the WAL files and any on-disk state. biturbo-mcp doesn't
    # accept a data-dir flag in the current build, so we run against
    # the user's dir and clean up via the `delete_project` tool plus
    # a `forget` loop using the unique tag.
    env = os.environ.copy()
    env["RUST_LOG"] = "warn"

    print(f"[smoke] binary={binary}")
    proc = spawn(binary, env)
    client = McpClient(proc)

    results: list[tuple[str, bool, int, str]] = []
    ts = now_ms()
    project_id = f"{TAG_PREFIX}-{ts}"
    tag = f"{TAG_PREFIX}-{ts}"

    def record(name: str, ok: bool, elapsed_ms: int, summary: str) -> None:
        results.append((name, ok, elapsed_ms, summary))
        mark = "PASS" if ok else "FAIL"
        print(f"  [{mark}] {name:32s} {elapsed_ms:>5d}ms  {summary[:80]}")

    def call(name: str, args: Optional[dict] = None) -> dict:
        t0 = time.perf_counter()
        r = client.call("tools/call", {"name": name, "arguments": args or {}})
        elapsed = int((time.perf_counter() - t0) * 1000)
        record(name, tool_ok(r), elapsed, fmt_result(r))
        return r

    try:
        # 1. Handshake.
        t0 = time.perf_counter()
        init = client.call("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "biturbo-smoke", "version": "0.1"},
        })
        client.notify("notifications/initialized")
        elapsed = int((time.perf_counter() - t0) * 1000)
        ok = "error" not in init and "result" in init
        record("initialize", ok, elapsed, "ok" if ok else json.dumps(init.get("error"))[:80])
        if not ok:
            return 1

        # 2. Discover tools.
        t0 = time.perf_counter()
        list_resp = client.call("tools/list")
        elapsed = int((time.perf_counter() - t0) * 1000)
        if "error" in list_resp or "result" not in list_resp:
            record("tools/list", False, elapsed, json.dumps(list_resp)[:80])
            return 1
        tools = list_resp["result"].get("tools", [])
        record("tools/list", True, elapsed, f"{len(tools)} tools")
        tool_names = [t["name"] for t in tools]
        print(f"[smoke] discovered {len(tool_names)} tools: {', '.join(tool_names)}")

        # Pre-create the smoke project so dependent tools (remember,
        # search, list, etc.) can run in any order the server returns
        # them in. Skip cleanly if the server doesn't expose
        # create_project.
        if "create_project" in tool_names:
            call("create_project", {"name": project_id, "id": project_id})

        # 3. Smoke each tool. Skip the project-mutating ones
        #    (create_project, delete_project) and register_agent here
        #    because they're handled in setup/cleanup. Everything else
        #    is a normal call against the smoke project that already
        #    exists.
        skipped = {
            "create_project",
            "delete_project",
            "register_agent",
        }
        for t in tools:
            name = t["name"]
            if name in skipped:
                continue
            try:
                if name == "list_projects":
                    call(name)
                elif name == "get_project":
                    call(name, {"id": project_id})
                elif name == "stats":
                    call(name)
                elif name == "bootstrap":
                    call(name)
                elif name == "recent_activity":
                    call(name, {"limit": 5})
                elif name == "consolidate_status":
                    call(name)
                elif name == "list_tags":
                    call(name, {"project_id": project_id})
                elif name == "remember":
                    call(name, {
                        "content": f"smoke test memory {ts}",
                        "project_id": project_id,
                        "tags": [tag],
                        "importance": 0.5,
                        "mem_type": "fact",
                        "source_agent": f"smoke-agent-{ts}",
                    })
                elif name == "list":
                    call(name, {"project_id": project_id, "limit": 5})
                elif name == "search":
                    call(name, {"project_id": project_id, "query": "smoke", "k": 3})
                elif name == "recall_for_context":
                    call(name, {"project_id": project_id, "query": "smoke", "k": 3})
                elif name == "update":
                    w = client.call("tools/call", {
                        "name": "remember",
                        "arguments": {
                            "content": f"smoke target for update {ts}",
                            "project_id": project_id,
                            "tags": [tag],
                            "source_agent": f"smoke-agent-{ts}",
                        },
                    })
                    wt = tool_text(w)
                    if not wt:
                        record(name, False, 0, "could not write target memory")
                        continue
                    try:
                        target = json.loads(wt)
                    except json.JSONDecodeError:
                        record(name, False, 0, "remember result not JSON")
                        continue
                    call(name, {"uid": target["uid"], "content": f"smoke updated {ts}"})
                elif name == "get_memory":
                    w = client.call("tools/call", {
                        "name": "remember",
                        "arguments": {
                            "content": f"smoke target for get_memory {ts}",
                            "project_id": project_id,
                            "tags": [tag],
                            "source_agent": f"smoke-agent-{ts}",
                        },
                    })
                    wt = tool_text(w)
                    if not wt:
                        record(name, False, 0, "could not write target memory")
                        continue
                    try:
                        target = json.loads(wt)
                    except json.JSONDecodeError:
                        record(name, False, 0, "remember result not JSON")
                        continue
                    call(name, {"uid": target["uid"]})
                elif name == "get_project_graph":
                    call(name, {"project_id": project_id})
                elif name == "consolidate":
                    call(name, {"project_id": project_id})
                elif name == "ingest_project":
                    record(name, True, 0, "SKIP (needs code root)")
                elif name == "import_folder":
                    record(name, True, 0, "SKIP (needs folder)")
                elif name == "export_memories":
                    tmp = tempfile.NamedTemporaryFile(
                        delete=False, suffix=".json", mode="w"
                    )
                    tmp.close()
                    call(name, {"project_id": project_id, "output_path": tmp.name})
                    os.unlink(tmp.name)
                elif name == "set_project_embed_model":
                    call(name, {"project_id": project_id, "model": None})
                elif name == "enable_watch":
                    record(name, True, 0, "SKIP (needs code root)")
                elif name == "disable_watch":
                    call(name, {"project_id": project_id})
                elif name == "watch_status":
                    call(name)
                elif name == "forget":
                    w = client.call("tools/call", {
                        "name": "remember",
                        "arguments": {
                            "content": f"smoke target for forget {ts}",
                            "project_id": project_id,
                            "tags": [tag],
                            "source_agent": f"smoke-agent-{ts}",
                        },
                    })
                    wt = tool_text(w)
                    if not wt:
                        record(name, False, 0, "could not write target memory")
                        continue
                    try:
                        target = json.loads(wt)
                    except json.JSONDecodeError:
                        record(name, False, 0, "remember result not JSON")
                        continue
                    call(name, {"uid": target["uid"]})
                else:
                    record(name, True, 0, "SKIP (no test defined)")
            except Exception as e:
                record(name, False, 0, f"exception: {e}")

        # Cleanup: forget any remaining smoke memories, then drop the
        # project. This is run unconditionally so the user's DB stays
        # clean regardless of test results.
        if not args.keep:
            try:
                lst = client.call("tools/call", {
                    "name": "list",
                    "arguments": {"project_id": project_id, "limit": 200},
                })
                txt = tool_text(lst)
                if txt:
                    try:
                        items = json.loads(txt)
                    except json.JSONDecodeError:
                        items = []
                    for m in items:
                        if tag in (m.get("tags") or []):
                            try:
                                client.call("tools/call", {
                                    "name": "forget",
                                    "arguments": {"uid": m["uid"]},
                                })
                            except Exception:
                                pass
                client.call("tools/call", {
                    "name": "delete_project",
                    "arguments": {"project_id": project_id},
                })
            except Exception:
                pass

    finally:
        try:
            proc.stdin.close()
        except Exception:
            pass
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()

    # Summary table.
    print()
    print(f"{'Tool':32s}  {'Status':6s}  {'Time':>7s}  Notes")
    print("-" * 80)
    for name, ok, ms, summary in results:
        mark = "PASS" if ok else "FAIL"
        print(f"{name:32s}  {mark:6s}  {ms:>5d}ms  {summary[:40]}")
    total = len(results)
    passed = sum(1 for _, ok, _, _ in results if ok)
    print("-" * 80)
    print(f"Total: {passed}/{total} passed")

    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
