from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse,unquote
import json
root=Path('/tmp/dystrail-share-hud-review/verified-web').resolve()
out=Path('/tmp/dystrail-native-device-check')
saved=json.loads(Path('/tmp/dystrail-share-hud-review/update-check/before.json').read_text())
for key in ['state','pending']:
 saved[key]['abandoned']=True
 for crew in saved[key]['party']['members']:
  if crew['persona']==saved[key]['persona_id']:crew['name']='Native Share Review'
saved['phase']='Result'
(out/'fixture.json').write_text(json.dumps(saved,indent=2))
script=json.dumps(saved).replace('<','\\u003c')
page=('''<!doctype html><html lang="en"><meta charset="utf-8"><title>Isolated Dystrail share check</title><style>body{font:20px system-ui;background:#102230;color:#f6edcf;margin:4rem;max-width:40rem}button{font:inherit;padding:1rem;background:#f4d570;border-radius:.5rem}p{line-height:1.5}</style><h1>Isolated share check</h1><p>This local test uses a disposable finished journey and the exact published game. It does not change the journey on port 8180 or dystrail.com.</p><button id="prepare">Open test ending</button><script>document.querySelector('#prepare').onclick=()=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(FIXTURE));location.href='/play/';};</script></html>''').replace('FIXTURE',script).encode()
class Handler(SimpleHTTPRequestHandler):
 def __init__(self,*args,**kwargs):super().__init__(*args,directory=str(root),**kwargs)
 def do_GET(self):
  if urlparse(self.path).path=='/prepare':
   self.send_response(200);self.send_header('Content-Type','text/html; charset=utf-8');self.send_header('Cache-Control','no-store');self.end_headers();self.wfile.write(page);return
  super().do_GET()
 def translate_path(self,path):
  path=unquote(urlparse(path).path).removeprefix('/play/').lstrip('/')
  f=(root/path).resolve()
  return str(f if f.is_relative_to(root) and f.is_file() else root/'index.html')
 def log_message(self,*args):pass
server=ThreadingHTTPServer(('127.0.0.1',62101),Handler)
(out/'origin.txt').write_text(f'http://127.0.0.1:{server.server_port}')
print((out/'origin.txt').read_text(),flush=True);server.serve_forever()
