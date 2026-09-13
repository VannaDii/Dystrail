import {readFileSync,writeFileSync} from 'node:fs';
import vm from 'node:vm';
const report=[];
for(const file of ['safari-feedback.mjs','safari-smoke.mjs']){
 const source=readFileSync('/tmp/dystrail-native-safari-balance/'+file,'utf8');
 let checked=0;const failures=[];
 const pattern=/\b(?:exec|asyncExec|until)\(\s*('(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*")\s*(?=[,)])/g;
 for(const m of source.matchAll(pattern)){
  try{const value=vm.runInNewContext(m[1]);vm.compileFunction(value,[]);checked++;}
  catch(error){failures.push({line:source.slice(0,m.index).split('\n').length,message:String(error),script:m[1]});}
 }
 report.push({file,checked,failures});
}
writeFileSync('/tmp/dystrail-native-safari-balance/static-script-validation.json',JSON.stringify(report,null,2));
console.log(JSON.stringify(report,null,2));
if(report.some(r=>r.failures.length))process.exitCode=1;
