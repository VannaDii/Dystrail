from pathlib import Path
import argparse,csv,hashlib,io,json,os,re,shutil,subprocess,time
b=Path('/tmp/dystrail-retune-2026-09-13')
args=argparse.ArgumentParser();args.add_argument('phase',choices=['before','after']);args.add_argument('--run-diagnostic',action='store_true',help='Use only after root has released Cargo and approved the frozen diff.');a=args.parse_args();name='trace-rm-'+a.phase;freeze=json.loads((b/(name+'-freeze.json')).read_text());source=Path(freeze['source']);binary=Path(freeze['base_binary']);env=dict(os.environ);env.pop('DYSTRAIL_DEBUG_LOGS',None);env['CARGO_TARGET_DIR']='/Users/vanna/Source/Dystrail/target'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def verify():
    bad=[p for p,h in freeze['inputs'].items() if sha(source/p)!=h];assert not bad,bad
    assert sha(binary)==freeze['base_binary_sha256'];assert str((Path(freeze['base'])/'dystrail-tester').resolve()).encode() in binary.read_bytes()
def loadcsv(p):
    with p.open() as f:return {(r['scenario'],int(r['seed_value'])):r for r in csv.DictReader(f) if r.get('mode') in ('Classic','Deep')}
def run(label,argv):
    out=b/(name+'-'+label+'.log');start=time.monotonic()
    with out.open('w') as f:r=subprocess.run(argv,cwd=source,env=env,stdout=f,stderr=subprocess.STDOUT)
    d={'argv':argv,'cwd':str(source),'exit_code':r.returncode,'seconds':round(time.monotonic()-start,3),'log_path':str(out),'log_sha256':sha(out)};print(label,r.returncode,flush=True);return d
verify();baseline=loadcsv(Path(freeze['base_campaign_csv']));result={'phase':a.phase,'frozen_patch_sha256':freeze['patch_sha256'],'commands':{}}
refcsv=b/(name+'-reference.csv');refargv=[str(binary),'--mode','logic','--scenarios','real-game','--seeds','1351,1391','--iterations','1','--verbose','--report','csv','--output',str(refcsv)]
result['commands']['reference']=run('reference',refargv);verify();reference=loadcsv(refcsv);assert len(reference)==16,len(reference)
refdif={str(k):{f:{'original':baseline[k][f],'replayed':v} for f,v in r.items() if baseline[k][f]!=v} for k,r in reference.items() if baseline[k]!=r};result['reference_campaign_parity']={'rows':len(reference),'fields_per_row':48,'exact':not refdif,'differences':refdif,'csv_sha256':sha(refcsv)}
runfile=b/(name+'-run.json');runfile.write_text(json.dumps(result,indent=2)+'\n');assert not refdif,refdif
if not a.run_diagnostic:
    print('Reference parity passed. Diagnostic has not been built or run.',flush=True);raise SystemExit(0)
verify()
for p in source.rglob('*.rs'):p.touch()
argv=['cargo','test','-p','dystrail-tester','trace_resource_manager_activity_settlement','--','--nocapture']
result['commands']['diagnostic']=run('diagnostic',argv);verify();result['source_hashes_unchanged']=True;runfile.write_text(json.dumps(result,indent=2)+'\n')
if result['commands']['diagnostic']['exit_code']:
    raise SystemExit(result['commands']['diagnostic']['exit_code'])
raw=(b/(name+'-diagnostic.log')).read_text();binary_path=Path(re.search(r'Running unittests [^\n]+ \(([^)]+)\)',raw).group(1));testbinary=b/(name+'-test-binary');shutil.copy2(binary_path,testbinary);compiled=str((source/'dystrail-tester').resolve());assert compiled.encode() in testbinary.read_bytes();result['binary_path']=str(testbinary);result['binary_sha256']=sha(testbinary);result['compiled_source_path_verified']=compiled;runfile.write_text(json.dumps(result,indent=2)+'\n');print('Diagnostic binary copied and source verified; Cargo can be released.',flush=True)
records=[]
for line in raw.splitlines():
    marker=re.search(r'\b(TRACE_STATE|TRACE_ACTIVITY|TRACE_TRAVEL|TRACE_SUMMARY|TRACE_DECISION|TRACE_CSV) (\{.*\})$',line)
    if marker:records.append(dict(trace_type=marker[1],**json.loads(marker[2])))
jsonl=b/(name+'.jsonl');jsonl.write_text(''.join(json.dumps(r,ensure_ascii=False,separators=(',',':'))+'\n' for r in records))
# Keep verbose reference decision sequences separate from instrumented observations.
verbose={};active=None
for line in (b/(name+'-reference.log')).read_text().splitlines():
    line=re.sub(r'\x1b\[[0-9;]*m','',line)
    match=re.search(r'Starting simulation \| seed:(\d+) mode:(Classic|Deep) policy:(.+)$',line)
    if match:
        active=(match[2],int(match[1]),match[3]);verbose.setdefault(active,[]).append([])
    elif active and line.startswith('🎯 Day '):
        match=re.match(r'🎯 Day (\d+): (.*?) -> (.*) \(([^()]*)\)$',line);assert match,line
        verbose[active][-1].append({'day':int(match[1]),'encounter_name':match[2],'choice_label':match[3],'policy':match[4]})
encounters={e['id']:e for e in json.loads((source/'dystrail-web/static/assets/data/game.json').read_text())};parity={}
for mode,seed in [('Classic',1351),('Deep',1391)]:
    key=(mode+' - Resource Manager',seed);actual=[r for r in records if r['trace_type']=='TRACE_CSV' and r['mode']==mode and r['seed']==seed];assert len(actual)==1
    decoded=list(csv.DictReader(io.StringIO(actual[0]['csv'])));assert len(decoded)==1;row=decoded[0];fieldcheck={f:row[f]==baseline[key][f]==reference[key][f] for f in row};assert len(fieldcheck)==48
    decs=[r for r in records if r['trace_type']=='TRACE_DECISION' and r['mode']==mode and r['seed']==seed];observed=[{f:r[f] for f in ['day','encounter_name','choice_label','policy']} for r in decs];expected=verbose[(mode,seed,'Resource Manager')];assert len(expected)==1
    indices=[encounters[r['encounter_id']]['name']==r['encounter_name'] and encounters[r['encounter_id']]['choices'][r['choice_index']]['label']==r['choice_label'] for r in decs]
    parity[f'{mode}-{seed}']={'all_48_fields_exact':all(fieldcheck.values()),'fields':fieldcheck,'metric_differences':{f:{'campaign':baseline[key][f],'reference':reference[key][f],'trace':row[f]} for f,ok in fieldcheck.items() if not ok},'decisions_count':len(decs),'decisions_match_order_day_name_label_policy':observed==expected[0],'choice_ids_and_indices_match_unchanged_game_data':all(indices),'decision_sequence':observed,'expected_decision_sequence':expected[0]}
pf=b/(name+'-equivalence.json');pf.write_text(json.dumps(parity,indent=2,ensure_ascii=False)+'\n');result['parity_file']=str(pf);result['parity_sha256']=sha(pf);result['trace_jsonl_sha256']=sha(jsonl);result['source_hashes_unchanged_after_analysis']=True;verify();runfile.write_text(json.dumps(result,indent=2)+'\n')
assert all(p['all_48_fields_exact'] and p['decisions_match_order_day_name_label_policy'] and p['choice_ids_and_indices_match_unchanged_game_data'] for p in parity.values()),parity
print(json.dumps({'phase':a.phase,'parity':'all 48 fields and every decision match','cases':{k:v['decisions_count'] for k,v in parity.items()},'snapshots':len(records),'run_manifest':str(runfile)},indent=2),flush=True)
