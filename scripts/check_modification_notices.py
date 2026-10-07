#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.
"""Record or check fork-change accounting, not legal compliance.

The upstream commit must be available locally. No network access, Git staging,
or source edits occur; `record` writes only MODIFICATIONS.json. See the process
document for the intentionally separate source/distribution review.
"""

from __future__ import annotations

import argparse
from datetime import date
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

BASE = "f1f2f3eab310a3220b2fcdb47531d3d3b85d6713"
UPSTREAM = "https://github.com/jub0t/concat"
FORK = "https://github.com/puntastic/concat-song"
INDEX = "MODIFICATIONS.json"
INLINE_SUFFIXES = {
    ".rs", ".py", ".sh", ".bash", ".toml", ".yaml", ".yml", ".nix",
    ".c", ".h", ".cpp", ".hpp", ".js", ".ts", ".css", ".scss", ".ps1",
}
NOTICE = "Modified for concat-song on {day}; see FORK-NOTICE.md."


class NoticeError(Exception):
    """An actionable accounting failure."""


def valid_date(value: object) -> bool:
    if not isinstance(value, str) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", value):
        return False
    try:
        date.fromisoformat(value)
        return True
    except ValueError:
        return False


def canonical(data: dict) -> bytes:
    return (json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")


def index_digest(data: dict) -> str:
    # A self-fingerprint without an impossible recursive hash. Every other
    # field, including the index's own dated entry, is covered.
    return hashlib.sha256(canonical({**data, "index_sha256": None})).hexdigest()


def legal_header_lines(text: str) -> set[str]:
    """Recognize actual notice lines, not prose about copyright or examples."""
    pattern = re.compile(
        r"^(?:(?://|#|/\*+|\*|<!--)\s*)?"
        r"(?:SPDX-(?:License-Identifier|FileCopyrightText):|Copyright\s+(?:\([cC]\)|©|[0-9]))",
        re.IGNORECASE,
    )
    result = set()
    fence = None
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if line.startswith(("```", "~~~")):
            marker = line[:3]
            if fence is None:
                fence = marker
            elif fence == marker:
                fence = None
            continue
        if fence is None and pattern.match(line):
            result.add(line)
    return result


class Repository:
    def __init__(self, root: Path, base: str = BASE):
        self.root = root.resolve()
        self.base = base
        self._baseline = None

    def git(self, *args: str, input_bytes: bytes | None = None) -> bytes:
        command = ["git", "-c", f"safe.directory={self.root.as_posix()}", *args]
        result = subprocess.run(command, cwd=self.root, input=input_bytes,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode:
            raise NoticeError(result.stderr.decode("utf-8", errors="replace").strip())
        return result.stdout

    def read(self, path: str) -> bytes:
        target = self.root / path
        if target.is_symlink():
            return os.readlink(target).encode("utf-8")
        if not target.is_file():
            raise NoticeError(f"{path}: unsupported file type (submodules are not covered)")
        return target.read_bytes()

    def baseline(self) -> dict:
        if self._baseline is not None:
            return self._baseline
        tree = self.git("ls-tree", "-r", "-z", self.base)
        result = {}
        for item in tree.split(b"\0"):
            if not item:
                continue
            metadata, path = item.split(b"\t", 1)
            mode, kind, blob = metadata.decode("ascii").split()
            if kind != "blob":
                raise NoticeError("Submodules require a separately designed notice policy")
            result[path.decode("utf-8")] = {"mode": mode, "blob": blob}
        self._baseline = result
        return result

    def snapshot(self) -> dict:
        base = self.baseline()
        # --no-renames deliberately represents a rename as a deletion + addition.
        # The comparison includes staged and unstaged changes together.
        raw = self.git("diff", "--raw", "--no-abbrev", "--no-renames", "-z", self.base, "--")
        parts = raw.split(b"\0")
        changes = {}
        for offset in range(0, len(parts) - 1, 2):
            metadata = parts[offset].decode("ascii").split()
            path = parts[offset + 1].decode("utf-8")
            changes[path] = metadata[1]  # actual working-tree mode, or 000000
        untracked = self.git("ls-files", "--others", "--exclude-standard", "-z")
        for item in untracked.split(b"\0"):
            if not item:
                continue
            path = item.decode("utf-8")
            target = self.root / path
            mode = "120000" if target.is_symlink() else "100644"
            if mode != "120000" and os.name != "nt" and target.stat().st_mode & 0o111:
                mode = "100755"
            changes[path] = mode
        # It exists conceptually before the first record, and has a canonical
        # self-hash instead of an ordinary Git blob fingerprint.
        changes[INDEX] = "100644"
        result = {}
        for path, mode in sorted(changes.items()):
            old = base.get(path)
            deleted = mode == "000000"
            blob = None
            if path != INDEX and not deleted:
                blob = self.git("hash-object", f"--path={path}", "--stdin",
                                input_bytes=self.read(path)).decode("ascii").strip()
            result[path] = {
                "change": "deleted" if deleted else "modified" if old else "added",
                "git_blob": blob,
                "mode": None if deleted else mode,
                "upstream_blob": old["blob"] if old else None,
                "upstream_mode": old["mode"] if old else None,
            }
        return result

    def notice_location(self, path: str, item: dict) -> str:
        if item["change"] == "deleted":
            return "central:deleted"
        if path == INDEX:
            return "central:index"
        if item["mode"] == "120000":
            return "central:symlink"
        if item["git_blob"] in {value["blob"] for value in self.baseline().values()}:
            return "central:unmodified-upstream-copy"
        data = self.read(path)
        if b"\0" in data:
            return "central:binary"
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            return "central:binary"
        if Path(path).name == "Cargo.lock" or re.search(
                r"@generated|automatically generated|generated file.*do not edit",
                text[:2048], re.IGNORECASE):
            return "central:generated"
        if PurePosixPath(path).suffix in INLINE_SUFFIXES or PurePosixPath(path).name == "Dockerfile":
            return "inline"
        return "central:non-commentable-or-prose"


def load_index(repo: Repository) -> dict:
    try:
        data = json.loads((repo.root / INDEX).read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise NoticeError(f"Cannot read {INDEX}: {exc}") from exc
    if not isinstance(data, dict):
        raise NoticeError(f"{INDEX}: expected a JSON object")
    return data


def validate_shape(repo: Repository, data: dict) -> list[str]:
    errors = []
    expected = {"schema_version": 1, "upstream_repository": UPSTREAM,
                "fork_repository": FORK, "upstream_commit": repo.base,
                "license": "AGPL-3.0-or-later"}
    for field, value in expected.items():
        if data.get(field) != value:
            errors.append(f"{field}: expected {value!r}")
    committed_at = repo.git("show", "-s", "--format=%cI", repo.base).decode().strip()
    if data.get("upstream_committed_at") != committed_at:
        errors.append("upstream_committed_at: does not match the pinned upstream commit")
    if not valid_date(data.get("recorded_on")):
        errors.append("recorded_on: expected a relevant ISO date (YYYY-MM-DD)")
    if not isinstance(data.get("entries"), dict):
        errors.append("entries: expected an object keyed by repository-relative paths")
        return errors
    for path, entry in data["entries"].items():
        pure = PurePosixPath(path)
        if (pure.is_absolute() or ".." in pure.parts or "\\" in path or ":" in path
                or str(pure) != path or not path):
            errors.append(f"{path!r}: invalid repository-relative path")
        if not isinstance(entry, dict):
            errors.append(f"{path}: expected an entry object")
            continue
        if not valid_date(entry.get("changed_on")):
            errors.append(f"{path}: missing or invalid changed_on date")
        if not isinstance(entry.get("summary"), str) or not entry["summary"].strip():
            errors.append(f"{path}: a nonempty change summary is required")
    return errors


def source_errors(repo: Repository, snapshot: dict, entries: dict) -> list[str]:
    errors = []
    # LICENSE is a verbatim license document, not a place for fork comments.
    original_license = repo.git("show", f"{repo.base}:LICENSE")
    # Git may materialize the upstream LF blob as CRLF on Windows. Do not
    # rewrite the existing file just to satisfy the checker: both exact
    # checkout representations are allowed, but no textual edits are.
    allowed_license_bytes = {original_license, original_license.replace(b"\n", b"\r\n")}
    if not (repo.root / "LICENSE").is_file() or repo.read("LICENSE") not in allowed_license_bytes:
        errors.append("LICENSE: must remain byte-identical to an upstream LF/CRLF checkout")
    for path, item in snapshot.items():
        entry = entries.get(path, {})
        expected_location = repo.notice_location(path, item)
        if entry.get("notice") != expected_location:
            errors.append(f"{path}: expected notice location {expected_location}")
        if path == INDEX or item["change"] == "deleted":
            continue
        content = repo.read(path).decode("utf-8", errors="replace")
        if expected_location == "inline":
            day = entry.get("changed_on", "YYYY-MM-DD")
            wanted = NOTICE.format(day=day)
            descriptive = re.compile(r"Modified by the concat-song fork on " + re.escape(day) + r":\s*\S+")
            heading = "\n".join(content.splitlines()[:40])
            if wanted not in heading and not descriptive.search(heading):
                errors.append(f"{path}: add a valid comment near the top: {wanted}")
        if item["upstream_blob"] and item["mode"] != "120000":
            before = repo.git("cat-file", "blob", item["upstream_blob"]).decode("utf-8", errors="replace")
            # This is a narrow regression check; prose licenses, warranty
            # notices, and third-party inventories still require human review.
            old_notices = legal_header_lines(before)
            current_lines = {line.strip() for line in content.splitlines()}
            for notice in sorted(old_notices - current_lines):
                errors.append(f"{path}: upstream SPDX/copyright line removed: {notice}")
    return errors


def check(repo: Repository) -> list[str]:
    data = load_index(repo)
    errors = validate_shape(repo, data)
    if errors:
        return errors
    if data.get("index_sha256") != index_digest(data):
        errors.append(f"{INDEX}: stale canonical self-fingerprint; use record")
    snapshot = repo.snapshot()
    entries = data["entries"]
    for path in sorted(snapshot.keys() - entries.keys()):
        errors.append(f"{path}: unrecorded fork change")
    for path in sorted(entries.keys() - snapshot.keys()):
        errors.append(f"{path}: stale entry (no current change from upstream)")
    for path in sorted(snapshot.keys() & entries.keys()):
        for field, value in snapshot[path].items():
            if entries[path].get(field) != value:
                errors.append(f"{path}: stale {field}; review and record this change")
    errors.extend(source_errors(repo, snapshot, entries))
    return errors


def record(repo: Repository, day: str, summary: str) -> None:
    if not valid_date(day) or not summary.strip():
        raise NoticeError("record requires --date YYYY-MM-DD and a nonempty --summary")
    previous = load_index(repo) if (repo.root / INDEX).exists() else None
    if previous:
        errors = validate_shape(repo, previous)
        if errors:
            raise NoticeError("Repair index metadata before recording:\n" + "\n".join(errors))
    old_entries = previous["entries"] if previous else {}
    snapshot = repo.snapshot()
    entries = {}
    for path, item in snapshot.items():
        old = old_entries.get(path, {})
        unchanged = all(old.get(field) == value for field, value in item.items())
        entries[path] = {
            **item,
            "changed_on": old["changed_on"] if unchanged else day,
            "summary": old["summary"] if unchanged else summary.strip(),
            "notice": repo.notice_location(path, item),
        }
    changed = entries != old_entries or bool(
        previous and previous.get("index_sha256") != index_digest(previous))
    if changed:
        entries[INDEX]["changed_on"] = day
        entries[INDEX]["summary"] = "Update the dated fork modification index."
    data = {
        "schema_version": 1,
        "upstream_repository": UPSTREAM,
        "upstream_commit": repo.base,
        "upstream_committed_at": repo.git("show", "-s", "--format=%cI", repo.base).decode().strip(),
        "fork_repository": FORK,
        "license": "AGPL-3.0-or-later",
        "recorded_on": day if changed else previous["recorded_on"],
        "entries": entries,
        "index_sha256": None,
    }
    data["index_sha256"] = index_digest(data)
    errors = source_errors(repo, snapshot, entries)
    if errors:
        raise NoticeError("No index written. Fix these notices first:\n" + "\n".join(errors))
    (repo.root / INDEX).write_bytes(canonical(data))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("check", help="check accounting, inline notices and preserved SPDX/copyright lines")
    update = commands.add_parser("record", help="record reviewed changes; writes only MODIFICATIONS.json")
    update.add_argument("--date", required=True, help="relevant modification date, YYYY-MM-DD")
    update.add_argument("--summary", required=True, help="description for new or changed entries")
    args = parser.parse_args()
    repo = Repository(Path(__file__).resolve().parents[1])
    try:
        if args.command == "record":
            record(repo, args.date, args.summary)
            print(f"Updated {INDEX}; run check before committing or publishing.")
            return 0
        errors = check(repo)
        if errors:
            print("Modification accounting failed:\n" + "\n".join(f"- {error}" for error in errors), file=sys.stderr)
            return 1
        print("Modification accounting is current. This is not a legal-compliance certification.")
        return 0
    except (NoticeError, OSError, UnicodeError) as exc:
        print(f"Modification accounting unavailable: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
