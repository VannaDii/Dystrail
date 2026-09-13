"""Refresh route populations from an official Census subcounty release CSV."""
import argparse
import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROFILE_PATH = ROOT / "dystrail-web/static/assets/data/town-profiles.json"
REVIEW_PATH = ROOT / "docs/ux/review-2026-09-11/implementation/content"
ALIASES = {"D.C.": "Washington", "Boise": "Boise City"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("csv", type=Path)
    parser.add_argument("--year", type=int, required=True)
    parser.add_argument("--checked", required=True)
    args = parser.parse_args()
    raw = args.csv.read_bytes()
    rows = list(csv.DictReader(raw.decode("cp1252").splitlines()))
    profiles = json.loads(PROFILE_PATH.read_text())
    field = f"POPESTIMATE{args.year}"
    dataset = f"https://www2.census.gov/programs-surveys/popest/datasets/2020-{args.year}/cities/totals/sub-est{args.year}.csv"
    audit = []
    for profile in profiles:
        name = ALIASES.get(profile["town"], profile["town"].split(",")[0])
        state = "District of Columbia" if profile["state"] == "D.C." else profile["state"]
        matches = [row for row in rows if row["SUMLEV"] == "162"
                   and row["STNAME"] == state and row["NAME"] in (f"{name} city", f"{name} town")]
        if len(matches) != 1:
            raise ValueError(f"Expected one incorporated place for {profile['town']}: {len(matches)}")
        row = matches[0]
        population = int(row[field])
        if population <= 0:
            raise ValueError(f"Missing {field} for {profile['town']}")
        source = profile["population_source"].rsplit("/", 1)[0] + f"/PST0452{str(args.year)[-2:]}"
        profile.update(population=population, population_year=args.year,
                       population_as_of=f"{args.year}-07-01", population_checked=args.checked,
                       population_source=source, population_dataset=dataset,
                       population_fips=row["STATE"] + row["PLACE"])
        audit.append({"town": profile["town"], "name": row["NAME"], "state": state,
                      "fips": profile["population_fips"], "population": population, "source": source})
    PROFILE_PATH.write_text(json.dumps(profiles, ensure_ascii=False, indent=2) + "\n")
    report = {"dataset": dataset, "year": args.year, "as_of": f"{args.year}-07-01",
              "checked": args.checked, "csv_sha256": hashlib.sha256(raw).hexdigest(),
              "definition": "July 1 resident population estimate for the incorporated city, not its metro area.",
              "towns": audit}
    REVIEW_PATH.mkdir(parents=True, exist_ok=True)
    (REVIEW_PATH / "census-populations.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(f"Updated {len(audit)} town profiles from Census Vintage {args.year}.")


if __name__ == "__main__":
    main()
