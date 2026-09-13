# Scene asset inventory

Generated with the image-generation tool, reviewed against the existing cartoon pixel-art references, and copied without raster post-processing. The normal three-row van supersedes the rejected stretched van. The convoy was regenerated to remove realistic truck rendering.

The runtime uses six transparent busts, the empty van body and the approved full-crew alpha silhouette. Encounter subjects and route assignments are documented in [stage-map.md](stage-map.md).

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| `enc-bridge.png` | 2,647,779 | `296c58fdb7ab945a27332f6eb60845e0064793a392d7c94b5ba85ff30fde74c3` |
| `enc-checkpoint.png` | 2,283,910 | `2ba07cb7c6d7cf90efa88537f695ecb3ac1be53d57402fc9a38184d06de04f4b` |
| `enc-civic.png` | 1,947,370 | `0790094ee6f5c5a198f9520a98642650804151e9924f3cc9790a1c2e9b708af0` |
| `enc-clinic.png` | 2,283,216 | `33981456fe2b66bcc5659e480416bd2c10d73e0cbc6f565ed294f13e2e2d905a` |
| `enc-community.png` | 2,346,165 | `cfd788be9a4a799d9dce393eb56e74bd0f8d262ca03eff65df8006fb96dae2f1` |
| `enc-convoy.png` | 1,626,803 | `744077d6620e2a7a3ef63f8f1b4f7c8807c6791ff21374412a5ea48237431f3a` |
| `enc-media-workshop.png` | 2,023,985 | `7ef193d23c53015a2c940c2635440970d00e877a5bf077f01197029f82bc2db6` |
| `enc-night-briefing.png` | 2,246,195 | `53ce6b23df3743b5866366b7f1ff7637a52f76ae4cf64c6e27abf1e6e524283f` |
| `enc-radio.png` | 2,020,811 | `c6063d41c3da32f90fe0aeec339f95b6dd5c22d6aa42575a3ae2c9cdeaf5f070` |
| `enc-service.png` | 2,315,284 | `a02d770c28d2ef4aecb999d882cd6d36e73031112d21166a6fec943826854afe` |
| `enc-street.png` | 2,418,739 | `794698037295be8fa4f77b3e827493d90cf09762023d9b8c345116c2269fc1c3` |
| `occupant-journalist.png` | 923,191 | `453c6a65d35433fb0150ee5e2d5fbad2d1a82e2732278eb0c9a222dcd08da90f` |
| `occupant-lobbyist.png` | 704,662 | `f25d777ab60220c89472f70b50139beab163efd9a1dd2b9378e8364733ede02b` |
| `occupant-organizer.png` | 918,041 | `9db44253e4bb0d0a850a0b1bd37ce9b6c740c29acece25e92df7a69f32d1fcd1` |
| `occupant-satirist.png` | 861,064 | `1a1a13de2dfd187a998f0b209a8927f83ee7722ffc825b7068ab4e9ded588230` |
| `occupant-staffer.png` | 830,029 | `a4a52c97a0ecd871c545cd8a24cc8bf6ac0fb1cbbc5c5c329b20122eaf39413d` |
| `occupant-whistleblower.png` | 791,621 | `b30c7280deaa25df94f30f58dfb6e0076820f2c1ff741b7596866f2114385668` |
| `open-beltway-parkway.png` | 2,185,024 | `c26e9df42322a201f6aeb16e091a65b4811342d23240c6eee25ec73fa3be40cc` |
| `open-beltway-suburbs.png` | 2,101,757 | `01a34e87754c95b21d8c95a37496d4a8072355c9c96f8f84303276c4a0f91eff` |
| `open-heartland-orchard.png` | 2,072,146 | `48886bd79a046e5bb774f7af26592e6e37fa618b57607c1254648f0d23f44f88` |
| `open-heartland-prairie.png` | 2,060,522 | `de63d22e3c41a16b51db71c07504d23dda4f498daabdc44ea18658a6b80e50cb` |
| `open-rustbelt-foundry.png` | 2,243,397 | `95f3297657c71a1bccc77a821ce67c001b0c1359489ef6a73c864cae1f417017` |
| `open-rustbelt-lakeside.png` | 2,191,114 | `448ed2d15b9f3078b42e1f0f00ce51fc07fe9048aa9e69430ea2843952695eb1` |
| `van-crew.png` | 1,442,842 | `9a6e2c8aa3a073a3dce75c13f2bba175f0b777e0448c063a33e8a2fb615f1dba` |
| `van-empty.png` | 1,665,790 | `8e66cf17ba27a60fd82e92b81a8c80bcb389873fd2b638a123382144a26cc322` |

## Geographic route revision

- `open-appalachian-ridge.png`: generated in the established chunky 16-bit road style, visually checked before use. Wooded rounded Appalachian ridges, exposed rock cuts, flat side-view road, no baked crew or vehicles. Covers the Pittsburgh–Cumberland–Hagerstown approach.
- `static/img/map/us-states.svg`: authored presentation of Census state geometry. Route overlays use OSM roads; [provenance and reproducible inputs](map-sources/README.md). Pixel satire icons are authored SVG shapes.

Appalachian asset SHA-256: `52a50ccc02cac320ac2e343b93f367c84deba48c1c366ffbd2acaacad0361aa5`.
