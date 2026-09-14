# Offline fixture correction

The original complete run passed 412 of 414 browser cases. Both failures were the same cache-eviction test on desktop and mobile. Server-side instrumentation confirmed that Chromium's headless shell fetched the evicted image while the context reported offline, so the reachable fixture repaired it and the loading gate closed. Full Chrome blocked that request.

The fixture now refuses connections during offline phases, verifies navigator.onLine is false, confirms actual cache eviction, and arms the held repair before restoring connectivity. Every original assertion and time limit remains, including the 12-second Retry expectation. Both affected headless cases pass against the exact d6fc9d303f700bc08f88 artifact. Product code is unchanged.

The CI command now uses the configured retain-on-failure tracing, and video uses the same retention policy. All tests and explicit screenshots remain. The prior forced trace-on setting produced a 5.85 GB archive for one shard by embedding full asset downloads for passing tests.

Final source: eaeceeb2f67207df8c91fd87569937ae45ce63b8
Final full CI: https://github.com/VannaDii/Dystrail/actions/runs/34790458113
