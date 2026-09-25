#!/usr/bin/env python3
"""Keep English contextual help and the rendered guide on one source."""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOPICS = ROOT / "help" / "topics.json"
ENGLISH = ROOT / "dystrail-web" / "i18n" / "en.json"
GUIDE = ROOT / "docs" / "src"
TOKEN = re.compile(r"\{\{#help ([a-z][a-z0-9_-]*)\}\}")


def load():
    topics = json.loads(TOPICS.read_text())
    english = json.loads(ENGLISH.read_text())
    if not isinstance(topics, dict) or not topics:
        raise ValueError("help/topics.json must contain topics")
    keys = set()
    for name, topic in topics.items():
        if not re.fullmatch(r"[a-z][a-z0-9_-]*", name):
            raise ValueError(f"invalid topic name: {name}")
        if set(topic) != {"i18n_key", "text"}:
            raise ValueError(f"{name}: expected i18n_key and text")
        if not isinstance(topic["text"], str) or not topic["text"].strip():
            raise ValueError(f"{name}: text must be nonempty")
        key = topic["i18n_key"]
        if key in keys:
            raise ValueError(f"duplicate i18n key: {key}")
        keys.add(key)
        node = english
        for part in key.split("."):
            if not isinstance(node, dict) or part not in node:
                raise ValueError(f"{name}: missing game key {key}")
            node = node[part]
        if not isinstance(node, str):
            raise ValueError(f"{name}: game key {key} is not text")
    return topics, english


def check(topics, english):
    errors = []
    for name, topic in topics.items():
        node = english
        for part in topic["i18n_key"].split("."):
            node = node[part]
        if node != topic["text"]:
            errors.append(f"{name}: game translation differs from help/topics.json")
    used = set()
    for path in GUIDE.rglob("*.md"):
        for name in TOKEN.findall(path.read_text()):
            used.add(name)
            if name not in topics:
                errors.append(f"{path.relative_to(ROOT)}: unknown topic {name}")
    for name in topics.keys() - used:
        errors.append(f"{name}: not used by the Game Guide")
    if errors:
        raise ValueError("\n".join(errors))


def sync(topics, english):
    for topic in topics.values():
        node = english
        parts = topic["i18n_key"].split(".")
        for part in parts[:-1]:
            node = node[part]
        node[parts[-1]] = topic["text"]
    ENGLISH.write_text(json.dumps(english, ensure_ascii=False, indent=2) + "\n")


def preprocess(topics, english):
    check(topics, english)
    _, book = json.load(sys.stdin)

    def expand(item):
        if "Chapter" not in item:
            return
        chapter = item["Chapter"]
        chapter["content"] = TOKEN.sub(
            lambda match: topics[match.group(1)]["text"], chapter["content"]
        )
        for child in chapter["sub_items"]:
            expand(child)

    sections = book.get("sections", book.get("items"))
    if sections is None:
        raise ValueError(f"unknown mdBook structure: {list(book)}")
    for section in sections:
        expand(section)
    json.dump(book, sys.stdout, ensure_ascii=False)


def main():
    if "supports" in sys.argv:
        return
    command = sys.argv[1] if len(sys.argv) > 1 else "check"
    topics, english = load()
    if command == "sync":
        sync(topics, english)
        check(topics, english)
    elif command == "check":
        check(topics, english)
    elif command == "preprocess":
        preprocess(topics, english)
    else:
        raise ValueError(f"unknown command: {command}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
