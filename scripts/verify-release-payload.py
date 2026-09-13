"""Verify the complete prebuilt Pages payload before publication."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
payload = root / "release-site"
manifest = json.loads((root / "release-manifest.json").read_text())
assert manifest["revision"] == "08a07362f3e5c5445818", "Unexpected preview revision"
files = {str(path.relative_to(payload)): path for path in payload.rglob("*") if path.is_file()}
assert set(files) == set(manifest["files"]), "Payload inventory differs from reviewed artifact"
for name, path in files.items():
    assert not path.is_symlink(), f"Unexpected symlink: {name}"
    assert hashlib.sha256(path.read_bytes()).hexdigest() == manifest["files"][name], name
assert (payload / "CNAME").read_text().strip() == "dystrail.com"
assert (payload / "404.html").read_bytes() == (payload / "play/index.html").read_bytes()
assert manifest["revision"] in (payload / "play/sw.js").read_text()
print(f"Verified {len(files)} files for preview {manifest['revision']}")
