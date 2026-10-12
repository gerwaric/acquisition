"""Layer-1 contamination scan: lexical overlap between non-Tom C++ files and the Rust tree."""
import re, sys, os, collections, pathlib
ROOT = pathlib.Path(sys.argv[1]); RS = ROOT/"spikes/rust-playground"
GT = (ROOT/"docs/design/network-ground-truth.md").read_text(errors="replace").lower()

def cpp_files():
    for p in (ROOT/"src").rglob("*"):
        if p.suffix in (".cpp",".h") and p.is_file():
            t = p.read_text(errors="replace")
            m = re.search(r"CopyrightText:\s*(.*)", t)
            holder = re.sub(r"^[\d\s,–-]*","", m.group(1)).split("<")[0].strip() if m else "MISSING"
            if "Tom Holz" not in holder: yield p, t, holder
def rs_files():
    for p in RS.rglob("*.rs"):
        if "target" not in p.parts: yield p, p.read_text(errors="replace")

STR = re.compile(r'"((?:[^"\\\n]|\\.)*)"')
LINE_C = re.compile(r"//+!?\s?(.*)")
BLOCK_C = re.compile(r"/\*(.*?)\*/", re.S)
IDENT = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")
def shingles(text, n=6):
    w = re.findall(r"[a-z0-9']+", text.lower())
    return {" ".join(w[i:i+n]) for i in range(len(w)-n+1)}
def ident_norm(s):  # split camel/snake into lowercase word tuple so FooBar == foo_bar
    parts = re.findall(r"[A-Z]+(?![a-z])|[A-Z]?[a-z]+|\d+", s)
    return " ".join(p.lower() for p in parts)

def lineno(text, needle):
    i = text.find(needle); return text.count("\n",0,i)+1 if i>=0 else 0

def harvest(files):
    strings, comments, idents = {}, {}, collections.Counter()
    ident_loc = {}
    for p, t, *_ in files:
        for m in STR.finditer(t):
            s = m.group(1)
            if len(s) >= 8: strings.setdefault(s, (p, t.count("\n",0,m.start())+1))
        ctext = " ".join(LINE_C.findall(t)) + " " + " ".join(BLOCK_C.findall(t))
        for sh in shingles(ctext): comments.setdefault(sh, p)
        for m in IDENT.finditer(t):
            s = m.group(0)
            if len(s) >= 8 and not s.isupper():
                k = ident_norm(s); idents[k] += 1; ident_loc.setdefault(k, (s, p))
    return strings, comments, idents, ident_loc

C = list(cpp_files()); R = list(rs_files())
cs, cc, ci, cil = harvest(C); rs_, rc, ri, ril = harvest(R)
rel = lambda p: str(p.relative_to(ROOT))
print(f"C++ files (non-Tom): {len(C)}   Rust files: {len(R)}")
print(f"C++ literals>=8: {len(cs)}  Rust: {len(rs_)}   C++ comment shingles: {len(cc)}  Rust: {len(rc)}   C++ idents>=8: {len(ci)}  Rust: {len(ri)}")

def is_fact(s):
    sl = s.lower()
    return (sl in GT or sl.strip("/") in GT
            or re.fullmatch(r"[a-z][a-zA-Z0-9_]*", s) is not None   # json key
            or sl.startswith(("http","/api","/character","/stash","application/","content-","x-rate","retry-","authorization","bearer"))
            or re.fullmatch(r"[A-Za-z-]+:", s) is not None)        # header name

print("\n=== A. shared string literals (len>=8), facts filtered ===")
hits = sorted(set(cs)&set(rs_), key=lambda s:-len(s))
facts = [s for s in hits if is_fact(s)]
for s in hits:
    if is_fact(s): continue
    cp, cl = cs[s]; rp, rl = rs_[s]
    print(f"  {s!r}\n      C++ {rel(cp)}:{cl}\n      Rust {rel(rp)}:{rl}")
print(f"  ({len(facts)} shared literals filtered as GGG interface facts / JSON keys)")
if facts: print("   filtered:", ", ".join(repr(f) for f in facts[:60]))

print("\n=== B. shared 6-word comment shingles ===")
sh = sorted(set(cc)&set(rc))
# merge adjacent shingles into runs per (cpp file, rust file)
byfiles = collections.defaultdict(list)
for s in sh: byfiles[(rel(cc[s]), rel(rc[s]))].append(s)
for (cp, rp), ss in sorted(byfiles.items(), key=lambda kv:-len(kv[1])):
    print(f"  {len(ss):3} shingles  C++ {cp}  <->  Rust {rp}")
    for s in ss[:4]: print(f"        '{s}'")
print(f"  total shared shingles: {len(sh)}")

print("\n=== C. shared identifiers (normalized), rarest first, facts filtered ===")
shared = [k for k in set(ci)&set(ri) if k.replace(" ","") not in GT and k.replace(" ","_") not in GT]
shared.sort(key=lambda k: ci[k]+ri[k])
for k in shared[:80]:
    cs_, cp = cil[k]; rs__, rp = ril[k]
    print(f"  {cs_:32} {rs__:32} C++ {rel(cp)}  Rust {rel(rp)}")
print(f"  total shared identifiers: {len(shared)} (of {len(set(ci)&set(ri))} before fact filter)")

print("\n=== D. exposure per C++ file (literal + shingle + ident hits) ===")
score = collections.Counter()
for s in hits:
    if not is_fact(s): score[rel(cs[s][0])] += 3
for s in sh: score[rel(cc[s])] += 2
for k in shared: score[rel(cil[k][1])] += 1
for f, n in score.most_common(25): print(f"  {n:4}  {f}")
