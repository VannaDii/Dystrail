"""Refresh OSM road data using the public OSRM router; this is a development-only network task."""
import argparse, json, urllib.request
from pathlib import Path
from route_catalog import CITIES, COMMON, STARTS
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);root=parser.parse_args().output;root.mkdir(parents=True,exist_ok=True)
for persona,start in STARTS.items():
 names=start+COMMON
 coords=';'.join(','.join(str(v) for v in CITIES[name][:2]) for name in names)
 url='https://router.project-osrm.org/route/v1/driving/'+coords+'?overview=full&geometries=geojson&steps=false&annotations=distance'
 req=urllib.request.Request(url,headers={'User-Agent':'DystopianTrail-geography-build/1.0'})
 data=json.load(urllib.request.urlopen(req,timeout=60));assert data['code']=='Ok',data
 (root/(persona+'.json')).write_text(json.dumps(data))
 (root/(persona+'-cities.json')).write_text(json.dumps(names))
 print(persona,len(names),round(data['routes'][0]['distance']/1609.344,1),len(data['routes'][0]['geometry']['coordinates']),flush=True)
