from pathlib import Path
import json,hashlib,collections
b=Path('/tmp/dystrail-retune-2026-09-13');evidence={'parity_verified_before_interpretation':True,'cases':{},'scope':'Exactly Classic Resource Manager 1351 and Deep Resource Manager 1391, before/after Activity settlement. No fresh policy or production edits.'}
fields=['stage','day','clock_minutes','clock','pace','diet','stats','cash_cents','driving_minutes_total','pace_fatigue_remainder','miles_traveled_actual','distance_debug','day_state','rest_threshold','auto_camp_rest','should_auto_rest','camp','illness_days_remaining','illness_travel_penalty','disease_cooldown','weather','ledger','last_completed_day','last_day_reason','last_crossing','last_log','encounter_id','crew_care']
action_ends={'town_work':'after','town_conversation':'after_clock','gather':'after','camp_forage':'after','camp_rest':'after_clock','crew_care':'after_clock','repair':'after_clock','encounter':'after_clock','hearing':'after_clock','travel_step':'after'}
stationary={'town_work','town_conversation','gather','camp_forage','camp_rest','crew_care','repair','hearing'}
def slim(r):return {k:r[k] for k in fields if k in r}
for phase in ['before','after']:
    parity=json.loads((b/('trace-rm-'+phase+'-equivalence.json')).read_text());assert all(v['all_48_fields_exact'] and v['decisions_match_order_day_name_label_policy'] and v['choice_ids_and_indices_match_unchanged_game_data'] for v in parity.values())
    records=[json.loads(s) for s in (b/('trace-rm-'+phase+'.jsonl')).read_text().splitlines()]
    for mode,seed in [('Classic',1351),('Deep',1391)]:
        key=f'{mode}-{seed}';rs=[r for r in records if r['mode']==mode and r['seed']==seed];states=[r for r in rs if r['trace_type']=='TRACE_STATE'];summary=next(r for r in rs if r['trace_type']=='TRACE_SUMMARY');final=summary['day_records']+([summary['open_ledger']['current_day_record']] if summary['open_ledger']['current_day_record'] else []);end={r['day_index']:r for r in final}
        maxima={};drops=[];previous=None
        for r in states:
            current=r['ledger']['current_day_record']
            for d in [current,r['last_completed_day']]:
                if d:maxima[d['day_index']]=max(maxima.get(d['day_index'],0),d['miles'])
            if previous:
                old=previous['ledger']['current_day_record']
                if old and current and old['day_index']==current['day_index'] and current['miles']+0.001<old['miles']:drops.append({'before':slim(previous),'after':slim(r)})
            previous=r
        erased=[{'day':d+1,'maximum_seen':m,'final':end.get(d)} for d,m in maxima.items() if d not in end or end[d]['miles']+0.001<m]
        cumulative_drops=[{'before':slim(x),'after':slim(y)} for x,y in zip(states,states[1:]) if y['miles_traveled_actual']<x['miles_traveled_actual'] or y['driving_minutes_total']<x['driving_minutes_total']]
        pending={};actions=[]
        for r in states:
            prefix,_,stage=r['stage'].partition('.')
            if prefix not in action_ends:continue
            if stage=='before':pending[prefix]=r
            elif stage==action_ends[prefix] and prefix in pending:
                old=pending.pop(prefix);actions.append({'action':prefix,'before':slim(old),'after':slim(r),'active_minutes':(r['day']-old['day'])*300+r['clock_minutes']-old['clock_minutes'],'driving_minutes':r['driving_minutes_total']-old['driving_minutes_total'],'miles':r['miles_traveled_actual']-old['miles_traveled_actual'],'fractional_fatigue_changed':r['pace_fatigue_remainder']!=old['pace_fatigue_remainder']})
        bad_actions=[x for x in actions if x['action'] in stationary and (x['driving_minutes']!=0 or x['miles']!=0 or x['fractional_fatigue_changed'])]
        stationary_days=[r['day_index']+1 for r in final if r['kind']=='non_travel'];important={14,20,24} if mode=='Classic' else {6,16,24,25}
        entries={'parity':parity[key],'summary':summary,'final_day_records':final,'stationary_days':stationary_days,'ledger_conservation':{'total_ledger_miles':sum(r['miles'] for r in final),'total_campaign_miles':summary['miles_traveled'],'difference':sum(r['miles'] for r in final)-summary['miles_traveled'],'same_day_record_decreases':drops,'observed_earned_miles_missing_from_final_ledger':erased,'cumulative_miles_or_driving_counter_decreases':cumulative_drops},'stationary_action_conservation':{'action_count':sum(x['action'] in stationary for x in actions),'violations':bad_actions},'actions':actions,'relevant_day_boundaries':[slim(r) for r in states if r['day'] in important],'actual_pace_diet':dict(collections.Counter(r['pace']+'/'+r['diet'] for r in states if r['stage']=='travel_step.before'))}
        evidence['cases'].setdefault(key,{})[phase]=entries
        print(phase,key,'stationary',stationary_days,'ledgerdelta',entries['ledger_conservation']['difference'],'record_drops',len(drops),'missing',len(erased),'stationary_actions',entries['stationary_action_conservation']['action_count'],'violations',len(bad_actions),'pace/diet',entries['actual_pace_diet'])
        assert not drops and not erased and not cumulative_drops and not bad_actions
p=b/'trace-rm-activity-findings.json';p.write_text(json.dumps(evidence,indent=2,ensure_ascii=False)+'\n');print('findings',p,hashlib.sha256(p.read_bytes()).hexdigest())
