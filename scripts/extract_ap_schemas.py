"""
Extract Archipelago world option schemas into our randomizer schema format.

Run from the Zephyr repo root:

    python scripts/extract_ap_schemas.py

Reads every world under src-tauri/archipelago-runtime/worlds/ and writes one JSON file
per world to data/randomizer/schemas/<world_id>.json. Worlds whose modules
fail to import (missing optional deps, missing ROMs, etc.) are skipped with a
logged reason. The script never installs anything globally; it just adds the
local Archipelago package to sys.path.
"""

from __future__ import annotations

import argparse
import importlib
import json
import os
import re
import sys
import traceback
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent


def _resolve_ap_root() -> Path:
    """Figure out where Archipelago lives before argparse runs.

    Priority: `--ap-root <path>` in argv, then `$ZEPHYR_AP_ROOT`, then the
    dev layout (`<repo>/src-tauri/archipelago-runtime`). Rust passes
    `--ap-root` explicitly in release, where the runtime lives under
    `<app_data_dir>/randomizer/archipelago-runtime` and the dev fallback
    would point at a non-existent path.
    """
    argv = sys.argv[1:]
    for i, a in enumerate(argv):
        if a == "--ap-root" and i + 1 < len(argv):
            return Path(argv[i + 1]).expanduser().resolve()
        if a.startswith("--ap-root="):
            return Path(a.split("=", 1)[1]).expanduser().resolve()
    env = os.environ.get("ZEPHYR_AP_ROOT")
    if env:
        return Path(env).expanduser().resolve()
    return REPO_ROOT / "src-tauri" / "archipelago-runtime"


AP_ROOT = _resolve_ap_root()
WORLDS_DIR = AP_ROOT / "worlds"
# Directory where schemas land unless --out-dir overrides it.
DEFAULT_OUT_DIR = REPO_ROOT / "data" / "randomizer" / "schemas"
# Custom (user-installed) worlds live here. Archipelago's loader already
# discovers them at runtime; we also scan the folder here so the extractor
# can produce schemas for them without any extra plumbing.
CUSTOM_WORLDS_DIR = AP_ROOT / "custom_worlds"

# Make the bundled Archipelago importable.
sys.path.insert(0, str(AP_ROOT))

# Importing Options/AutoWorld pulls in a chunk of the Archipelago runtime.
import Options  # type: ignore  # noqa: E402
from worlds.AutoWorld import AutoWorldRegister, World  # type: ignore  # noqa: E402

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

CATEGORY_KEYWORDS: list[tuple[str, tuple[str, ...]]] = [
    ("goals", ("goal", "victory", "win_condition", "ending")),
    ("entrances", ("entrance", "shuffle_entrance", "door_shuffle", "warp")),
    ("logic", ("logic", "glitch", "trick", "difficulty", "accessibility")),
    ("cosmetic", ("color", "colour", "music", "sprite", "palette", "skin", "sfx", "sound")),
    ("items", ("item", "starting_inventory", "start_inventory", "loot", "shop")),
]


def humanize(name: str) -> str:
    return name.replace("_", " ").strip().title()


def first_sentence(doc: str | None) -> str:
    if not doc:
        return ""
    text = " ".join(line.strip() for line in doc.strip().splitlines() if line.strip())
    if not text:
        return ""
    match = re.search(r"(.+?[.!?])(\s|$)", text)
    return (match.group(1) if match else text).strip()


def guess_category(option_id: str) -> str:
    lowered = option_id.lower()
    for cat, keywords in CATEGORY_KEYWORDS:
        for kw in keywords:
            if kw in lowered:
                return cat
    return "general"


def slug(name: str) -> str:
    s = re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")
    return s or "world"


def choice_label(opt_cls: type, key: str) -> str:
    if getattr(opt_cls, "auto_display_name", False):
        return key.replace("_", " ").title()
    return key


def safe_default_key(opt_cls: type) -> str | None:
    """Return the default Choice key, or None if it can't be resolved."""
    default = getattr(opt_cls, "default", None)
    name_lookup = getattr(opt_cls, "name_lookup", None) or {}
    if default in name_lookup:
        return name_lookup[default]
    if isinstance(default, str):
        return default
    options_map = getattr(opt_cls, "options", None) or {}
    for k, v in options_map.items():
        if v == default:
            return k
    return None


def map_option(option_id: str, opt_cls: type) -> dict[str, Any] | None:
    """Map an Archipelago option class to our schema's `type` block (plus the wrapping dict).

    Returns None to indicate the option should be skipped.
    """
    # Skip Removed/hidden options.
    visibility = getattr(opt_cls, "visibility", None)
    if visibility is not None:
        try:
            if int(visibility) == 0:
                return None
        except Exception:
            pass

    label = humanize(option_id)
    description = first_sentence(opt_cls.__doc__)
    category = guess_category(option_id)

    type_block: dict[str, Any] | None = None
    advanced = False

    # Toggle / DefaultOnToggle
    if isinstance(opt_cls, type) and issubclass(opt_cls, Options.Toggle):
        # Choice subclasses Toggle? No - both inherit NumericOption. Toggle is its own.
        type_block = {"kind": "toggle", "default": bool(getattr(opt_cls, "default", 0))}

    # Choice / TextChoice (TextChoice extends Choice; we still treat as select)
    elif isinstance(opt_cls, type) and issubclass(opt_cls, Options.Choice):
        options_map = getattr(opt_cls, "options", None) or {}
        # Strip alias keys: keep only the canonical name for each id.
        name_lookup = getattr(opt_cls, "name_lookup", None) or {}
        canonical_keys = set(name_lookup.values())
        seen: set[str] = set()
        choices: list[dict[str, str]] = []
        for key, value in options_map.items():
            if canonical_keys and key not in canonical_keys:
                continue
            if key in seen:
                continue
            seen.add(key)
            choices.append({"value": key, "label": choice_label(opt_cls, key)})
        if not choices:
            return None
        default_key = safe_default_key(opt_cls) or choices[0]["value"]
        type_block = {"kind": "select", "default": default_key, "choices": choices}

    # NamedRange / Range
    elif isinstance(opt_cls, type) and issubclass(opt_cls, Options.Range):
        try:
            range_start = int(getattr(opt_cls, "range_start", 0))
            range_end = int(getattr(opt_cls, "range_end", 1))
            default_val = int(getattr(opt_cls, "default", range_start))
        except Exception:
            return None
        type_block = {
            "kind": "range",
            "min": range_start,
            "max": range_end,
            "step": 1,
            "default": default_val,
        }

    # OptionSet / OptionList -> multi_select
    elif isinstance(opt_cls, type) and issubclass(opt_cls, (Options.OptionSet, Options.OptionList)):
        valid_keys = sorted(getattr(opt_cls, "valid_keys", None) or [])
        if not valid_keys:
            # Skip multi-selects with no enumerated choices (item names, plando, etc.)
            return None
        default_raw = getattr(opt_cls, "default", None) or []
        try:
            defaults = sorted(str(v) for v in default_raw)
        except TypeError:
            defaults = []
        type_block = {
            "kind": "multi_select",
            "defaults": defaults,
            "choices": [{"value": k, "label": humanize(k)} for k in valid_keys],
        }
        advanced = True

    # FreeText / OptionString
    elif isinstance(opt_cls, type) and issubclass(opt_cls, Options.FreeText):
        type_block = {
            "kind": "text",
            "default": str(getattr(opt_cls, "default", "") or ""),
            "placeholder": None,
        }

    if type_block is None:
        return None

    return {
        "id": option_id,
        "label": label,
        "description": description,
        "category": category,
        "type": type_block,
        "advanced": advanced,
        "dependencies": None,
    }


def version_string(world_cls: type) -> str:
    rcv = getattr(world_cls, "required_client_version", None)
    if isinstance(rcv, tuple) and rcv:
        return ".".join(str(int(x)) for x in rcv)
    return "0.0.0"


def collect_advanced_option_ids(world_cls: type) -> set[str]:
    """Pull option ids that the world has placed in an obviously-advanced group."""
    advanced: set[str] = set()
    web = getattr(world_cls, "web", None)
    if web is None:
        return advanced
    groups = getattr(web, "option_groups", None) or []
    type_hints = getattr(world_cls.options_dataclass, "type_hints", {}) or {}
    cls_to_id = {cls: name for name, cls in type_hints.items()}
    for group in groups:
        name = (getattr(group, "name", "") or "").lower()
        if "advanced" not in name and "expert" not in name:
            continue
        for opt_cls in getattr(group, "options", []) or []:
            opt_id = cls_to_id.get(opt_cls)
            if opt_id:
                advanced.add(opt_id)
    return advanced


def build_schema(world_id: str, world_cls: type) -> dict[str, Any]:
    options_dc = getattr(world_cls, "options_dataclass", None)
    type_hints: dict[str, type] = {}
    if options_dc is not None:
        type_hints = dict(getattr(options_dc, "type_hints", {}) or {})

    advanced_ids = collect_advanced_option_ids(world_cls)

    options: list[dict[str, Any]] = []
    for option_id, opt_cls in type_hints.items():
        try:
            entry = map_option(option_id, opt_cls)
        except Exception:
            continue
        if entry is None:
            continue
        if option_id in advanced_ids:
            entry["advanced"] = True
        options.append(entry)

    description = first_sentence(world_cls.__doc__) or ""

    return {
        "id": world_id,
        "name": getattr(world_cls, "game", world_id),
        "version": version_string(world_cls),
        "description": description,
        "meta": {
            "rom_required": False,
            "supported_versions": [],
            "wiki_url": None,
            "icon": "mdi:gamepad-variant",
        },
        "options": options,
        "presets": [],
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def list_world_dirs() -> list[str]:
    out: list[str] = []
    for entry in sorted(os.listdir(WORLDS_DIR)):
        full = WORLDS_DIR / entry
        if not full.is_dir():
            continue
        if entry.startswith("_") or entry.startswith("."):
            continue
        if not (full / "__init__.py").exists():
            continue
        out.append(entry)
    return out


def import_world_module(dirname: str) -> tuple[Any, Exception | None]:
    try:
        module = importlib.import_module(f"worlds.{dirname}")
        return module, None
    except BaseException as exc:  # ROM check classes can raise SystemExit etc.
        return None, exc  # type: ignore[return-value]


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Extract Archipelago world schemas.")
    p.add_argument(
        "--out-dir",
        type=Path,
        default=DEFAULT_OUT_DIR,
        help="Directory to write schema JSON files into.",
    )
    p.add_argument(
        "--only",
        default="",
        help="Comma-separated list of world dir names to extract (default: all).",
    )
    p.add_argument(
        "--include-custom",
        action="store_true",
        default=True,
        help="(default) Also scan custom_worlds/ alongside the bundled worlds/.",
    )
    p.add_argument(
        "--ap-root",
        type=Path,
        default=None,
        help=(
            "Archipelago runtime root (directory containing Options.py and "
            "worlds/). Overrides auto-detection; also honored via "
            "$ZEPHYR_AP_ROOT."
        ),
    )
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv if argv is not None else sys.argv[1:])
    out_dir: Path = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    only_filter = {s.strip() for s in args.only.split(",") if s.strip()}

    # Archipelago's own `worlds/__init__.py` scans custom_worlds/ during
    # module import (triggered above by `from worlds.AutoWorld import ...`),
    # so AutoWorldRegister is already populated with any user-installed
    # worlds by the time we get here.

    # Pre-import the worlds package so AutoWorldRegister fills up.
    # Importing each subpackage individually below populates the registry.
    world_dirs = list_world_dirs()
    if args.include_custom and CUSTOM_WORLDS_DIR.exists():
        for entry in sorted(os.listdir(CUSTOM_WORLDS_DIR)):
            full = CUSTOM_WORLDS_DIR / entry
            name = entry
            if entry.endswith(".apworld"):
                name = entry[: -len(".apworld")]
            elif not full.is_dir():
                continue
            if name.startswith(("_", ".")):
                continue
            if name not in world_dirs:
                world_dirs.append(name)
    if only_filter:
        world_dirs = [d for d in world_dirs if d in only_filter]

    # First pass: try importing every world so AutoWorldRegister gets populated.
    import_failures: list[tuple[str, str]] = []
    for d in world_dirs:
        _, err = import_world_module(d)
        if err is not None:
            reason = f"{type(err).__name__}: {err}".splitlines()[0][:200]
            import_failures.append((d, reason))

    # Build a directory -> World class map by inspecting __module__.
    dir_to_world: dict[str, type] = {}
    for game_name, world_cls in AutoWorldRegister.world_types.items():
        module_name = getattr(world_cls, "__module__", "") or ""
        # module_name looks like "worlds.alttp" or "worlds.alttp.<sub>"
        parts = module_name.split(".")
        if len(parts) >= 2 and parts[0] == "worlds":
            dir_name = parts[1]
            # Prefer the first registered world per directory (most have one).
            dir_to_world.setdefault(dir_name, world_cls)

    generated = 0
    skipped: list[tuple[str, str]] = []

    # Track import failures for dirs that produced no world class.
    failed_dirs = {d for d, _ in import_failures}

    for d in world_dirs:
        world_cls = dir_to_world.get(d)
        if world_cls is None:
            reason = next((r for dd, r in import_failures if dd == d), "no World subclass registered")
            skipped.append((d, reason))
            continue
        try:
            world_id = slug(d)
            schema = build_schema(world_id, world_cls)
            out_path = out_dir / f"{world_id}.json"
            with out_path.open("w", encoding="utf-8") as fh:
                json.dump(schema, fh, indent=2, ensure_ascii=False)
                fh.write("\n")
            generated += 1
        except Exception as exc:
            tb = traceback.format_exc(limit=1).splitlines()[-1][:200]
            skipped.append((d, f"build error: {tb}"))

    # Summary
    print()
    print(f"Generated {generated} schemas, skipped {len(skipped)} worlds.")
    if skipped:
        print("\nSkipped worlds:")
        for d, reason in skipped:
            print(f"  - {d}: {reason}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
