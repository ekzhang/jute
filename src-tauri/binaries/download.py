"""Python script that downloads/populates sidecar binaries for app build.

This script can be run using vanilla Python 3.13. Avoid using any external
modules that are not in the standard library, for portability. It's just a
simple download that should be done once.

Each binary that is bundled with the app must be generated for every target
triple that Jute runs on. See https://v2.tauri.app/develop/sidecar/ for details.
This allows us to embed external binaries across platforms that can be called
from JavaScript or Rust code.
"""

import os
import tarfile
import urllib.request
import zipfile
from io import BytesIO
from pathlib import Path
from typing import IO

# Which version of uv to ship with the app.
UV_VERSION = "0.5.13"

# Supported target triples that sidecars are generated for.
TARGET_TRIPLES = [
    "aarch64-apple-darwin",  # macOS Apple Silicon
    "x86_64-apple-darwin",  # macOS Intel
    "i686-pc-windows-msvc",  # Windows 32-bit
    "x86_64-pc-windows-msvc",  # Windows 64-bit
    "aarch64-unknown-linux-musl",  # Linux ARM64 (musl)
    "x86_64-unknown-linux-musl",  # Linux x86_64 (musl)
    "i686-unknown-linux-musl",  # Linux x86 (musl)
]


def read_archive(
    name: str, fileobj: IO[bytes], archive_paths: list[str]
) -> list[bytes]:
    """Extracts certain files within a tarball or zip archive."""
    if name.endswith(".tar.gz") or name.endswith(".tgz"):
        with tarfile.open(name, "r:*", fileobj=fileobj) as tar:
            results: list[bytes] = []
            for path in archive_paths:
                if fileobj := tar.extractfile(path):
                    with fileobj:
                        results.append(fileobj.read())
                else:
                    raise FileNotFoundError(f"File not found in archive: {path}")
            return results

    elif name.endswith(".zip"):
        with zipfile.ZipFile(fileobj, "r") as zip:
            results: list[bytes] = []
            for path in archive_paths:
                with zip.open(path) as f:
                    results.append(f.read())
            return results

    else:
        raise ValueError(f"Unsupported archive format: {name!r}")


def display_file(path: str):
    size = os.stat(path).st_size
    if size < 1024:
        size_str = f"{size} B"
    elif size < 1024**2:
        size_str = f"{size / 1024:.2f} KiB"
    else:
        size_str = f"{size / 1024**2:.2f} MiB"
    print(f"  - {path} ({size_str})")


def download_uv(target: str, version: str = UV_VERSION) -> None:
    url = "https://github.com/astral-sh/uv/releases/download/"
    if "windows" in target:
        url += f"{version}/uv-{target}.zip"
        archive_paths = ["uv.exe", "uvx.exe"]
    else:
        url += f"{version}/uv-{target}.tar.gz"
        archive_paths = [f"uv-{target}/uv", f"uv-{target}/uvx"]

    with urllib.request.urlopen(url) as resp:
        archive_data = resp.read()

    uv_binary, _uvx_binary = read_archive(url, BytesIO(archive_data), archive_paths)
    Path(f"uv-{target}").write_bytes(uv_binary)
    display_file(f"uv-{target}")


def main() -> None:
    os.chdir(Path(__file__).resolve().parent)

    for target in TARGET_TRIPLES:
        print(f"Downloading sidecar binaries for {target}...")
        download_uv(target)


if __name__ == "__main__":
    main()
