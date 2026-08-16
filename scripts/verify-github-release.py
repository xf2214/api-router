#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Verify v0.1.0-alpha release metadata + assets on xf2214/api-router."""

from __future__ import annotations

import json
import subprocess
import sys
import urllib.request
import urllib.error

OWNER, REPO, TAG = "xf2214", "api-router", "v0.1.0-alpha"


def token_via_gcm() -> str:
    payload = "protocol=https\nhost=github.com\n\n".encode()
    for _ in range(2):
        p = subprocess.run(["git", "credential", "fill"],
                           input=payload, capture_output=True)
        if p.returncode == 0:
            break
    if p.returncode != 0:
        raise RuntimeError(p.stderr.decode(errors="replace"))
    for ln in p.stdout.decode().splitlines():
        if ln.startswith("password="):
            return ln.split("=", 1)[1]
    raise RuntimeError("no token")


def http(method, url, token):
    req = urllib.request.Request(url, method=method, headers={
        "Accept": "application/vnd.github+json",
        "Authorization": f"Bearer {token}",
        "X-GitHub-Api-Version": "2022-11-28",
    })
    try:
        with urllib.request.urlopen(req) as r:
            return r.status, json.loads(r.read().decode())
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read().decode())


def main():
    tok = token_via_gcm()
    list_url = f"https://api.github.com/repos/{OWNER}/{REPO}/releases"
    tags_url = f"https://api.github.com/repos/{OWNER}/{REPO}/tags"

    s1, rels = http("GET", list_url, tok)
    s2, tags = http("GET", tags_url, tok)
    print(f"list releases HTTP {s1}  tags HTTP {s2}")
    print("TAGS:")
    for t in tags:
        print(" -", t.get("name"), "->", t.get("commit", {}).get("sha"))
    print("RELEASES:")
    for r in rels:
        assets = r.get("assets", [])
        print(f"  tag={r.get('tag_name')}  id={r.get('id')}  name={r.get('name')}")
        print(f"      prerelease={r.get('prerelease')}  draft={r.get('draft')}  url={r.get('html_url')}")
        print(f"      body bytes={len(r.get('body', ''))}  assets={len(assets)}")
        for a in assets:
            print(f"       - {a['name']}  ({a.get('size')} B)  label={a.get('label')!r}  dl={a.get('browser_download_url')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
