"""Read-only RGBA/RGB PNG opacity audit; never rewrites artwork."""
import json, struct, sys, zlib
from pathlib import Path

def inspect(path):
    raw=Path(path).read_bytes()
    assert raw[:8]==b'\x89PNG\r\n\x1a\n'
    pos=8;parts=[];header=None;transparent=False
    while pos<len(raw):
        size=struct.unpack('>I',raw[pos:pos+4])[0]
        kind=raw[pos+4:pos+8];data=raw[pos+8:pos+8+size];pos+=size+12
        if kind==b'IHDR':header=struct.unpack('>IIBBBBB',data)
        if kind==b'IDAT':parts.append(data)
        if kind==b'tRNS':transparent=True
    width,height,depth,color,_,_,interlace=header
    if color==2 and not transparent:
        return {'path':str(path),'width':width,'height':height,'opaque':True,'alpha_min':255,'nonopaque_pixels':0}
    assert depth==8 and color==6 and interlace==0,(path,header)
    stream=zlib.decompress(b''.join(parts));stride=width*4;previous=bytearray(stride);offset=0;minimum=255;count=0
    for _ in range(height):
        method=stream[offset];row=bytearray(stream[offset+1:offset+1+stride]);offset+=stride+1
        for i in range(stride):
            left=row[i-4] if i>=4 else 0;up=previous[i];corner=previous[i-4] if i>=4 else 0
            if method==1:predictor=left
            elif method==2:predictor=up
            elif method==3:predictor=(left+up)//2
            elif method==4:
                p=left+up-corner;a=abs(p-left);b=abs(p-up);c=abs(p-corner)
                predictor=left if a<=b and a<=c else up if b<=c else corner
            else:
                assert method==0;predictor=0
            row[i]=(row[i]+predictor)&255
        alpha=row[3::4];minimum=min(minimum,min(alpha));count+=sum(a<255 for a in alpha);previous=row
    return {'path':str(path),'width':width,'height':height,'opaque':count==0,'alpha_min':minimum,'nonopaque_pixels':count}

if __name__=='__main__':
    records=[inspect(p) for p in sys.argv[1:]]
    print(json.dumps(records,indent=2))
    sys.exit(0 if all(r['opaque'] for r in records) else 1)
