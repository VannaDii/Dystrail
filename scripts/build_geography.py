"""Build local map assets from Census KML and OSRM road responses (stdlib only).
Defaults to the reproducible, compressed geographic inputs in docs/ux/map-sources.
No network requests run in the game; geometry is baked into local assets.
"""
import argparse, gzip, json, math, xml.etree.ElementTree as ET
from pathlib import Path
from route_catalog import CITIES, setting
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--inputs',type=Path,default=Path('docs/ux/map-sources/west-coast-v1'));root=parser.parse_args().inputs
def source(name):
    path=root/name
    if name == 'states.kml' and not path.exists() and not (root/(name+'.gz')).exists():
        return gzip.decompress((root.parent/(name+'.gz')).read_bytes())
    return path.read_bytes() if path.exists() else gzip.decompress((root/(name+'.gz')).read_bytes())
ns={'k':'http://www.opengis.net/kml/2.2'}
def albers(lon,lat,lon0=-96,lat0=37.5,p1=29.5,p2=45.5):
    p1,p2,lat0,lat,lon,lon0=map(math.radians,(p1,p2,lat0,lat,lon,lon0))
    n=(math.sin(p1)+math.sin(p2))/2;c=math.cos(p1)**2+2*n*math.sin(p1)
    rho=math.sqrt(c-2*n*math.sin(lat))/n;r0=math.sqrt(c-2*n*math.sin(lat0))/n;t=n*(lon-lon0)
    return rho*math.sin(t),rho*math.cos(t)-r0
states=[]
for mark in ET.fromstring(source('states.kml')).findall('.//k:Placemark',ns):
    props={e.attrib['name']:e.text for e in mark.findall('.//k:SimpleData',ns)}
    code=props['STUSPS']
    if code in ['PR','VI','GU','MP','AS']:continue
    rings=[]
    for poly in mark.findall('.//k:Polygon',ns):
        rings.extend([[(float(p.split(',')[0]),float(p.split(',')[1])) for p in e.text.split()] for e in poly.findall('.//k:coordinates',ns)])
    states.append({'code':code,'rings':rings})
main=[albers(*p) for s in states if s['code'] not in ['AK','HI'] for r in s['rings'] for p in r]
minx,miny=min(p[0] for p in main),min(p[1] for p in main);maxx,maxy=max(p[0] for p in main),max(p[1] for p in main)
scale=min(930/(maxx-minx),520/(maxy-miny))
def project(lon,lat):
    x,y=albers(lon,lat);return round(35+(x-minx)*scale,2),round(32+(y-miny)*scale,2)
def inset(rings,code):
    def raw(lon,lat):
        if code=='AK':return albers(lon-360 if lon>0 else lon,lat,-154,58,55,65)
        return lon*math.cos(math.radians(20)), -lat
    pts=[raw(*p) for r in rings for p in r];ax,ay=min(p[0] for p in pts),min(p[1] for p in pts);bx,by=max(p[0] for p in pts),max(p[1] for p in pts)
    ox,oy,w,h=(40,490,195,135) if code=='AK' else (275,555,100,70)
    k=min(w/(bx-ax),h/(by-ay))
    return [[(round(ox+(raw(*p)[0]-ax)*k,2),round(oy+(raw(*p)[1]-ay)*k,2)) for p in r] for r in rings]
paths=[];labels=[]
for s in states:
    code=s['code'];rings=s['rings']
    if code=='HI':rings=[r for r in rings if any(-161<p[0]<-154 and 18<p[1]<23 for p in r)]
    rings=inset(rings,code) if code in ['AK','HI'] else [[project(*p) for p in r] for r in rings]
    d=' '.join('M'+'L'.join(f'{x},{y}' for x,y in r)+'Z' for r in rings)
    fill='#bec79e' if code in ['CO','NE','KS','IA','MN','TX','OK','MO'] else '#a9bbb1' if code in ['IL','IN','OH','MI','WI','PA','NY'] else '#c7bca0' if code in ['MD','VA','DE','DC'] else '#94aaa3'
    paths.append(f'<path data-state="{code}" d="{d}" fill="{fill}" stroke="#3d5555" stroke-width=".9" fill-rule="evenodd"/>')
    ring=max(rings,key=lambda r:abs(sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(r,r[1:]))))
    area=sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(ring,ring[1:]))
    if abs(area)>1:
        cx=sum((a[0]+b[0])*(a[0]*b[1]-b[0]*a[1]) for a,b in zip(ring,ring[1:]))/(3*area)
        cy=sum((a[1]+b[1])*(a[0]*b[1]-b[0]*a[1]) for a,b in zip(ring,ring[1:]))/(3*area)
        if code not in ['DC','RI','DE','CT','MA','NJ']:labels.append(f'<text x="{cx:.1f}" y="{cy:.1f}" text-anchor="middle" fill="#23382f" font-family="monospace" font-size="10">{code}</text>')
svg='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 660"><rect width="1000" height="660" fill="#193d49"/>'+''.join(paths)+''.join(labels)+'<path d="M30 480H250V635M260 545H390V635" fill="none" stroke="#87a4a0" stroke-width="1" stroke-dasharray="4 4"/></svg>'
Path('dystrail-web/static/img/map/us-states.svg').write_text(svg)

routes=[]
for persona in ['journalist','organizer','whistleblower','lobbyist','staffer','satirist']:
    response=json.loads(source(persona+'.json'));r=response['routes'][0];names=json.loads(source(persona+'-cities.json'));coordinates=r['geometry']['coordinates']
    distances=[d for l in r['legs'] for d in l['annotation']['distance']];cumulative=[0]
    for d in distances:cumulative.append(cumulative[-1]+d/1609.344)
    cursor=0;stops=[];indexes={0,len(coordinates)-1}
    for i,name in enumerate(names):
        if i:cursor+=len(r['legs'][i-1]['annotation']['distance'])
        indexes.add(cursor);lon,lat=coordinates[cursor];x,y=project(lon,lat)
        region, scene = setting(name, i >= names.index('Chicago'))
        stops.append({'name':name,'state':CITIES[name][2],'mile':round(cumulative[cursor]),'x':x,'y':y,'lon':lon,'lat':lat,'region':region,'scene':scene})
    # Keep real highway turns at map resolution, plus every service location exactly.
    last=None
    for i,(lon,lat) in enumerate(coordinates):
        pos=project(lon,lat)
        if last is None or math.dist(pos,last)>=.5:indexes.add(i);last=pos
    points=[]
    for i in sorted(indexes):
        lon,lat=coordinates[i];x,y=project(lon,lat);points.append({'mile':round(cumulative[i],3),'x':x,'y':y,'lon':lon,'lat':lat})
    routes.append({'id':persona,'total_miles':round(cumulative[-1],3),'stops':stops,'points':points})
Path('dystrail-game/data/routes.json').write_text(json.dumps(routes,separators=(',',':'))+'\n')
print('Built',len(states),'state/DC outlines and',len(routes),'routes; bytes',len(svg))
for r in routes:print(r['id'],r['total_miles'],len(r['points']))
