"""Serve the CI release artifact, including client-side routes, for browser tests."""
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import sys
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1] / "dist"


class ReleaseHandler(SimpleHTTPRequestHandler):
    def translate_path(self, path):
        route = unquote(urlsplit(path).path).removeprefix("/play/")
        target = (ROOT / route.lstrip("/")).resolve()
        if not target.is_relative_to(ROOT):
            return str(ROOT / "404.html")
        if not target.suffix and not target.is_file():
            target = ROOT / "index.html"
        return str(target)

    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()


if __name__ == "__main__":
    assert (ROOT / "index.html").is_file(), "Download the release artifact first"
    ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), ReleaseHandler).serve_forever()
