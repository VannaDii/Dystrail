from pathlib import Path
import os,sys,subprocess,json,hashlib,shutil,time
base=Path('/tmp/dystrail-retune-2026-09-13')
name=sys.argv[1]; source=base/(name+'-source'); manifest=base/(name+'-inputs.json')
record=json.loads(manifest.read_text())
env=dict(os.environ,CARGO_TARGET_DIR='/Users/vanna/Source/Dystrail/target')
def verify():
    changed=[f for f,h in record['inputs'].items() if hashlib.sha256((source/f).read_bytes()).hexdigest()!=h]
    assert not changed,changed

def run(label,args):
    start=time.monotonic()
    with (base/(name+'-'+label+'.log')).open('w') as out:
        result=subprocess.run(args,cwd=source,env=env,stdout=out,stderr=subprocess.STDOUT)
    record.setdefault('commands',{})[label]={'argv':args,'exit_code':result.returncode,'seconds':round(time.monotonic()-start,3)}
    manifest.write_text(json.dumps(record,indent=2)+'\n')
    print(label,result.returncode,flush=True)
    return result.returncode
verify()
for file in source.rglob('*.rs'): file.touch()
if name=='pace':
    result=run('regression',['cargo','test','-p','dystrail-game','pace_fatigue'])
    if result: sys.exit(result)
result=run('seed4242',['cargo','test','-p','dystrail-tester','balanced_run_survives_past_day_45'])
if result: sys.exit(result)
for file in source.rglob('*.rs'): file.touch()
result=run('build',['cargo','build','--release','-p','dystrail-tester'])
if result: sys.exit(result)
verify(); binary=base/(name+'-tester'); shutil.copy2('/Users/vanna/Source/Dystrail/target/release/dystrail-tester',binary)
binary_bytes=binary.read_bytes()
assert str((source/'dystrail-tester').resolve()).encode() in binary_bytes,'Release binary does not embed the intended tester source path'
record['binary_sha256']=hashlib.sha256(binary_bytes).hexdigest()
record['compiled_source_path_verified']=str((source/'dystrail-tester').resolve())
result=run('campaign',[str(binary),'--mode','logic','--scenarios','real-game','--seeds','1337','--iterations','250','--acceptance','--report','csv','--output',str(base/(name+'-results.csv'))])
verify(); record['source_unchanged_after_run']=True
with (base/(name+'-analysis.json')).open('w') as out:
    subprocess.run(['python3','/tmp/dystrail-hourly-analysis.py',str(base/(name+'-results.csv'))],stdout=out,check=True)
record['outputs']={suffix:hashlib.sha256((base/(name+suffix)).read_bytes()).hexdigest() for suffix in ['-results.csv','-analysis.json','-campaign.log']}
manifest.write_text(json.dumps(record,indent=2)+'\n')
sys.exit(result)
