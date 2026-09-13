"""Evidence earned by the documentation choices in the shipped encounter bank.

The record IDs are save data; player-facing labels come from the shared Receipts
stat. Reapply this mapping whenever a regional content generator replaces rows.
"""
import json
from pathlib import Path

REWARDS = {
    'beltway_briefing': 0,
    'classic_bridge_crews': 1,
    'classic_mail_drop': 0,
    'classic_mutual_aid_dispatch': 2,
    'classic_press_briefing': 2,
    'deep_field_intel': 0,
    'deep_grassroots_signal': 1,
    'deep_memorandum_dump': 0,
    'deep_secure_line': 0,
    'deep_watchdog_sync': 0,
    'sat_corn_bullets': 0,
    'sat_cow_citations': 0,
    'sat_alternator_tariff': 1,
    'sat_billion_pothole': 0,
    'sat_bridge_bullets': 0,
    'sat_factcheck_shift': 0,
    'sat_cabinet_guest': 0,
    'sat_straw_inspection': 1,
    'sat_receipt_museum': 0,
    'sat_weather_desk': 0,
    'sat_bibliography_emergency': 0,
    'west_grant_translation': 1,
    'west_rail_replacement': 1,
    'west_laboratory_overhead': 0,
    'west_wind_loyalty': 1,
    'west_beef_passports': 1,
}


def apply_evidence(events):
    for event in events:
        if event['id'] in REWARDS:
            index = REWARDS[event['id']]
            event['choices'][index]['effects']['add_receipt'] = event['id']


if __name__ == '__main__':
    path = Path('dystrail-web/static/assets/data/game.json')
    events = json.loads(path.read_text())
    assert set(REWARDS) <= {event['id'] for event in events}
    apply_evidence(events)
    path.write_text(json.dumps(events, ensure_ascii=False, indent=2) + '\n')
    print(f'{len(REWARDS)} evidence-gathering choices')
