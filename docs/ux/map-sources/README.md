# Geographic inputs

Retrieved September 11, 2026. These files are geographic datasets, not vendored library code. The game makes no map or routing network requests.

- **Boundaries:** [U.S. Census 2024 cartographic state boundaries, 1:20 million KML](https://www2.census.gov/geo/tiger/GENZ2024/kml/cb_2024_us_state_20m.zip). Federal public geographic data, simplified for cartographic display. Fifty states plus the District of Columbia; Alaska and Hawaii appear in separate insets. These are not cadastral or navigation-grade boundaries.
- **Roads:** [OpenStreetMap contributors](https://www.openstreetmap.org/copyright), served by the [OSRM public routing service](https://project-osrm.org/docs/v5.24.0/api/#route-service). OSM data is available under ODbL. Attribution is displayed in every map scene. Each compressed response retains the returned full driving geometry and distance annotations; the adjacent cities file records the requested waypoint order. The generated road database `dystrail-game/data/routes.json` is an adapted OSM database under ODbL; unrelated game code/art retain their own licenses.

The current six West Coast itineraries are in `west-coast-v1/`; the earlier root-level road responses remain historical source evidence. `west-coast-v1/checksums.json` records SHA-256 hashes of the uncompressed source bytes. Compressed sources allow reproducible regeneration without contacting the routing service:

```sh
python3 scripts/build_geography.py
```

The authored stdlib-only generator applies one spherical Albers equal-area projection to the lower 48 boundaries, city points and road geometry (central meridian −96°, latitude of origin 37.5°, standard parallels 29.5° and 45.5°). Alaska/Hawaii have separate inset transforms. Route points are sampled at roughly half a display pixel, preserving every service waypoint. Road lengths use the original full-resolution distance annotations, not straight-line distances between towns.

To refresh roads explicitly, run `python3 scripts/fetch_routes.py --output /tmp/dystrail-geography`, copy the retained Census `states.kml.gz` into that directory, and run `python3 scripts/build_geography.py --inputs /tmp/dystrail-geography`. Review route changes before replacing the retained inputs and checksums. The public service is not a runtime dependency and its future responses need not be byte-identical.

Routes intentionally visit the named story towns; they are not promises of the shortest possible trip. Physical road miles map proportionally to the existing simulation progress budget so distinct origins do not silently change the strategy's configured total turn distance. All player-facing geographic mileage, location, map progress, town services and regional art use the same physical route. New sessions initialize the strategy distance before the first map render. No legacy-route compatibility layer is required or shipped.
