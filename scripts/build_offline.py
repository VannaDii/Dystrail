"""Fingerprint the entire finished Trunk build and emit its offline worker."""
import base64
import hashlib
import json
import os
import re
from pathlib import Path

staging = Path(os.environ["TRUNK_STAGING_DIR"])
index = staging / "index.html"
html = index.read_text()
html = re.sub(r'(<script\b[^>]*type="module"[^>]*>)(?!\s*</script>)', r'\1\nawait window.dystrailLaunch;\n', html)
locale_dir = Path(__file__).resolve().parents[1] / "dystrail-web/i18n"
copy = {lang: json.loads((locale_dir / f"{lang}.json").read_text())["offline"] for lang in ["en", "it", "es", "ar"]}
html = html.replace('<script src="static/offline-client.js">', '<script>window.dystrailOfflineCopy=' + json.dumps(copy).replace('<', '\\u003c') + ';</script><script src="static/offline-client.js">')
index.write_text(html)
assets = []
for path in sorted(staging.rglob("*")):
    if not path.is_file() or path.name in {"sw.js", "offline-manifest.json"}:
        continue
    content = path.read_bytes()
    assets.append({
        "path": path.relative_to(staging).as_posix(),
        "bytes": len(content),
        "integrity": "sha256-" + base64.b64encode(hashlib.sha256(content).digest()).decode(),
    })
revision = hashlib.sha256(json.dumps(assets, sort_keys=True).encode()).hexdigest()[:20]
manifest = {"revision": revision, "bytes": sum(a["bytes"] for a in assets), "assets": assets}
template = Path(__file__).with_name("offline-worker.js").read_text()
(staging / "sw.js").write_text("const BUILD = " + json.dumps(manifest) + ";\n" + template)
(staging / "offline-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(f"Offline build {revision}: {len(assets)} files, {manifest['bytes'] / 1_000_000:.1f} MB")
