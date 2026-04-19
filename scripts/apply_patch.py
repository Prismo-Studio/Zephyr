"""
Apply an Archipelago `.apXXX` patch to its base ROM and write the output next
to the patch.

Usage:
    python scripts/apply_patch.py <path/to/AP_whatever_P1_Slot.aplttp>

Prints the produced ROM path on stdout. Exits non-zero with a human-readable
reason on failure. Designed to be invoked from Zephyr's Rust side so games
that use an external client (SNI-based games like A Link to the Past,
Super Metroid, SMW, Yoshi's Island, DKC3, KDL3, Lufia 2 AC, SMZ3) still get
their ROM patched even though no `SNIClient.py` is bundled.

Runs inside the bundled Archipelago runtime's venv so it has every world's
dependencies available. We add the runtime dir to `sys.path` so the world
registry + `AutoPatchRegister` populate the same way `Generate.py` does.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Apply an Archipelago patch file.")
    parser.add_argument("patch", type=Path, help="Path to a .apXXX patch file.")
    parser.add_argument(
        "--runtime",
        type=Path,
        default=None,
        help=(
            "Archipelago runtime directory (the one that contains Generate.py). "
            "Defaults to auto-detection based on this script's location."
        ),
    )
    args = parser.parse_args()

    patch_path: Path = args.patch.resolve()
    if not patch_path.is_file():
        print(f"error: patch not found: {patch_path}", file=sys.stderr)
        return 2

    runtime_dir = args.runtime
    if runtime_dir is None:
        # scripts/apply_patch.py -> <repo>/src-tauri/archipelago-runtime
        here = Path(__file__).resolve().parent
        candidates = [
            here.parent / "src-tauri" / "archipelago-runtime",
            here.parent.parent / "src-tauri" / "archipelago-runtime",
        ]
        runtime_dir = next((c for c in candidates if (c / "Generate.py").exists()), None)
        if runtime_dir is None:
            print(
                "error: could not locate Archipelago runtime directory; pass --runtime",
                file=sys.stderr,
            )
            return 2
    if not (runtime_dir / "Generate.py").exists():
        print(
            f"error: --runtime {runtime_dir} is not an Archipelago runtime (no Generate.py)",
            file=sys.stderr,
        )
        return 2

    # Make the bundled Archipelago importable BEFORE touching anything that
    # depends on it (world registry, Files.AutoPatchRegister).
    sys.path.insert(0, str(runtime_dir))
    os.chdir(str(runtime_dir))

    # Import triggers worlds/__init__.py which populates AutoPatchRegister via
    # the class-definition side effect (each APAutoPatchInterface subclass
    # registers itself by patch_file_ending).
    try:
        import worlds  # noqa: F401
        from worlds.Files import AutoPatchRegister
    except Exception as exc:  # pragma: no cover — surfaces to user
        print(f"error: failed to initialise Archipelago runtime: {exc}", file=sys.stderr)
        return 3

    handler = AutoPatchRegister.get_handler(str(patch_path))
    if handler is None:
        suffix = patch_path.suffix
        print(
            f"error: no patch handler registered for extension '{suffix}' — "
            f"is the world for this patch installed?",
            file=sys.stderr,
        )
        return 4

    # Build the output path next to the patch using the patch class's own
    # `result_file_ending` (".sfc" for SNES games, ".gba" for GBA, etc.).
    result_ext = getattr(handler, "result_file_ending", ".sfc")
    out_path = patch_path.with_suffix(result_ext)

    print(f"applying {patch_path.name} using {handler.__name__} -> {out_path.name}")
    try:
        patch = handler(path=str(patch_path))
        patch.patch(str(out_path))
    except Exception as exc:
        # Surface the cause verbatim — likely "base ROM not configured" or a
        # hash mismatch, both of which tell the user exactly what to fix.
        print(f"error: patch failed: {exc}", file=sys.stderr)
        return 5

    print(str(out_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
