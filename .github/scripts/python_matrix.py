"""Select stable CPython builds allowed by the package's Python requirement."""

# ruff: noqa: INP001 -- This executable CI script is not an importable package.

import json
import os
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

from packaging.specifiers import SpecifierSet
from packaging.version import Version

PLATFORMS = (
	{
		"name": "linux-x64",
		"runner": "ubuntu-24.04",
		"os": "linux",
		"arch": "x86_64",
		"libc": "gnu",
		"target": "x86_64-unknown-linux-gnu",
	},
	{
		"name": "linux-arm64",
		"runner": "ubuntu-24.04-arm",
		"os": "linux",
		"arch": "aarch64",
		"libc": "gnu",
		"target": "aarch64-unknown-linux-gnu",
	},
	{
		"name": "windows-x64",
		"runner": "windows-latest",
		"os": "windows",
		"arch": "x86_64",
		"libc": "none",
		"target": "x86_64-pc-windows-msvc",
	},
	{
		"name": "windows-arm64",
		"runner": "windows-11-arm",
		"os": "windows",
		"arch": "aarch64",
		"libc": "none",
		"target": "aarch64-pc-windows-msvc",
	},
	{
		"name": "macos-arm64",
		"runner": "macos-latest",
		"os": "macos",
		"arch": "aarch64",
		"libc": "none",
		"target": "aarch64-apple-darwin",
	},
)


def _available_releases(
	downloads: list[dict[str, str]],
	requirement: SpecifierSet,
) -> dict[tuple[str, str, str, tuple[int, int], str], Version]:
	"""Find the latest matching patch of each stable interpreter and platform."""
	available = {}
	for download in downloads:
		if download["implementation"] != "cpython":
			continue
		version = Version(download["version"])
		if version.is_prerelease or version.is_devrelease or not requirement.contains(version):
			continue
		minor = (version.major, version.minor)
		variant = download["variant"]
		if variant not in {"default", "freethreaded"}:
			continue
		# Python 3.13t was experimental, rather than an officially supported variant.
		if variant == "freethreaded" and minor < (3, 14):
			continue
		key = (download["os"], download["arch"], download["libc"], minor, variant)
		available[key] = max(version, available.get(key, version))
	return available


def make_matrix(
	downloads: list[dict[str, str]],
	requires_python: str,
) -> dict[str, list[dict[str, str]]]:
	"""Cover each supported minor and platform using its latest compatible patch."""
	available = _available_releases(downloads, SpecifierSet(requires_python))
	minors = {key[3] for key in available if key[4] == "default"}
	free_threaded = {key[3] for key in available if key[4] == "freethreaded"}

	if not minors:
		msg = f"No stable CPython releases match requires-python = {requires_python!r}"
		raise ValueError(msg)

	entries = []
	for platform in PLATFORMS:
		for minor in sorted(minors):
			variants = ("default", "freethreaded") if minor in free_threaded else ("default",)
			for variant in variants:
				key = (platform["os"], platform["arch"], platform["libc"], minor, variant)
				suffix = "t" if variant == "freethreaded" else ""
				label = f"{minor[0]}.{minor[1]}{suffix}"
				if key not in available:
					msg = f"No compatible Python {label} download for {platform['name']}"
					raise ValueError(msg)
				tag = f"cp{minor[0]}{minor[1]}"
				entries.append(
					{
						**platform,
						"python": f"{available[key]}{suffix}",
						"label": label,
						"interpreter": {
							"linux": f"/opt/python/{tag}-{tag}{suffix}/bin/python",
							"windows": ".venv/Scripts/python.exe",
							"macos": ".venv/bin/python",
						}[platform["os"]],
					}
				)
	return {"include": entries}


def main() -> None:
	"""Discover uv's available interpreters and emit a GitHub Actions matrix."""
	project = tomllib.loads(Path("pyproject.toml").read_text())
	result = subprocess.run(  # noqa: S603 -- The executable and arguments are fixed by this script.
		[
			shutil.which("uv") or "uv",
			"python",
			"list",
			"--only-downloads",
			"--all-versions",
			"--all-platforms",
			"--all-arches",
			"--output-format",
			"json",
		],
		check=True,
		capture_output=True,
		text=True,
	)
	# Example output:
	# {
	#   "key": "cpython-3.15.0rc2-linux-x86_64-gnu",
	#   "version": "3.15.0rc2",
	#   "version_parts": {
	#     "major": 3,
	#     "minor": 15,
	#     "patch": 0
	#   },
	#   "path": null,
	#   "symlink": null,
	#   "url": "https://releases.astral.sh/github/python-build-standalone/releases/download/20260924/cpython-3.15.0rc2%2B20260924-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz",
	#   "os": "linux",
	#   "variant": "default",
	#   "implementation": "cpython",
	#   "arch": "x86_64",
	#   "libc": "gnu"
	# }
	matrix = make_matrix(json.loads(result.stdout), project["project"]["requires-python"])
	encoded = json.dumps(matrix, separators=(",", ":"))
	if output_path := os.environ.get("GITHUB_OUTPUT"):
		with Path(output_path).open("a") as output:
			output.write(f"matrix={encoded}\n")
	sys.stdout.write(f"{encoded}\n")


if __name__ == "__main__":
	main()
