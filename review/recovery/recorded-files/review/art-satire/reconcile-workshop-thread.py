"""Pin the user-designated workshop handoff without changing runtime mechanics."""
import ast, hashlib, json, re, shutil
from pathlib import Path
root=Path(__file__).resolve().parents[2]
snapshot=root/'review/art-satire/source-refresh-20260914'
source=Path('/Users/vanna/Source/Dystrail/review/satire-fresh-2026-09-13')
parts=json.loads((snapshot/'manual-integration-parts.json').read_text())
messages=json.loads((snapshot/'workshop-thread-recent.json').read_text())
final=next(m for m in reversed(messages) if m['phase']=='final_answer')
body=final['text'].split(':::writing',1)[1].split('\n',1)[1].rsplit('\n:::',1)[0]
normalize=lambda text:'\n'.join(line for line in text.splitlines() if line.strip())
assert normalize(body)==normalize(parts[0]['text'])
shutil.copy2(source/'manual-integration.md',snapshot/'manual-integration.md')
shutil.copy2(source/'assemble.py',snapshot/'source-assemble.py')
full=(snapshot/'manual-integration.md').read_text()
assert all(part['text'] in full for part in parts)
# Load only the source's pure formatting functions, never its write-on-import code.
tree=ast.parse((snapshot/'source-assemble.py').read_text())
functions=ast.Module(body=[node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name in ['line','render']],type_ignores=[])
namespace={'parts':{'functional':json.loads((snapshot/'functional.json').read_text())}}
exec(compile(functions,'source-render','exec'),namespace)
units={unit['id']:unit for unit in json.loads((snapshot/'complete-pack.json').read_text())['units']}
ids=[id for part in parts for id in part['ids']]
assert len(ids)==len(set(ids))==172
rows=0
for part in parts:
    chunks=re.split(r'(?m)^### ',part['text'])
    for id in part['ids']:
        matches=[chunk for chunk in chunks if chunk.startswith(id+' · ')]
        assert len(matches)==1,id
        plain=matches[0].replace('**','')
        for row in namespace['render'](units[id]):
            assert row['text'] in plain,(id,row)
            if row.get('link'):assert row['link'] in plain,(id,row)
            rows+=1
record={
 'thread_id':'01a0932f-0202-7150-a55e-455d057576ca',
 'title':'Satire Workshop',
 'authority':'User explicitly designated this conversation as the source for satire missing from the document.',
 'final_timestamp':final['timestamp'],
 'final_message_line':final['line'],
 'thread_copy':'57 road units in the final reply, matching Part 1 except blank-line placement.',
 'linked_handoff':'Complete 172-unit manual-integration.md and its three structured parts.',
 'parts':[{'part':part['part'],'title':part['title'],'units':len(part['ids']),'ids':part['ids']} for part in parts],
 'validation':{'unique_units':len(ids),'editorial_rows_and_source_links_checked':rows,'missing':0,'complete_pack_contains_all':True,'runtime_changes':False},
 'result':'All omitted-document handoff entries are already present verbatim in the pinned latest structured pack; no duplicate or replacement records needed. This is source reconciliation, not a claim that all units are implemented.',
 'document_status':'Partially synchronized; no document write attempted.',
 'hashes':{name:hashlib.sha256((snapshot/name).read_bytes()).hexdigest() for name in ['workshop-thread-recent.json','workshop-thread-final.md','manual-integration.md','manual-integration-parts.json','complete-pack.json','source-assemble.py']}
}
(snapshot/'thread-reconciliation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
p=snapshot/'reconciliation.json';current=json.loads(p.read_text());current['thread_authority']='thread-reconciliation.json';p.write_text(json.dumps(current,ensure_ascii=False,indent=2)+'\n')
print(f'Confirmed {len(ids)} handoff units and {rows} editorial rows against the pinned source; no missing entries.')
