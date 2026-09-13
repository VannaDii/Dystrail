#!/usr/bin/env python3
"""Launch macOS Chrome without lending Playwright's log pipes to updater helpers.

Chrome may pass stdout/stderr to detached background processes. Playwright waits
for those pipes to close, even after Chrome exits (microsoft/playwright#39736).
Use a regular log file for Chrome and forward its contents while Chrome is alive.
The debugging pipes and the browser's actual exit status are preserved.
"""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def main():
    chrome = os.environ.get(
        "DYSTRAIL_CHROME_EXECUTABLE",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    )
    debug_fds = []
    for fd in (3, 4):
        try:
            os.fstat(fd)
        except OSError:
            continue
        debug_fds.append(fd)

    with tempfile.TemporaryDirectory(prefix="dystrail-chrome-log-") as directory:
        path = Path(directory) / "browser.log"
        with path.open("wb") as output:
            browser = subprocess.Popen(
                [chrome, *sys.argv[1:]],
                stdout=output,
                stderr=subprocess.STDOUT,
                pass_fds=tuple(debug_fds),
            )
            # Separate file descriptions keep the writer and reader offsets apart.
            with path.open("rb") as log:
                while True:
                    data = log.read(65536)
                    if data:
                        sys.stderr.buffer.write(data)
                        sys.stderr.buffer.flush()
                    elif browser.poll() is not None:
                        break
                    else:
                        time.sleep(0.02)
            code = browser.wait()
    return code if code >= 0 else 128 - code


if __name__ == "__main__":
    sys.exit(main())
