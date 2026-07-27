#!/usr/bin/env python3
"""Agent-facing CLI for the code-map service.

Subcommands:
  query <term>            Ranked candidate files for a keyword/class/topic/table.
  neighbors <entity>      Resolve an entity (topic/table/feign/class) to
                          producers/consumers/readers/writers/callers.

Use --json for compact machine-readable output optimized for agent ingestion:
each hit is `{repo, path, line, score, reason, symbols:[{kind,name,line}]}`.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

DEFAULT_BASE = os.environ.get("CODE_MAP_BASE_URL", "http://127.0.0.1:18765")
DEFAULT_MAX = int(os.environ.get("CODE_MAP_MAX_RESULTS", "12"))


def http_json(base: str, path: str, payload: dict) -> dict:
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{base}{path}",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        try:
            msg = json.loads(body).get("message", body)
        except json.JSONDecodeError:
            msg = body
        print(f"code-map error: {msg}", file=sys.stderr)
        sys.exit(1)
    except urllib.error.URLError as exc:
        print(f"code-map error: cannot reach {base} ({exc.reason})", file=sys.stderr)
        print("hint: start the service with `npm run deploy` or `npm run api`", file=sys.stderr)
        sys.exit(1)


def http_get(base: str, path: str) -> dict:
    req = urllib.request.Request(f"{base}{path}", method="GET")
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except (urllib.error.HTTPError, urllib.error.URLError) as exc:
        print(f"code-map error: {exc}", file=sys.stderr)
        sys.exit(1)


def resolve_project(base: str, explicit: str | None) -> str:
    if explicit:
        return explicit
    settings = http_get(base, "/api/settings")
    active = settings.get("active_project_id")
    if active:
        return active
    projects = http_get(base, "/api/projects")
    if len(projects) == 1:
        return projects[0]["id"]
    print(
        "code-map error: no active project; set CODE_MAP_PROJECT_ID or mark one active",
        file=sys.stderr,
    )
    sys.exit(1)


def hit_location(repo: str | None, repo_rel: str, rel: str) -> tuple[str, str, int]:
    repo_val = repo or ""
    path = repo_rel if repo_val else rel
    return repo_val, path, 0


def first_line(result: dict) -> int:
    for key in ("snippets", "symbols", "relationships"):
        items = result.get(key) or []
        if items:
            return items[0].get("line", 0)
    return 0


def compact_query(resp: dict) -> dict:
    results = []
    for result in resp.get("results", []):
        results.append(
            {
                "repo": result.get("repo"),
                "path": result.get("repo_relative_path") or result.get("relative_path"),
                "line": first_line(result),
                "score": result.get("score", 0),
                "reason": (result.get("reasons") or [""])[0],
                "symbols": [
                    {"kind": s.get("kind"), "name": s.get("name"), "line": s.get("line")}
                    for s in (result.get("symbols") or [])
                ],
            }
        )
    return {
        "project": resp.get("project_name"),
        "query": resp.get("query"),
        "count": resp.get("result_count"),
        "notes": [
            {
                "query": n.get("query"),
                "summary": n.get("summary"),
                "pointers": n.get("pointers") or [],
            }
            for n in (resp.get("notes") or [])
        ],
        "results": results,
    }


def compact_neighbors(resp: dict) -> dict:
    return {
        "project": resp.get("project_name"),
        "entity": resp.get("entity"),
        "definitions": resp.get("definitions", []),
        "producers": resp.get("producers", []),
        "consumers": resp.get("consumers", []),
        "readers": resp.get("readers", []),
        "writers": resp.get("writers", []),
        "callers": resp.get("callers", []),
    }


def print_query_human(resp: dict) -> None:
    print("== Summary ==")
    for line in resp.get("summary_lines", []):
        print(line)
    print()
    print("== Results ==")
    for result in resp.get("results", []):
        repo = result.get("repo")
        path = result.get("repo_relative_path") or result.get("relative_path")
        header = f"{repo}:{path}" if repo else path
        print(f"{header} score={result.get('score')}")
        reasons = result.get("reasons") or []
        if reasons:
            print(f"  reasons: {'; '.join(reasons)}")
        symbols = result.get("symbols") or []
        if symbols:
            print(
                "  symbols: "
                + "; ".join(f"{s['kind']}:{s['name']}@L{s['line']}" for s in symbols)
            )
        rels = result.get("relationships") or []
        if rels:
            print(
                "  relationships: "
                + "; ".join(
                    f"{r['kind']}:{r['from']}->{r['to']}@L{r['line']}" for r in rels
                )
            )
        for snippet in result.get("snippets") or []:
            print(f"  L{snippet['line']}: {snippet['text']}")
        print()


def print_neighbors_human(resp: dict) -> None:
    print(f"== {resp.get('entity')} in {resp.get('project_name')} ==")
    for label in ["definitions", "producers", "consumers", "readers", "writers", "callers"]:
        hits = resp.get(label, [])
        print(f"-- {label} ({len(hits)}) --")
        for hit in hits:
            repo = hit.get("repo") or ""
            path = hit.get("repo_relative_path") or ""
            loc = f"{repo}:{path}" if repo else path
            print(f"  {hit.get('kind')} {hit.get('name')} -> {loc}:L{hit.get('line')}")
        print()


def cmd_query(args: argparse.Namespace) -> int:
    base = args.base_url
    project_id = resolve_project(base, args.project_id)
    resp = http_json(
        base,
        "/api/query",
        {
            "project_id": project_id,
            "query": args.term,
            "max_results": args.max_results,
        },
    )
    if args.json:
        print(json.dumps(compact_query(resp), ensure_ascii=False))
    else:
        print_query_human(resp)
    return 0


def cmd_neighbors(args: argparse.Namespace) -> int:
    base = args.base_url
    project_id = resolve_project(base, args.project_id)
    resp = http_json(
        base,
        "/api/neighbors",
        {"project_id": project_id, "entity": args.entity},
    )
    if args.json:
        print(json.dumps(compact_neighbors(resp), ensure_ascii=False))
    else:
        print_neighbors_human(resp)
    return 0


def build_parser() -> argparse.ArgumentParser:
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument(
        "--base-url",
        default=DEFAULT_BASE,
        help=f"code-map service base URL (default: {DEFAULT_BASE})",
    )
    common.add_argument(
        "--project-id",
        default=os.environ.get("CODE_MAP_PROJECT_ID"),
        help="project id (default: active project or CODE_MAP_PROJECT_ID)",
    )
    common.add_argument("--json", action="store_true", help="compact JSON for agent ingestion")

    parser = argparse.ArgumentParser(
        prog="code-map",
        description="Agent-facing cross-repo code map CLI.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    q = sub.add_parser("query", parents=[common], help="ranked candidate files for a term")
    q.add_argument("term", help="keyword / class / endpoint / topic / table / config key")
    q.add_argument(
        "--max-results",
        type=int,
        default=DEFAULT_MAX,
        help=f"max results (default: {DEFAULT_MAX})",
    )
    q.set_defaults(func=cmd_query)

    n = sub.add_parser(
        "neighbors",
        parents=[common],
        help="resolve entity to producers/consumers/readers/writers/callers",
    )
    n.add_argument("entity", help="topic / table / feign client / class name / constant")
    n.set_defaults(func=cmd_neighbors)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
