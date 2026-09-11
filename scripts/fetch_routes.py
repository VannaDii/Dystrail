"""Refresh OSM road data using the public OSRM router; this is a development-only network task."""
import argparse, json, urllib.request
from pathlib import Path
cities={
'Denver':(-104.9903,39.7392),'North Platte':(-100.7654,41.1403),'Omaha':(-95.9345,41.2565),'Des Moines':(-93.6250,41.5868),'Iowa City':(-91.5302,41.6611),
'Minneapolis':(-93.2650,44.9778),'La Crosse':(-91.2396,43.8014),'Madison':(-89.4012,43.0731),
'Kansas City':(-94.5786,39.0997),'Columbia':(-92.3341,38.9517),'St. Louis':(-90.1994,38.6270),'Springfield, IL':(-89.6501,39.7817),
'Austin':(-97.7431,30.2672),'Waco':(-97.1467,31.5493),'Dallas':(-96.7970,32.7767),'Oklahoma City':(-97.5164,35.4676),'Tulsa':(-95.9928,36.1540),'Joplin':(-94.5133,37.0842),'Springfield, MO':(-93.2923,37.2089),
'Chicago':(-87.6298,41.8781),'South Bend':(-86.2520,41.6764),'Toledo':(-83.5552,41.6639),'Cleveland':(-81.6944,41.4993),'Pittsburgh':(-79.9959,40.4406),'Cumberland':(-78.7625,39.6529),'Hagerstown':(-77.7200,39.6418),'Frederick':(-77.4105,39.4143),'D.C.':(-77.0091,38.8899)}
common=['Chicago','South Bend','Toledo','Cleveland','Pittsburgh','Cumberland','Hagerstown','Frederick','D.C.']
starts={'whistleblower':['Denver','North Platte','Omaha','Des Moines','Iowa City'],'staffer':['Omaha','Des Moines','Iowa City'],'journalist':['Minneapolis','La Crosse','Madison'],'organizer':['Kansas City','Columbia','St. Louis','Springfield, IL'],'lobbyist':['St. Louis','Springfield, IL'],'satirist':['Austin','Waco','Dallas','Oklahoma City','Tulsa','Joplin','Springfield, MO','St. Louis','Springfield, IL']}
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);root=parser.parse_args().output;root.mkdir(parents=True,exist_ok=True)
for persona,start in starts.items():
 names=start+common
 coords=';'.join(','.join(str(v) for v in cities[name]) for name in names)
 url='https://router.project-osrm.org/route/v1/driving/'+coords+'?overview=full&geometries=geojson&steps=false&annotations=distance'
 req=urllib.request.Request(url,headers={'User-Agent':'DystopianTrail-geography-build/1.0'})
 data=json.load(urllib.request.urlopen(req,timeout=60));assert data['code']=='Ok',data
 (root/(persona+'.json')).write_text(json.dumps(data))
 (root/(persona+'-cities.json')).write_text(json.dumps(names))
 print(persona,len(names),round(data['routes'][0]['distance']/1609.344,1),len(data['routes'][0]['geometry']['coordinates']),flush=True)
