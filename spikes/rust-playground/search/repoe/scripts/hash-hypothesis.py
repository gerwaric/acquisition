#!/usr/bin/env python3
"""repoe step 2, the negative result: is the `Stats.dat` hash a known string hash of the stat id?

For every translation entry with one stat id and one single-stat `stat_N` trade id (6,678 pairs at
e2bd511a), computes 19 common string hashes of the id — each plain, NUL-terminated and lower-cased —
and counts those for which MurmurHash2 (seed 0x02312233) of the 4-byte value equals the trade number.
Prints a line per candidate that scores; none did (`data/hash-recipe.md`). Writes nothing.

Input (read-only; the clone must sit at the manifest's commit or this refuses to run):
  ../../../../../poe1/data/stat_translations.json
"""
import json, zlib, collections, struct
M = 0xffffffff
def murmur2(data: bytes, seed: int) -> int:
    m = 0x5bd1e995; r = 24
    ln = len(data); h = (seed ^ ln) & M; i = 0
    while ln >= 4:
        k = int.from_bytes(data[i:i+4], 'little')
        k = (k * m) & M; k ^= k >> r; k = (k * m) & M
        h = (h * m) & M; h ^= k
        i += 4; ln -= 4
    if ln:
        h ^= int.from_bytes(data[i:] + b'\0'*(4-ln), 'little'); h = (h * m) & M
    h ^= h >> 13; h = (h * m) & M; h ^= h >> 15
    return h
def fnv1_32(b):
    h=0x811c9dc5
    for c in b: h=(h*0x01000193)&M; h^=c
    return h
def fnv1a_32(b):
    h=0x811c9dc5
    for c in b: h^=c; h=(h*0x01000193)&M
    return h
def fnv1_64(b):
    h=0xcbf29ce484222325
    for c in b: h=(h*0x100000001b3)&0xffffffffffffffff; h^=c
    return h
def fnv1a_64(b):
    h=0xcbf29ce484222325
    for c in b: h^=c; h=(h*0x100000001b3)&0xffffffffffffffff
    return h
def djb2(b):
    h=5381
    for c in b: h=(h*33+c)&M
    return h
def djb2x(b):
    h=5381
    for c in b: h=((h*33)^c)&M
    return h
def sdbm(b):
    h=0
    for c in b: h=(c+(h<<6)+(h<<16)-h)&M
    return h
def java(b):
    h=0
    for c in b: h=(31*h+c)&M
    return h
def oaat(b):
    h=0
    for c in b:
        h=(h+c)&M; h=(h+(h<<10))&M; h^=h>>6
    h=(h+(h<<3))&M; h^=h>>11; h=(h+(h<<15))&M
    return h
def murmur3_32(data, seed=0):
    c1=0xcc9e2d51; c2=0x1b873593; ln=len(data); h=seed; i=0
    def rotl(x,r): return ((x<<r)|(x>>(32-r)))&M
    while i+4<=ln:
        k=int.from_bytes(data[i:i+4],'little'); k=(k*c1)&M; k=rotl(k,15); k=(k*c2)&M
        h^=k; h=rotl(h,13); h=(h*5+0xe6546b64)&M; i+=4
    k=0
    tail=data[i:]
    if tail:
        k=int.from_bytes(tail,'little'); k=(k*c1)&M; k=rotl(k,15); k=(k*c2)&M; h^=k
    h^=ln; h^=h>>16; h=(h*0x85ebca6b)&M; h^=h>>13; h=(h*0xc2b2ae35)&M; h^=h>>16
    return h
cands = {
 'fnv1_32': fnv1_32, 'fnv1a_32': fnv1a_32,
 'fnv1_64lo': lambda b: fnv1_64(b)&M, 'fnv1_64hi': lambda b: fnv1_64(b)>>32, 'fnv1_64fold': lambda b: (fnv1_64(b)&M)^(fnv1_64(b)>>32),
 'fnv1a_64lo': lambda b: fnv1a_64(b)&M, 'fnv1a_64hi': lambda b: fnv1a_64(b)>>32, 'fnv1a_64fold': lambda b: (fnv1a_64(b)&M)^(fnv1a_64(b)>>32),
 'murmur2_0': lambda b: murmur2(b,0), 'murmur2_seed': lambda b: murmur2(b,0x02312233),
 'murmur3_0': lambda b: murmur3_32(b,0), 'murmur3_seed': lambda b: murmur3_32(b,0x02312233),
 'crc32': lambda b: zlib.crc32(b)&M, 'adler32': lambda b: zlib.adler32(b)&M,
 'djb2': djb2, 'djb2x': djb2x, 'sdbm': sdbm, 'java': java, 'oaat': oaat,
}
import os, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__))
CLONES = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(HERE))))))
POE1 = os.path.join(CLONES, 'poe1'); POE1_COMMIT = 'e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9'  # MANIFEST.md
head = subprocess.run(['git', '-C', POE1, 'rev-parse', 'HEAD'], capture_output=True, text=True, check=True).stdout.strip()
if head != POE1_COMMIT:
    sys.exit(f"{POE1} is at {head}, not the manifest's {POE1_COMMIT}: add a manifest row before re-running")
st = json.load(open(os.path.join(POE1, 'data', 'stat_translations.json')))
pairs = []  # (stat id, trade number) for single-id entries with a single explicit stat_N id
for e in st:
    if len(e['ids']) != 1: continue
    nums = {ts['id'].split('.',1)[1] for ts in (e.get('trade_stats') or []) if ts["id"].split(".",1)[1].startswith("stat_") and "|" not in ts["id"]}
    if len(nums) == 1:
        pairs.append((e['ids'][0], int(nums.pop()[5:])))
print('pairs', len(pairs))
print('sanity: murmur2(b"",0)=', murmur2(b"",0))
for name, fn in cands.items():
    for variant, tf in (('', lambda s: s.encode()), ('+nul', lambda s: s.encode()+b'\0'), ('lower', lambda s: s.lower().encode())):
        hits = 0
        for sid, num in pairs:
            hv = fn(tf(sid))
            if murmur2(struct.pack('<I', hv), 0x02312233) == num: hits += 1
        if hits: print(f'{name}{variant}: {hits}/{len(pairs)}')
print('done; strength ids:', [e['ids'] for e in st if any(ts['id']=='explicit.stat_4080418644' for ts in (e.get('trade_stats') or []))])
