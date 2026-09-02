#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Scan all git-tracked files for common secret / token patterns."""

from __future__ import annotations

import os
import re
import subprocess
import sys

def _resolve_repo_root() -> str:
    if len(sys.argv) > 1 and sys.argv[1].strip():
        return os.path.abspath(sys.argv[1])
    # default: repo root is parent of scripts/ (i.e. script's grandparent)
    return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


REPO = _resolve_repo_root()

PATTERNS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"\bsk-[A-Za-z0-9]{20,}\b"), "OpenAI sk-*"),
    (re.compile(r"\bsk-ant-[A-Za-z0-9_-]{20,}"), "Anthropic sk-ant-*"),
    (re.compile(r"\bsk-proj-[A-Za-z0-9_-]{20,}"), "OpenAI project sk-proj-*"),
    (re.compile(r"\bghp_[A-Za-z0-9]{20,}\b"), "GitHub PAT ghp_*"),
    (re.compile(r"\bgho_[A-Za-z0-9]{20,}\b"), "GitHub OAuth gho_*"),
    (re.compile(r"\bghu_[A-Za-z0-9]{20,}\b"), "GitHub user-to-server ghu_*"),
    (re.compile(r"\bghs_[A-Za-z0-9]{20,}\b"), "GitHub server-to-server ghs_*"),
    (re.compile(r"\bghr_[A-Za-z0-9]{20,}\b"), "GitHub refresh ghr_*"),
    (re.compile(r"\bglpat-[A-Za-z0-9_-]{20,}"), "GitLab PAT glpat-*"),
    (re.compile(r"\bxox[bsp]-[A-Za-z0-9-]{10,}"), "Slack xox* token"),
    (re.compile(r"Bearer\s+[A-Za-z0-9._\-]{20,}"), "Authorization: Bearer ..."),
    (re.compile(r"\bAKIA[0-9A-Z]{16}\b"), "AWS Access Key AKIA*"),
    (re.compile(r"\bAIza[0-9A-Za-z_-]{35}\b"), "Google API Key AIza*"),
    (re.compile(r"""["']?api[_-]?key["']?\s*[:=]\s*["'][^"'\s]{8,}["']"""), "api_key= / apiKey: assignment"),
    (re.compile(r"""["']?token["']?\s*[:=]\s*["'][^"'\s]{12,}["']"""), "token= / token: assignment"),
    (re.compile(r"""["']?secret["']?\s*[:=]\s*["'][^"'\s]{12,}["']"""), "secret= / secret: assignment"),
    (re.compile(r"\bdpk-[A-Za-z0-9._-]{20,}"), "HuggingFace dpk-* (provider / deploy)"),
    (re.compile(r"\bhf_[A-Za-z0-9]{20,}\b"), "HuggingFace hf_* user token"),
]

SKIP_EXTS = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico", ".icns", ".woff",
             ".woff2", ".ttf", ".eot", ".mp4", ".mp3", ".zip", ".tar", ".gz", ".7z",
             ".rar", ".msi", ".exe", ".dll", ".so", ".dylib", ".wasm", ".svg"}


def iter_tracked(root: str) -> list[str]:
    out = subprocess.check_output(
        ["git", "-C", root, "ls-files", "-z"],
    )
    return [p for p in out.decode("utf-8", errors="replace").split("\x00") if p]


def main() -> int:
    files = iter_tracked(REPO)
    found: list[tuple[str, int, str, str]] = []
    for rel in files:
        ext = os.path.splitext(rel)[1].lower()
        if ext in SKIP_EXTS:
            continue
        path = os.path.join(REPO, rel)
        try:
            with open(path, "r", encoding="utf-8", errors="ignore") as fh:
                for lineno, line in enumerate(fh, 1):
                    for rx, label in PATTERNS:
                        m = rx.search(line)
                        if m:
                            hit = m.group(0).replace("\r", "").replace("\n", "")
                            found.append((rel, lineno, label, hit[:80]))
        except (OSError, IsADirectoryError):
            continue
    print(f"Scanned {len(files)} tracked files (skipped binary-ish exts)")
    if found:
        print(f"\n!!! {len(found)} suspicious match(es) !!!")
        for file, ln, lbl, hit in found:
            print(f"  - {file}:{ln}  [{lbl}]  ->  {hit!r}")
        return 1
    print("\nOK: no secret / API-key-like patterns detected in tracked files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
