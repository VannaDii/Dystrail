"""Apply editorial copy from a saved Google Docs paragraph response, retaining game effects."""
import argparse
import copy
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "dystrail-web/static/assets/data"
LOCALES = ROOT / "dystrail-web/i18n"
OUT = ROOT / "docs/ux/review-2026-09-11/implementation/content"


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def mechanics(events):
    snapshot = copy.deepcopy(events)
    for event in snapshot:
        for field in ("name", "desc"):
            event.pop(field, None)
        for choice in event["choices"]:
            choice.pop("label", None)
            choice["effects"].pop("log", None)
    return snapshot


def dotted_value(locale, key):
    section, field = key.split(".")
    if section not in ("trail", "ally_loss", "journey"):
        raise ValueError(f"Unsupported narrative key: {key}")
    return locale[section][field]


def set_dotted(locale, key, value):
    section, field = key.split(".")
    locale[section][field] = value


def placeholders(value):
    return Counter(re.findall(r"\{[a-z_]+\}", value))


def apply_narrative(source, args, towns, locales, covered_towns):
    """Apply reviewed named-person adaptations, without importing editorial instructions."""
    if not args.narrative_audit or not args.narrative_translations:
        raise ValueError("Narrative copy requires its source audit and complete translations.")
    narrative = read(args.narrative_copy)
    audit = read(args.narrative_audit)
    translations = read(args.narrative_translations)
    if audit["source_revision"] != source["revisionId"]:
        raise ValueError("Narrative mappings must be reviewed against this source revision.")
    if set(narrative) != {"town_comments", "locale_copy"} or set(translations) != {"it", "es", "ar"}:
        raise ValueError("Expected town/locale copy and all three supported translations.")
    audited_copy = {item["key"]: item["after"] for item in audit["i18n"]}
    if narrative["locale_copy"] != audited_copy:
        raise ValueError("Narrative copy differs from the reviewed identity and cost adaptations.")
    for item in audit["i18n"]:
        paragraph = source["paragraphs"][item["source_paragraph"]]
        if paragraph["text"].strip() != item["source_text"]:
            raise ValueError(f"Narrative source changed: {item['key']}")
    if set(narrative["town_comments"]) != set(covered_towns):
        raise ValueError("Town narrative mapping must cover every reviewed town.")
    for item in audit["towns"]:
        line = source["paragraphs"][item["source_paragraph"]]["text"].strip()
        if narrative["town_comments"][item["town"]] != line or item["after"] != line:
            raise ValueError(f"Town narrative source changed: {item['town']}")
    for language, sections in translations.items():
        if set(sections) != set(narrative):
            raise ValueError(f"Incomplete narrative translation: {language}")
        for section, fields in narrative.items():
            if set(sections[section]) != set(fields):
                raise ValueError(f"Incomplete narrative translation: {language}/{section}")
            for key, value in fields.items():
                translated = sections[section][key]
                if not isinstance(translated, str) or not translated.strip():
                    raise ValueError(f"Empty narrative translation: {language}/{key}")
                if placeholders(translated) != placeholders(value) or translated.count(" · ") != value.count(" · "):
                    raise ValueError(f"Narrative placeholders/subtitles changed: {language}/{key}")
    english = copy.deepcopy(locales[LOCALES / "en.json"])
    changed = {"town_comments": [], "locale_copy": []}
    for town in towns:
        if town["town"] not in narrative["town_comments"]:
            continue
        value = narrative["town_comments"][town["town"]]
        previous = town["comment"]["en"]
        if previous != value:
            changed["town_comments"].append(town["town"])
        for language, text in town["comment"].items():
            if language == "en" or text == previous:
                town["comment"][language] = value
        for language, sections in translations.items():
            town["comment"][language] = sections["town_comments"][town["town"]]
    for key, value in narrative["locale_copy"].items():
        previous = dotted_value(english, key)
        if not isinstance(value, str) or not value.strip() or set(placeholders(previous)) != set(placeholders(value)):
            raise ValueError(f"Narrative identity/stat placeholders changed: {key}")
        if previous != value:
            changed["locale_copy"].append(key)
        for path, locale in locales.items():
            if path.stem == "en" or dotted_value(locale, key) == previous:
                set_dotted(locale, key, value)
        for language, sections in translations.items():
            set_dotted(locales[LOCALES / f"{language}.json"], key, sections["locale_copy"][key])
    return changed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--rename", action="append", default=[], metavar="TITLE=ID")
    parser.add_argument("--translations", type=Path)
    parser.add_argument("--narrative-copy", type=Path)
    parser.add_argument("--narrative-audit", type=Path)
    parser.add_argument("--narrative-translations", type=Path)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--report", type=Path, default=OUT / "workshop-sync.json")
    args = parser.parse_args()
    source = read(args.source)
    paragraphs = source["paragraphs"]
    events = read(DATA / "game.json")
    before = mechanics(events)
    originals = copy.deepcopy(events)
    names = {event["name"]: event for event in events}
    by_id = {event["id"]: event for event in events}
    for rename in args.rename:
        title, event_id = rename.rsplit("=", 1)
        if event_id not in by_id or (title in names and names[title]["id"] != event_id):
            raise ValueError(f"Unrecognized or ambiguous rename: {rename}")
        names[title] = by_id[event_id]
    covered, placeholders, changes = [], [], {}
    for index, paragraph in enumerate(paragraphs):
        title = paragraph["text"].strip()
        if title not in names:
            continue
        event = names[title]
        if event["id"] in covered:
            raise ValueError(f"Duplicate workshop encounter: {event['id']}")
        stop = index + 1
        while (stop < len(paragraphs) and paragraphs[stop].get("tabId") == paragraph.get("tabId")
               and paragraphs[stop]["text"].strip() not in names):
            stop += 1
        block = [p["text"].strip() for p in paragraphs[index + 1:stop]]
        lead_start = next(i + 1 for i, text in enumerate(block[:-1])
                          if text in ("Lead-in", "Choose or rewrite the lead-in"))
        lead_end = next(i for i in range(lead_start, len(block))
                        if block[i].startswith("Choices"))
        lead = "\n\n".join(block[lead_start:lead_end])
        if not lead:
            raise ValueError(f"Missing lead-in: {event['id']}")
        pairs = [text.split(" → ", 1) for text in block
                 if " → " in text and not text.startswith("Choices")]
        if len(pairs) != len(event["choices"]):
            raise ValueError(f"Choice count changed for {event['id']}")
        covered.append(event["id"])
        if event["name"] != title:
            event["name"] = title
            changes.setdefault(event["id"], {})["name"] = title
        if lead not in event["desc"]:
            event["desc"] = lead
            changes.setdefault(event["id"], {})["desc"] = lead
        for number, ((label, response), choice) in enumerate(zip(pairs, event["choices"])):
            label = label.rstrip(".")
            if choice["label"] != label:
                choice["label"] = label
                changes.setdefault(event["id"], {})[f"choice_{number}"] = label
            if response.startswith("["):
                placeholders.append({"id": event["id"], "choice": number,
                                     "action": "Preserved current outcome; editorial placeholder was not imported."})
            elif choice["effects"].get("log") != response:
                choice["effects"]["log"] = response
                changes.setdefault(event["id"], {})[f"log_{number}"] = response
    assert mechanics(events) == before, "Editorial sync must not change gameplay."
    locales = {}
    for path in LOCALES.glob("*.json"):
        locale = read(path)
        for event in originals:
            for key, value in changes.get(event["id"], {}).items():
                previous = event[key] if key in ("name", "desc") else (
                    event["choices"][int(key.split("_")[1])]["label"] if key.startswith("choice_")
                    else event["choices"][int(key.split("_")[1])]["effects"].get("log", ""))
                if path.stem == "en" or locale["encounter_copy"][event["id"]].get(key, previous) == previous:
                    locale["encounter_copy"][event["id"]][key] = value
        for event_id in covered:
            locale["encounter_copy"][event_id].setdefault("name", by_id[event_id]["name"])
        locales[path] = locale
    if args.translations:
        for language, translated_events in read(args.translations).items():
            path = LOCALES / f"{language}.json"
            if language == "en" or path not in locales:
                raise ValueError(f"Invalid translation language: {language}")
            for event_id, fields in translated_events.items():
                expected = set(locales[LOCALES / "en.json"]["encounter_copy"].get(event_id, {}))
                if event_id not in covered or set(fields) != expected or not all(fields.values()):
                    raise ValueError(f"Incomplete translation: {language}/{event_id}")
                locales[path]["encounter_copy"][event_id] = fields
    towns = read(DATA / "town-facts.json")
    town_facts_before = [{key: value for key, value in town.items() if key != "comment"} for town in towns]
    town_copy = [p["text"].strip() for p in paragraphs if p.get("tabId") == "t.e4vbtrh2qo76"]
    covered_towns = []
    for town in towns:
        if town["town"] in town_copy:
            start = town_copy.index(town["town"])
            line = town_copy[town_copy.index("Resident’s line", start) + 1]
            if not args.narrative_copy and town["comment"]["en"] != line:
                raise ValueError(f"Town translation review required: {town['town']}")
            covered_towns.append(town["town"])
    english = read(LOCALES / "en.json")
    crew_lines = [p["text"].strip() for p in paragraphs if p.get("tabId") == "t.7s63fsgx99si" and p["text"].startswith("{name}")]
    current_crew = [english["trail"][f"care_reason_{i}"] for i in range(8)] + [english["ally_loss"][f"reason_{i}"] for i in range(6)]
    narrative_changes = {}
    if args.narrative_copy:
        narrative_changes = apply_narrative(source, args, towns, locales, covered_towns)
        narrative = read(args.narrative_copy)["locale_copy"]
        crew_lines = [value for key, value in narrative.items()
                      if key.startswith(("trail.care_reason_", "ally_loss.reason_"))]
    else:
        assert crew_lines == current_crew, "Crew translation review required."
    town_facts_after = [{key: value for key, value in town.items() if key != "comment"} for town in towns]
    assert town_facts_before == town_facts_after, "Editorial sync must preserve official town facts and sources."
    report = {"source": source["document_url"], "title": source["title"], "revision": source["revisionId"],
              "tabs": list(dict.fromkeys(p["tabId"] for p in paragraphs)),
              "encounters_reviewed": covered, "changed_copy": changes, "town_comments_reviewed": covered_towns,
              "crew_and_ally_lines_reviewed": len(crew_lines), "placeholders": placeholders,
              "unmentioned_encounters_preserved": [e["id"] for e in events if e["id"] not in covered],
              "policy": "Import the complete approved lead-in and mapped choices. Retain existing policy context when the document's lead-in is already present. Preserve every gameplay cost, reward, region and eligibility rule; keep existing outcomes for unfinished editorial placeholders."}
    report["mechanics_unchanged"] = mechanics(events) == before
    report["renamed_encounters"] = {key: value["name"] for key, value in changes.items() if "name" in value}
    report["lead_in_policy"] = "Preserve every lead-in paragraph, separated by a blank line."
    report["translations"] = sorted(read(args.translations)) if args.translations else []
    report["narrative_changes"] = narrative_changes
    report["narrative_translations"] = sorted(read(args.narrative_translations)) if args.narrative_translations else []
    report["town_facts_unchanged"] = town_facts_before == town_facts_after
    report["dry_run"] = args.dry_run
    if not args.dry_run:
        write(DATA / "game.json", events)
        write(DATA / "town-facts.json", towns)
        for path, locale in locales.items():
            write(path, locale)
    write(args.report, report)
    print(f"Reviewed {len(covered)} encounters, {len(covered_towns)} town comments, and {len(crew_lines)} crew/ally lines; updated {len(changes)} encounters; left {len(placeholders)} placeholders out.")


if __name__ == "__main__":
    main()
