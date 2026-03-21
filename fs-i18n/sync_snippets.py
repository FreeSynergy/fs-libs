#!/usr/bin/env python3
"""
Sync i18n snippet files.

- EN is the source of truth (schema).
- All other languages get missing EN keys added as TODO placeholders.
- Each language gets a meta.toml with schema_version + completeness.
- EN gets a meta.toml marking it as the schema source.

Usage: python3 sync_snippets.py [--dry-run]
"""

import os
import re
import sys
from pathlib import Path

SNIPPETS_DIR = Path(__file__).parent / "snippets"
SCHEMA_VERSION = "1.0.0"
EN = "en"

# Files that contain keys (exclude meta.toml)
SNIPPET_FILES = [
    "actions.toml",
    "confirmations.toml",
    "errors.toml",
    "help.toml",
    "labels.toml",
    "notifications.toml",
    "nouns.toml",
    "phrases.toml",
    "status.toml",
    "time.toml",
    "validation.toml",
]

DRY_RUN = "--dry-run" in sys.argv


def parse_keys(path: Path) -> dict[str, str]:
    """Extract key = value pairs from a TOML snippet file. Preserves order."""
    keys = {}
    if not path.exists():
        return keys
    for line in path.read_text(encoding="utf-8").splitlines():
        # Match: key = "value" or key = 'value' (with optional spaces around key)
        m = re.match(r"^(\w+)\s*=\s*(['\"])(.*)\2\s*$", line)
        if m:
            keys[m.group(1)] = m.group(3)
    return keys


def append_missing_keys(path: Path, missing: dict[str, str]) -> None:
    """Append missing keys to the end of a file as TODO placeholders."""
    if not missing:
        return
    content = path.read_text(encoding="utf-8")
    # Ensure file ends with newline
    if not content.endswith("\n"):
        content += "\n"
    lines = ["\n# --- TODO: translate ---"]
    for key, en_val in missing.items():
        # Escape any quotes in the EN value
        safe_val = en_val.replace('"', '\\"')
        lines.append(f'# {key} = "{safe_val}"')
    content += "\n".join(lines) + "\n"
    if not DRY_RUN:
        path.write_text(content, encoding="utf-8")


def write_meta(lang_dir: Path, completeness: int, is_schema: bool = False) -> None:
    meta_path = lang_dir / "meta.toml"
    if is_schema:
        content = f"""\
# meta.toml — Schema source (English is the source of truth)

schema_version = "{SCHEMA_VERSION}"
is_schema      = true
completeness   = 100
"""
    else:
        content = f"""\
# meta.toml — Language metadata

schema_version = "{SCHEMA_VERSION}"
completeness   = {completeness}
"""
    if not DRY_RUN:
        meta_path.write_text(content, encoding="utf-8")


def count_total_en_keys(en_dir: Path) -> int:
    total = 0
    for fname in SNIPPET_FILES:
        total += len(parse_keys(en_dir / fname))
    return total


def sync_language(lang: str, en_keys_per_file: dict[str, dict[str, str]], en_total: int) -> None:
    lang_dir = SNIPPETS_DIR / lang
    if not lang_dir.is_dir():
        print(f"  SKIP {lang}: directory not found")
        return

    total_missing = 0
    total_translated = 0

    for fname in SNIPPET_FILES:
        en_keys = en_keys_per_file[fname]
        if not en_keys:
            continue

        lang_path = lang_dir / fname
        if not lang_path.exists():
            # Create file from scratch with all EN keys as TODO
            section = fname.replace(".toml", "")
            header = f"# {fname} — (untranslated)\n\n[{section}]\n"
            lines = ["# --- TODO: translate ---"]
            for key, val in en_keys.items():
                safe_val = val.replace('"', '\\"')
                lines.append(f'# {key} = "{safe_val}"')
            full = header + "\n".join(lines) + "\n"
            if not DRY_RUN:
                lang_path.write_text(full, encoding="utf-8")
            total_missing += len(en_keys)
            print(f"  CREATE {lang}/{fname}: {len(en_keys)} keys (all TODO)")
            continue

        lang_keys = parse_keys(lang_path)
        missing = {k: v for k, v in en_keys.items() if k not in lang_keys}
        translated = len(lang_keys)  # existing keys count as translated
        total_missing += len(missing)
        total_translated += translated

        if missing:
            print(f"  FIX    {lang}/{fname}: +{len(missing)} missing keys")
            append_missing_keys(lang_path, missing)
        else:
            total_translated += len(en_keys)  # fully covered

    # Completeness = (en_total - total_missing) / en_total * 100
    completeness = max(0, round((en_total - total_missing) / en_total * 100))
    write_meta(lang_dir, completeness)

    status = "✓" if total_missing == 0 else f"⚠ {total_missing} TODO"
    print(f"  {lang}: {completeness}% complete  [{status}]")


def main() -> None:
    if DRY_RUN:
        print("DRY RUN — no files will be written\n")

    en_dir = SNIPPETS_DIR / EN
    if not en_dir.is_dir():
        print(f"ERROR: EN directory not found: {en_dir}")
        sys.exit(1)

    # Load all EN keys
    en_keys_per_file: dict[str, dict[str, str]] = {}
    en_total = 0
    for fname in SNIPPET_FILES:
        keys = parse_keys(en_dir / fname)
        en_keys_per_file[fname] = keys
        en_total += len(keys)
    print(f"EN schema: {en_total} keys across {len(SNIPPET_FILES)} files\n")

    # Write EN meta
    write_meta(en_dir, 100, is_schema=True)
    print(f"  en: schema source (100%)\n")

    # Sync all other languages
    all_langs = sorted(
        d.name for d in SNIPPETS_DIR.iterdir()
        if d.is_dir() and d.name != EN
    )
    print(f"Syncing {len(all_langs)} languages...\n")

    for lang in all_langs:
        sync_language(lang, en_keys_per_file, en_total)

    print(f"\nDone. Schema version: {SCHEMA_VERSION}")


if __name__ == "__main__":
    main()
