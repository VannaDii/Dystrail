"""Verify exact published bytes, offline integrity, and preserved website files."""
import base64
import hashlib
import json
import re
from pathlib import Path

root = Path(__file__).resolve().parents[1]
payload = root / "release-site"
manifest = json.loads((root / "release-manifest.json").read_text())
assert manifest["revision"] == "8b8fcc3f7004fd8ec354", "Unexpected preview revision"
files = {str(path.relative_to(payload)): path for path in payload.rglob("*") if path.is_file()}
assert set(files) == set(manifest["files"]), "Payload inventory differs from reviewed artifact"
assert all(not path.is_symlink() for path in payload.rglob("*")), "Unexpected symlink"
for name, path in files.items():
    assert hashlib.sha256(path.read_bytes()).hexdigest() == manifest["files"][name], name
for name, expected in manifest["preserved_files"].items():
    assert manifest["files"][name] == expected, name
for name, expected in manifest["preserved_hashed_assets"].items():
    assert manifest["files"][name] == expected, name
assert (payload / "CNAME").read_text().strip() == "dystrail.com"
assert (payload / "404.html").read_bytes() == (payload / "play/index.html").read_bytes()
offline_bytes = (payload / "play/offline-manifest.json").read_bytes()
assert hashlib.sha256(offline_bytes).hexdigest() == manifest["artifact_manifest_sha256"]
offline = json.loads(offline_bytes)
assert offline["revision"] == manifest["revision"]
worker = (payload / "play/sw.js").read_text()
assert json.loads(re.search(r"^const BUILD = (.*);$", worker, re.M).group(1)) == offline
for asset in offline["assets"]:
    data = (payload / "play" / asset["path"]).read_bytes()
    assert len(data) == asset["bytes"], asset["path"]
    assert "sha256-" + base64.b64encode(hashlib.sha256(data).digest()).decode() == asset["integrity"], asset["path"]
print(f"Verified {len(files)} files and {len(offline['assets'])} offline assets for preview {manifest['revision']}")
