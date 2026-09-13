import {readFileSync} from 'node:fs';import {join} from 'node:path';
export const routes=JSON.parse(readFileSync(join(__dirname,'../../dystrail-game/data/routes.json'),'utf8'));
export function atTown(state:any,name:string,offset=1){
 const route=routes.find((r:any)=>r.id===state.persona_id);const town=route.stops.find((s:any)=>s.name===name);
 state.miles_traveled_actual=(town.mile+offset)/route.total_miles*state.trail_distance;state.miles_traveled=Math.round(state.miles_traveled_actual);
 state.route_services={route_id:route.id,stop:town.mile,traded_at:null,map_reviewed:null};return town;
}
