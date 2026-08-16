#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Create release v0.1.1 on xf2214/api-router and upload 2 assets.

Authentication: retrieves a GitHub token from Git Credential Manager
via `git credential fill`, then uses standard github.com REST API.

Usage:
    py -3 scripts\\create-github-release.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import urllib.parse
import urllib.request
from pathlib import Path

OWNER = "xf2214"
REPO = "api-router"
TAG = "v0.1.1"
TITLE = "API Router v0.1.1 · 补丁版（修复 Token 统计不计数）"


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def get_token_via_gcm() -> str:
    """Call `git credential fill` for github.com and return the bearer token."""
    payload = "protocol=https\nhost=github.com\n\n".encode("utf-8")
    proc = subprocess.run(
        ["git", "credential", "fill"],
        input=payload,
        capture_output=True,
    )
    if proc.returncode != 0:
        # retry once to give GCM time for interactive prompt
        proc = subprocess.run(
            ["git", "credential", "fill"],
            input=payload,
            capture_output=True,
        )
    if proc.returncode != 0:
        raise RuntimeError(
            "git credential fill failed:\n"
            + proc.stdout.decode(errors="replace")
            + proc.stderr.decode(errors="replace")
        )
    out = proc.stdout.decode(errors="replace")
    for line in out.splitlines():
        if line.startswith("password="):
            return line.split("=", 1)[1].strip()
    raise RuntimeError("No password/token returned by git credential (GCM).")


def api_request(method: str, url: str, token: str, *,
                json_body: dict | None = None,
                raw_body: bytes | None = None,
                content_type: str = "application/json",
                extra_accept: str | None = None) -> dict:
    headers = {
        "Accept": extra_accept or "application/vnd.github+json",
        "Authorization": f"Bearer {token}",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    data: bytes | None
    if json_body is not None:
        data = json.dumps(json_body, ensure_ascii=False).encode("utf-8")
        headers["Content-Type"] = content_type
    elif raw_body is not None:
        data = raw_body
        headers["Content-Type"] = content_type
    else:
        data = None

    req = urllib.request.Request(url, data=data, method=method, headers=headers)
    try:
        with urllib.request.urlopen(req) as resp:
            body = resp.read()
    except urllib.error.HTTPError as e:
        info = e.read().decode(errors="replace")
        raise RuntimeError(
            f"HTTP {e.code} {e.reason} for {method} {url}\n{info}"
        ) from e
    if not body:
        return {}
    try:
        return json.loads(body.decode("utf-8"))
    except json.JSONDecodeError:
        return {"_raw": body.decode(errors="replace")}


def main() -> int:
    root = repo_root()
    notes_path = root / f"release-notes-{TAG}.md"
    bundle = root / "src-tauri" / "target" / "release" / "bundle"
    assets = [
        (bundle / "nsis" / "API Router_0.1.1_x64-setup.exe",
         "Windows x64 NSIS installer (recommended) - API Router_0.1.1_x64-setup.exe"),
        (bundle / "msi" / "API Router_0.1.1_x64_en-US.msi",
         "Windows x64 WiX installer - API Router_0.1.1_x64_en-US.msi"),
    ]
    for p, _ in assets:
        if not p.is_file():
            print(f"Asset missing: {p}", file=sys.stderr)
            return 2
    if not notes_path.is_file():
        print(f"Notes file missing: {notes_path}", file=sys.stderr)
        return 2

    print("[1/4] Acquiring GitHub token via git-credential (GCM) ...", flush=True)
    token = get_token_via_gcm()
    print("      OK", flush=True)

    print(f"[2/4] Creating release {TAG} on {OWNER}/{REPO} ...", flush=True)
    body = {
        "tag_name": TAG,
        "target_commitish": "master",
        "name": TITLE,
        "body": notes_path.read_text(encoding="utf-8"),
        "draft": False,
        "prerelease": False,
        "generate_release_notes": False,
        "make_latest": "true",
    }
    release_url = f"https://api.github.com/repos/{OWNER}/{REPO}/releases"
    rel = api_request("POST", release_url, token, json_body=body)
    release_id = rel.get("id")
    upload_tpl = rel.get("upload_url", "")
    upload_base = upload_tpl.split("{?name,label}", 1)[0]
    print(f"      Release ID = {release_id}", flush=True)

    print(f"[3/4] Uploading {len(assets)} assets ...", flush=True)
    for path, label in assets:
        size_mb = path.stat().st_size / (1024 * 1024)
        q = urllib.parse.urlencode({"name": path.name, "label": label})
        url = f"{upload_base}?{q}"
        print(f"      -> {path.name} ({size_mb:.2f} MB) ... ", end="", flush=True)
        api_request(
            "POST", url, token,
            raw_body=path.read_bytes(),
            content_type="application/octet-stream",
        )
        print("done", flush=True)

    print("")
    print("[4/4] Release created.")
    print(f"      URL: https://github.com/{OWNER}/{REPO}/releases/tag/{TAG}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
