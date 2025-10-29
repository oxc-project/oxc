"""LLDB Rust pretty-printer bootstrap for CodeLLDB.

Loads official Rust formatters (lldb_lookup + lldb_commands) without hardcoded toolchain paths.
Adds summaries for core collection types and adjusts string length. Safe to re-run.
"""
from __future__ import annotations
import os, sys, shutil, subprocess

def log(*parts: object) -> None:
    print("[lldb_rust_init]", *parts)

# 1) Ensure ~/.cargo/bin on PATH so rustc is discoverable when launched via CodeLLDB.
home = os.environ.get("HOME", "")
if home:
    cargo_bin = os.path.join(home, ".cargo", "bin")
    path = os.environ.get("PATH", "")
    parts = path.split(os.pathsep) if path else []
    if cargo_bin not in parts:
        # Prepend cargo_bin preserving existing PATH using the correct platform-specific separator.
        os.environ["PATH"] = os.pathsep.join([cargo_bin] + parts)
        log("PATH prepended with", cargo_bin)
else:
    log("HOME unset; skipping PATH prepend")

# 2) Locate rustc & sysroot.
rustc = shutil.which("rustc")
if not rustc:
    log("rustc NOT FOUND; aborting formatter init")
    raise SystemExit
log("rustc ->", rustc)
try:
    sysroot = subprocess.check_output([rustc, "--print", "sysroot"], text=True).strip()
except Exception as e:  # noqa: BLE001
    log("Failed to get sysroot:", e)
    raise SystemExit
log("sysroot ->", sysroot)

etc_dir = os.path.join(sysroot, "lib", "rustlib", "etc")
if not os.path.isdir(etc_dir):
    log("Missing etc dir:", etc_dir)
    raise SystemExit
log("Loading Rust formatters from", etc_dir)

# 3) Import lldb_lookup & source lldb_commands via LLDB command API.
if etc_dir not in sys.path:
    sys.path.append(etc_dir)
try:
    import lldb_lookup  # type: ignore
    log("Imported lldb_lookup OK")
except Exception as e:  # noqa: BLE001
    log("Import lldb_lookup FAILED:", e)
    raise SystemExit

# Acquire lldb debugger object from injected global 'lldb' (provided by CodeLLDB environment).
try:
    import lldb  # type: ignore
except Exception as e:  # noqa: BLE001
    log("Unable to import lldb module (unexpected):", e)
    raise SystemExit

dbg = lldb.debugger

# Source the static commands file for additional summaries (matches rust-lldb behavior).
commands_path = os.path.join(etc_dir, "lldb_commands")
if os.path.isfile(commands_path):
    dbg.HandleCommand(f"command source -s 0 {commands_path}")
    log("Sourced", commands_path)
else:
    log("Commands file not found:", commands_path)

# Enable Rust category & increase max string summary length.
dbg.HandleCommand("type category enable Rust")
dbg.HandleCommand("settings set target.max-string-summary-length 2000")

# Register Vec printers explicitly (defensive if commands file changes in future).
dbg.HandleCommand(
    'type synthetic add -l lldb_lookup.synthetic_lookup -x "^(alloc::([a-z_]+::)+)Vec<.+>$" --category Rust'
)
dbg.HandleCommand(
    'type summary add -F lldb_lookup.summary_lookup -e -x -h "^(alloc::([a-z_]+::)+)Vec<.+>$" --category Rust'
)

# Provide a concise summary for URI types (lsp_types::Url / fluent_uri::Uri wrappers).
# These commonly contain an inner String we want to display directly.
# We attempt a regex that matches types ending with `::Uri` or `::Url` and which have a
# single field referencing alloc::string::String.
def uri_summary(val_obj):  # noqa: D401
    """LLDB summary callback for various Uri/Url wrapper types.

    Tries to locate an inner alloc::string::String and return its contents.
    Falls back to existing lldb_lookup.summary_lookup if structure differs.
    """
    try:
        # Heuristics: search immediate children then recurse one level.
        for child in val_obj.children:
            ty = child.type.name or ""
            if "alloc::string::String" in ty:
                # Use the default Rust String summary by delegating to lldb_lookup.
                import lldb_lookup  # type: ignore
                return lldb_lookup.summary_lookup(child)
        # Recurse one level for wrappers like tuple struct or newtype.
        for child in val_obj.children:
            for gchild in child.children:
                ty = gchild.type.name or ""
                if "alloc::string::String" in ty:
                    import lldb_lookup  # type: ignore
                    return lldb_lookup.summary_lookup(gchild)
    except Exception as e:  # noqa: BLE001
        return f"<uri err: {e}>"
    return "<uri>"

try:
    dbg.HandleCommand(
        'type summary add -e -x -F lldb_rust_init.uri_summary "^(.*(::)+)(Url|Uri)$" --category Rust'
    )
    log("Registered custom Url/Uri summary")
except Exception as e:
    log("Failed to register custom Url/Uri summary:", e)

log("Rust formatter initialization complete")
