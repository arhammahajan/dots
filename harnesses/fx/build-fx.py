#!/usr/bin/env python3
"""Build the fx effort fork and install the binary used by the router launcher."""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import urllib.request

SOURCE_URL = "https://github.com/arhammahajan/fx.git"
SOURCE_BRANCH = "main"
ZIG_SHA256 = "1cbe9df9f27e6b78d14ccbca43b6703a404ef79ef1c463de901d7f088d4e2026"
ZIG_URL = "https://ziglang.org/download/0.17.0/zig-x86_64-linux-0.17.0.tar.xz"


def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def main():
    root = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share")) / "fx-router"
    root.mkdir(parents=True, exist_ok=True)
    source = Path(os.environ.get("FX_SOURCE_DIR", Path.home() / "dev/fx")).expanduser()
    if not source.exists():
        source.parent.mkdir(parents=True, exist_ok=True)
        run("git", "clone", "--branch", SOURCE_BRANCH, SOURCE_URL, str(source))
    branch = subprocess.check_output(["git", "branch", "--show-current"], cwd=source, text=True).strip()
    if branch != SOURCE_BRANCH:
        raise SystemExit(f"Expected branch {SOURCE_BRANCH!r} in {source}; switch branches before building.")
    toolchain = root / "toolchain"
    compiler = toolchain / "zig-x86_64-linux-0.17.0/zig"
    if not compiler.exists():
        toolchain.mkdir(parents=True, exist_ok=True)
        archive = toolchain / "zig-0.17.0.tar.xz"
        urllib.request.urlretrieve(ZIG_URL, archive)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != ZIG_SHA256:
            raise SystemExit("Zig download checksum mismatch.")
        with tarfile.open(archive) as package:
            package.extractall(toolchain, filter="data")
    run(str(compiler), "fmt", "--check", "src/core/config/configured_provider.zig",
        "src/gateway/chat_completions.zig", "src/gateway/chat_completions_protocol.zig", cwd=source)
    run(str(compiler), "build", "-Doptimize=ReleaseSafe", "-j4", cwd=source)
    target = Path.home() / ".local/libexec/fx/fx-router"
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = target.with_name("fx-router.new")
    shutil.copy2(source / "zig-out/bin/fx", temporary)
    temporary.chmod(0o755)
    temporary.replace(target)
    print(f"Installed fx from {source} at {target}")


if __name__ == "__main__":
    main()
