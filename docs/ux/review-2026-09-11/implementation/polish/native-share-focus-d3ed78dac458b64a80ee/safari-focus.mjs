// Focus regression in real Safari; sharing responses are controlled, not an OS sheet assertion.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import assert from 'node:assert/strict';
const out='/tmp/dystrail-native-share-focus/safari-focus-run2';mkdirSync(out,{recursive:true});
const fixture=JSON.parse(readFileSync('/tmp/dystrail-native-device-check/fixture.json','utf8'));
const report={browser:'Native Safari',revision:'d3ed78dac458b64a80ee',basis:'Real Safari keyboard and focus behavior with controlled share promises; actual OS handoff is recorded separately.',checks:[]};
let session;
async function call(path,body,method=body===undefined?'GET':'POST'){
 const response=await fetch('http://127.0.0.1:9515'+path,{method,signal:AbortSignal.timeout(30000),headers:{'Content-Type':'application/json'},...(body===undefined?{}:{body:JSON.stringify(body)})});
 const data=await response.json();if(!response.ok||data.value?.error)throw Error(JSON.stringify(data.value));return data.value;
}
const cmd=(path,body,method)=>call('/session/'+session+path,body,method);
const exec=(script,args=[])=>cmd('/execute/sync',{script,args});
async function until(script,timeout=30000){const start=Date.now();while(Date.now()-start<timeout){if(await exec(script))return;await new Promise(r=>setTimeout(r,150));}throw Error('Timed out: '+script);}
async function click(selector){const e=await cmd('/element',{using:'css selector',value:selector});await exec('arguments[0].scrollIntoView({block:"center",behavior:"instant"});',[e]);await until('const e=document.querySelector('+JSON.stringify(selector)+'),r=e.getBoundingClientRect();return r.width>0&&r.height>0&&document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest("button")===e');await cmd('/element/'+e['element-6066-11e4-a52e-4f735466cecf']+'/click',{});}
async function key(value,shift=false){const actions=[];if(shift)actions.push({type:'keyDown',value:'\uE008'});actions.push({type:'keyDown',value},{type:'keyUp',value});if(shift)actions.push({type:'keyUp',value:'\uE008'});await cmd('/actions',{actions:[{type:'key',id:'focus-check',actions}]});}
try{
 const created=await call('/session',{capabilities:{alwaysMatch:{browserName:'safari'}}});session=created.sessionId;report.capabilities={browserName:created.capabilities.browserName,browserVersion:created.capabilities.browserVersion};
 await cmd('/timeouts',{script:30000,pageLoad:30000,implicit:0});await cmd('/window/rect',{width:1280,height:980});
 for(const mode of ['Classic','Deep']){
  await cmd('/url',{url:'http://127.0.0.1:62101/prepare'});
  const saved=structuredClone(fixture);saved.state.mode=mode;saved.pending.mode=mode;
  await exec('localStorage.setItem("dystrail.autosave.v1",JSON.stringify(arguments[0]));',[saved]);
  await cmd('/url',{url:'http://127.0.0.1:62101/play/'});
  await until('return window.dystrailOffline?.revision==="d3ed78dac458b64a80ee"&&window.dystrailOffline.state==="ready"&&!!document.querySelector("#result-share-open")',120000);
  const before=await exec('return localStorage.getItem("dystrail.autosave.v1")');
  await exec(`const state=window.sharingProbe={mode:'cancel',calls:[]};
   Object.defineProperty(navigator,'canShare',{configurable:true,value:data=>!!data.files?.length});
   Object.defineProperty(navigator,'share',{configurable:true,value:data=>{
    state.calls.push({text:data.text,type:data.files[0].type,size:data.files[0].size});
    return new Promise((resolve,reject)=>{state.finish=()=>{document.activeElement?.blur();if(state.mode==='cancel')reject(new DOMException('Cancelled','AbortError'));else if(state.mode==='fail')reject(new DOMException('Unavailable','NotAllowedError'));else resolve();};});
   }});`);
  await click('#result-share-open');await until('return document.querySelector(".share-preview img")?.naturalWidth===1200&&!!document.querySelector(".share-actions .retro-btn-primary")');
  const draft='Native Safari focus check — preserve this draft.';
  await exec('const e=document.querySelector("#share-post");e.value=arguments[0];e.dispatchEvent(new Event("input",{bubbles:true}));',[draft]);
  for(const [result,message] of [['cancel','Sharing cancelled.'],['fail','Sharing is unavailable.'],['success','Passed to your device']]){
   await exec('window.sharingProbe.mode=arguments[0];',[result]);
   await click('.share-actions .retro-btn-primary');await until('return document.querySelector(".share-actions .retro-btn-primary").disabled');
   await exec('window.sharingProbe.finish();');
   await until('const b=document.querySelector(".share-actions .retro-btn-primary");return !b.disabled&&document.activeElement===b');
   assert((await exec('return document.querySelector(".share-status").textContent')).includes(message));
   assert.equal(await exec('return document.querySelector("#share-post").value'),draft);
   await key('\uE004');await until('return document.activeElement===document.querySelector(".share-close")');
   await key('\uE004',true);await until('return document.activeElement===document.querySelector(".share-actions .retro-btn-primary")');
   report.checks.push(mode+' '+result+': status, preserved draft, enabled button, restored focus and forward/reverse focus trap');
  }
  const calls=await exec('return window.sharingProbe.calls');assert.equal(calls.length,3);assert(calls.every(c=>c.text===draft&&c.type==='image/png'&&c.size>10000));
  await key('\uE00C');await until('return !document.querySelector(".share-composer")&&document.activeElement.id==="result-share-open"');
  assert.equal(await exec('return localStorage.getItem("dystrail.autosave.v1")'),before);
  report.checks.push(mode+': Escape closes dialog, focus returns to opener and complete saved game is unchanged');
 }
 report.passed=true;console.log(JSON.stringify(report,null,2));
}catch(error){report.failure=String(error.stack||error);report.passed=false;console.error(report.failure);process.exitCode=1;}
finally{if(session)await cmd('',undefined,'DELETE').catch(()=>{});writeFileSync(out+'/verification.json',JSON.stringify(report,null,2)+'\n');}
