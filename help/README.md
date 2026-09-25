# Shared help copy

`topics.json` is the English source for the contextual help passages used in both
the game and the Game Guide. Each topic names its existing localization key.
The guide inserts a topic with `{{#help topic_name}}` when mdBook builds; the
published page contains the full text, so readers do not need to open the game.

After changing a topic, run `python3 help/build.py sync` and commit the updated
`dystrail-web/i18n/en.json`. `python3 help/build.py check` verifies the game
copy, guide references, and topic coverage. The guide build also checks these
before it renders. Other language files retain their translations and need a
translation review when the English meaning changes.
