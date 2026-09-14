//! engine-bench — is an in-memory scan over the parsed corpus already instant at league scale,
//! or is a derived search schema in SQLite needed?
//!
//! Inputs (read-only):
//!   ../raw/corpus.jsonl                            built by ../scripts/build-corpus.py from the two stores
//!   ../../repoe/data/template-vs-translation.csv   display template -> trade stat id (identity B)
//! Output:
//!   ../data/results.csv, ../data/numbers.csv        (written by this binary; the shell wrapper adds the header line)
//!
//! Engines: (A) a scan over a compact parsed struct per item; (B) SQLite, a wide attributes table
//! plus an indexed (item, line, value) table, built twice — keyed by template text, keyed by stat id.
//! Every body is parsed defensively: a malformed line is skipped and counted, never a panic.

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

// ---------------------------------------------------------------- allocator

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        LIVE.fetch_add(l.size(), Ordering::Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Ordering::Relaxed);
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        LIVE.fetch_add(new, Ordering::Relaxed);
        LIVE.fetch_sub(l.size(), Ordering::Relaxed);
        System.realloc(p, l, new)
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn live() -> usize {
    LIVE.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------- interning

#[derive(Default)]
struct Interner {
    map: HashMap<String, u32>,
    vec: Vec<String>,
}

impl Interner {
    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&i) = self.map.get(s) {
            return i;
        }
        let i = self.vec.len() as u32;
        self.vec.push(s.to_string());
        self.map.insert(s.to_string(), i);
        i
    }
    fn get(&self, s: &str) -> u32 {
        self.map.get(s).copied().unwrap_or(NONE)
    }
    fn name(&self, i: u32) -> &str {
        self.vec.get(i as usize).map(|s| s.as_str()).unwrap_or("")
    }
    fn len(&self) -> usize {
        self.vec.len()
    }
}

const NONE: u32 = u32::MAX;
/// A constant that no interned id can equal: a query term the corpus does not know.
const NOMATCH: u32 = u32::MAX - 1;

// ---------------------------------------------------------------- the parsed item

#[derive(Clone)]
struct Line {
    arr: u8,     // interned mod-array id
    tmpl: u32,   // display template, numbers -> '#'
    stat: u32,   // trade stat id via template-vs-translation.csv, NONE when unmapped
    value: f32,  // mean of the line's numbers (the C++ app's rule)
}

#[derive(Clone)]
struct Item {
    gid: Box<str>,
    pretty: Box<str>, // lowercased `name + " " + typeLine`, the C++ Name haystack
    base: Box<str>,   // lowercased baseType
    frame: u8,
    tab: u32,
    league: u16,
    ilvl: u16,
    req_level: u16,
    req_str: u16,
    req_dex: u16,
    req_int: u16,
    armour: u32,
    evasion: u32,
    energy_shield: u32,
    quality: u16,
    identified: bool,
    corrupted: bool,
    lines: Box<[Line]>,
}

// ---------------------------------------------------------------- template / value extraction

/// The census's rule: every number becomes `#`; a sign right after a digit or a `#` is a range dash.
/// Returns the template and the mean of the numbers (the C++ app's per-line value).
fn template_and_value(s: &str) -> (String, f32) {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut sum = 0f64;
    let mut n = 0usize;
    let mut i = 0usize;
    let mut prev = 0u8;
    while i < b.len() {
        let c = b[i];
        let sign = (c == b'+' || c == b'-') && i + 1 < b.len() && b[i + 1].is_ascii_digit();
        let starts = (c.is_ascii_digit() || sign) && !(prev.is_ascii_digit() || prev == b'#');
        if starts {
            let start = i;
            if sign {
                i += 1;
            }
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if let Ok(v) = std::str::from_utf8(&b[start..i]).unwrap_or("0").parse::<f64>() {
                sum += v;
                n += 1;
            }
            out.push(b'#');
            prev = b'#';
        } else {
            if c == b'\n' {
                out.push(b'\\');
                out.push(b'n');
            } else {
                out.push(c);
            }
            prev = c;
            i += 1;
        }
    }
    let t = String::from_utf8(out).unwrap_or_default();
    (t, if n == 0 { 0.0 } else { (sum / n as f64) as f32 })
}

fn first_number(v: &serde_json::Value) -> Option<f32> {
    // properties/requirements: values = [[ "text", flag ], ...]
    let vals = v.get("values")?.as_array()?;
    let first = vals.first()?.as_array()?.first()?.as_str()?;
    let (_, x) = template_and_value(first);
    Some(x)
}

/// PoE2 wraps some names in `[Tag|Display]`; take the display half.
fn plain(name: &str) -> &str {
    if let Some(rest) = name.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            let inner = &rest[..end];
            return inner.rsplit('|').next().unwrap_or(inner);
        }
    }
    name
}

// ---------------------------------------------------------------- load

struct Corpus {
    items: Vec<Item>,
    tmpl: Interner,
    stat: Interner,
    arrays: Interner,
    tabs: Interner,
    frames: Interner,
    leagues: Interner,
    skipped: usize,
}

fn load(raw: &str, copies: usize, stat_of: &HashMap<(String, String), String>) -> (Corpus, u128) {
    let mut c = Corpus {
        items: Vec::with_capacity(36_500 * copies),
        tmpl: Interner::default(),
        stat: Interner::default(),
        arrays: Interner::default(),
        tabs: Interner::default(),
        frames: Interner::default(),
        leagues: Interner::default(),
        skipped: 0,
    };
    let t0 = Instant::now();
    for copy in 0..copies {
        for line in raw.lines() {
            if line.is_empty() {
                continue;
            }
            let v: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => {
                    c.skipped += 1;
                    continue;
                }
            };
            let (co, it) = match (v.get("c"), v.get("i")) {
                (Some(a), Some(b)) if b.is_object() => (a, b),
                _ => {
                    c.skipped += 1;
                    continue;
                }
            };
            let s = |o: &serde_json::Value, k: &str| -> String {
                o.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
            };
            let name = s(it, "name");
            let type_line = s(it, "typeLine");
            let pretty = if name.is_empty() {
                type_line.to_lowercase()
            } else {
                format!("{name} {type_line}").to_lowercase()
            };
            let gid = if copy == 0 {
                s(co, "id")
            } else {
                format!("{}#{copy}", s(co, "id"))
            };
            let mut item = Item {
                gid: gid.into_boxed_str(),
                pretty: pretty.into_boxed_str(),
                base: s(it, "baseType").to_lowercase().into_boxed_str(),
                frame: c.frames.intern(&s(it, "frameTypeId")) as u8,
                tab: c.tabs.intern(&s(co, "tab")),
                league: c.leagues.intern(&s(co, "league")) as u16,
                ilvl: it.get("ilvl").and_then(|x| x.as_u64()).unwrap_or(0) as u16,
                req_level: 0,
                req_str: 0,
                req_dex: 0,
                req_int: 0,
                armour: 0,
                evasion: 0,
                energy_shield: 0,
                quality: 0,
                identified: it.get("identified").and_then(|x| x.as_bool()).unwrap_or(false),
                corrupted: it.get("corrupted").and_then(|x| x.as_bool()).unwrap_or(false),
                lines: Box::new([]),
            };
            if let Some(rs) = it.get("requirements").and_then(|x| x.as_array()) {
                for r in rs {
                    let n = s(r, "name");
                    let x = first_number(r).unwrap_or(0.0) as u16;
                    match plain(&n) {
                        "Level" => item.req_level = x,
                        "Str" | "Strength" => item.req_str = x,
                        "Dex" | "Dexterity" => item.req_dex = x,
                        "Int" | "Intelligence" => item.req_int = x,
                        _ => {}
                    }
                }
            }
            if let Some(ps) = it.get("properties").and_then(|x| x.as_array()) {
                for p in ps {
                    let n = s(p, "name");
                    let x = first_number(p).unwrap_or(0.0);
                    match plain(&n) {
                        "Armour" => item.armour = x as u32,
                        "Evasion Rating" => item.evasion = x as u32,
                        "Energy Shield" => item.energy_shield = x as u32,
                        "Quality" => item.quality = x as u16,
                        _ => {}
                    }
                }
            }
            let mut lines: Vec<Line> = Vec::new();
            if let Some(obj) = it.as_object() {
                for (k, v) in obj {
                    if !k.ends_with("Mods") {
                        continue;
                    }
                    let Some(arr) = v.as_array() else { continue };
                    let aid = c.arrays.intern(k) as u8;
                    for e in arr {
                        let text = match e {
                            serde_json::Value::String(t) => Some(t.as_str()),
                            serde_json::Value::Object(o) => {
                                o.get("description").and_then(|d| d.as_str())
                            }
                            _ => None,
                        };
                        let Some(text) = text else { continue };
                        let (t, val) = template_and_value(text);
                        let stat = match stat_of.get(&(k.clone(), t.clone())) {
                            Some(sid) => c.stat.intern(sid),
                            None => NONE,
                        };
                        lines.push(Line {
                            arr: aid,
                            tmpl: c.tmpl.intern(&t),
                            stat,
                            value: val,
                        });
                    }
                }
            }
            item.lines = lines.into_boxed_slice();
            c.items.push(item);
        }
    }
    (c, t0.elapsed().as_millis())
}

// ---------------------------------------------------------------- template -> stat id

/// Minimal CSV reader: RFC-4180 quoting, `\n` terminators, first line a `#` comment.
fn read_csv(path: &str) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(path).expect("template-vs-translation.csv");
    let mut rows = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                cur.push(ch);
            }
        } else {
            match ch {
                '"' => quoted = true,
                ',' => row.push(std::mem::take(&mut cur)),
                '\n' => {
                    row.push(std::mem::take(&mut cur));
                    rows.push(std::mem::take(&mut row));
                }
                '\r' => {}
                _ => cur.push(ch),
            }
        }
    }
    if !cur.is_empty() || !row.is_empty() {
        row.push(cur);
        rows.push(row);
    }
    rows.retain(|r| !(r.len() == 1 && r[0].is_empty()));
    rows
}

/// (array, template) -> the stat id the trade site uses, e.g. `stat_3299347043`.
/// A template with several entries is ambiguous: the first trade id wins, counted.
fn stat_map(path: &str) -> (HashMap<(String, String), String>, usize, usize) {
    let rows = read_csv(path);
    let mut it = rows.into_iter();
    let _comment = it.next();
    let header = it.next().expect("header");
    let col = |n: &str| header.iter().position(|h| h == n).expect(n);
    let (ia, it_, itr, ine) = (col("array"), col("template"), col("trade_ids"), col("n_entries"));
    let mut map = HashMap::new();
    let mut ambiguous = 0;
    let mut mapped = 0;
    for r in it {
        if r.len() <= itr {
            continue;
        }
        let ids = r[itr].trim();
        if ids.is_empty() {
            continue;
        }
        let first = ids.split_whitespace().next().unwrap_or("");
        let bare = first.split_once('.').map(|(_, b)| b).unwrap_or(first);
        if r[ine].parse::<u32>().unwrap_or(0) > 1 {
            ambiguous += 1;
        }
        mapped += 1;
        map.insert((r[ia].clone(), r[it_].clone()), bare.to_string());
    }
    (map, mapped, ambiguous)
}

// ---------------------------------------------------------------- the queries

const LIFE: &str = "# to maximum Life";
const COLD: &str = "#% to Cold Resistance";
const FIRE: &str = "#% to Fire Resistance";
const EXPL: &str = "explicitMods";

struct Ids {
    expl: u8,
    life: u32,
    cold: u32,
    fire: u32,
}

fn ids(c: &Corpus, by_stat: bool, sm: &HashMap<(String, String), String>) -> Ids {
    let pick = |t: &str| -> u32 {
        let id = if by_stat {
            match sm.get(&(EXPL.to_string(), t.to_string())) {
                Some(s) => c.stat.get(s),
                None => NONE,
            }
        } else {
            c.tmpl.get(t)
        };
        if id == NONE {
            NOMATCH
        } else {
            id
        }
    };
    Ids {
        expl: c.arrays.get(EXPL) as u8,
        life: pick(LIFE),
        cold: pick(COLD),
        fire: pick(FIRE),
    }
}

const QUERIES: [&str; 12] = [
    "q01-mod-value",
    "q02-mod-exists",
    "q03-name-substring",
    "q04-base-with-mod",
    "q05-rarity-ilvl",
    "q06-one-tab",
    "q07-count-per-tab",
    "q08-count-group",
    "q09-weight-sum",
    "q10-boolean-or",
    "q11-cpp-mods-two-rows",
    "q12-leveling-set",
];

fn line_id(l: &Line, by_stat: bool) -> u32 {
    if by_stat {
        l.stat
    } else {
        l.tmpl
    }
}

/// The scan: one pass over the parsed structs, no index of any kind beyond the interning done at load.
fn scan(c: &Corpus, q: usize, k: &Ids, by_stat: bool) -> usize {
    let mut hits: Vec<u32> = Vec::new();
    match q {
        0 => {
            for (n, it) in c.items.iter().enumerate() {
                if it.lines.iter().any(|l| {
                    l.arr == k.expl && line_id(l, by_stat) == k.cold && l.value >= 30.0
                }) {
                    hits.push(n as u32);
                }
            }
        }
        1 => {
            for (n, it) in c.items.iter().enumerate() {
                if it
                    .lines
                    .iter()
                    .any(|l| l.arr == k.expl && line_id(l, by_stat) == k.life)
                {
                    hits.push(n as u32);
                }
            }
        }
        2 => {
            for (n, it) in c.items.iter().enumerate() {
                if it.pretty.contains("forbidden") {
                    hits.push(n as u32);
                }
            }
        }
        3 => {
            for (n, it) in c.items.iter().enumerate() {
                if &*it.base == "two-stone ring"
                    && it.lines.iter().any(|l| {
                        l.arr == k.expl && line_id(l, by_stat) == k.cold && l.value >= 20.0
                    })
                {
                    hits.push(n as u32);
                }
            }
        }
        4 => {
            let rare = c.frames.get("Rare") as u8;
            for (n, it) in c.items.iter().enumerate() {
                if it.frame == rare && it.ilvl >= 84 {
                    hits.push(n as u32);
                }
            }
        }
        5 => {
            let tab = c.tabs.get("Flasks");
            for (n, it) in c.items.iter().enumerate() {
                if it.tab == tab {
                    hits.push(n as u32);
                }
            }
        }
        6 => {
            let mut per: HashMap<u32, u32> = HashMap::new();
            for it in c.items.iter() {
                *per.entry(it.tab).or_insert(0) += 1;
            }
            return per.len();
        }
        7 => {
            for (n, it) in c.items.iter().enumerate() {
                let mut m = 0;
                let mut seen = [false; 3];
                for l in it.lines.iter() {
                    if l.arr != k.expl {
                        continue;
                    }
                    let id = line_id(l, by_stat);
                    if id == k.life && l.value >= 60.0 && !seen[0] {
                        seen[0] = true;
                        m += 1;
                    } else if id == k.cold && l.value >= 25.0 && !seen[1] {
                        seen[1] = true;
                        m += 1;
                    } else if id == k.fire && l.value >= 25.0 && !seen[2] {
                        seen[2] = true;
                        m += 1;
                    }
                }
                if m >= 2 {
                    hits.push(n as u32);
                }
            }
        }
        8 => {
            for (n, it) in c.items.iter().enumerate() {
                let mut w = 0f32;
                for l in it.lines.iter() {
                    if l.arr != k.expl {
                        continue;
                    }
                    let id = line_id(l, by_stat);
                    if id == k.life {
                        w += l.value;
                    } else if id == k.cold || id == k.fire {
                        w += 2.0 * l.value;
                    }
                }
                if w >= 150.0 {
                    hits.push(n as u32);
                }
            }
        }
        9 => {
            for (n, it) in c.items.iter().enumerate() {
                if it.armour > 1000 || it.req_level < 80 {
                    hits.push(n as u32);
                }
            }
        }
        10 => {
            for (n, it) in c.items.iter().enumerate() {
                let a = it
                    .lines
                    .iter()
                    .any(|l| l.arr == k.expl && line_id(l, by_stat) == k.life && l.value >= 50.0);
                let b = a
                    && it.lines.iter().any(|l| {
                        l.arr == k.expl && line_id(l, by_stat) == k.fire && l.value >= 20.0
                    });
                if b {
                    hits.push(n as u32);
                }
            }
        }
        11 => {
            let magic = c.frames.get("Magic") as u8;
            let rare = c.frames.get("Rare") as u8;
            let uniq = c.frames.get("Unique") as u8;
            for (n, it) in c.items.iter().enumerate() {
                if it.req_level >= 1
                    && it.req_level <= 45
                    && (it.frame == magic || it.frame == rare || it.frame == uniq)
                {
                    hits.push(n as u32);
                }
            }
        }
        _ => unreachable!(),
    }
    hits.len()
}

fn sql(q: usize, k: &Ids, c: &Corpus, by_stat: bool) -> String {
    let name = |id: u32| -> String {
        if id >= NOMATCH {
            "\u{0}nomatch".to_string()
        } else if by_stat {
            c.stat.name(id).to_string()
        } else {
            c.tmpl.name(id).to_string()
        }
    };
    let (life, cold, fire) = (name(k.life), name(k.cold), name(k.fire));
    let e = EXPL;
    match q {
        0 => format!("SELECT DISTINCT item FROM lines WHERE arr='{e}' AND line='{cold}' AND value>=30"),
        1 => format!("SELECT DISTINCT item FROM lines WHERE arr='{e}' AND line='{life}'"),
        2 => "SELECT rid FROM items WHERE pretty LIKE '%forbidden%'".into(),
        3 => format!(
            "SELECT rid FROM items i WHERE base='two-stone ring' AND EXISTS \
             (SELECT 1 FROM lines l WHERE l.item=i.rid AND l.arr='{e}' AND l.line='{cold}' AND l.value>=20)"
        ),
        4 => "SELECT rid FROM items WHERE frame='Rare' AND ilvl>=84".into(),
        5 => "SELECT rid FROM items WHERE tab='Flasks'".into(),
        6 => "SELECT tab, count(*) FROM items GROUP BY tab".into(),
        7 => format!(
            "SELECT item FROM lines WHERE arr='{e}' AND ((line='{life}' AND value>=60) OR \
             (line='{cold}' AND value>=25) OR (line='{fire}' AND value>=25)) \
             GROUP BY item HAVING count(DISTINCT line)>=2"
        ),
        8 => format!(
            "SELECT item FROM lines WHERE arr='{e}' AND line IN ('{life}','{cold}','{fire}') \
             GROUP BY item HAVING sum(CASE WHEN line='{life}' THEN value ELSE 2*value END)>=150"
        ),
        9 => "SELECT rid FROM items WHERE armour>1000 OR req_level<80".into(),
        10 => format!(
            "SELECT item FROM lines WHERE arr='{e}' AND line='{life}' AND value>=50 \
             INTERSECT SELECT item FROM lines WHERE arr='{e}' AND line='{fire}' AND value>=20"
        ),
        11 => "SELECT rid FROM items WHERE req_level BETWEEN 1 AND 45 AND frame IN ('Magic','Rare','Unique')".into(),
        _ => unreachable!(),
    }
}

// ---------------------------------------------------------------- SQLite build

fn build_db(c: &Corpus, path: &str, by_stat: bool) -> (u128, u64, u64) {
    let _ = std::fs::remove_file(path);
    let t0 = Instant::now();
    let mut db = rusqlite::Connection::open(path).expect("open");
    db.execute_batch(
        "PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;
         CREATE TABLE items (rid INTEGER PRIMARY KEY, gid TEXT, pretty TEXT, base TEXT, frame TEXT,
             tab TEXT, league TEXT, ilvl INT, req_level INT, req_str INT, req_dex INT, req_int INT,
             armour INT, evasion INT, energy_shield INT, quality INT, identified INT, corrupted INT);
         CREATE TABLE lines (item INTEGER, arr TEXT, line TEXT, value REAL);",
    )
    .expect("schema");
    let mut nlines = 0u64;
    {
        let tx = db.transaction().expect("tx");
        {
            let mut ins = tx
                .prepare(
                    "INSERT INTO items VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
                )
                .expect("prep");
            let mut insl = tx
                .prepare("INSERT INTO lines VALUES (?1,?2,?3,?4)")
                .expect("prepl");
            for (n, it) in c.items.iter().enumerate() {
                let rid = n as i64;
                ins.execute(rusqlite::params![
                    rid,
                    &*it.gid,
                    &*it.pretty,
                    &*it.base,
                    c.frames.name(it.frame as u32),
                    c.tabs.name(it.tab),
                    c.leagues.name(it.league as u32),
                    it.ilvl as i64,
                    it.req_level as i64,
                    it.req_str as i64,
                    it.req_dex as i64,
                    it.req_int as i64,
                    it.armour as i64,
                    it.evasion as i64,
                    it.energy_shield as i64,
                    it.quality as i64,
                    it.identified as i32,
                    it.corrupted as i32,
                ])
                .expect("ins");
                for l in it.lines.iter() {
                    let id = line_id(l, by_stat);
                    if id == NONE {
                        continue; // unmapped by the stat identity: not addressable, so not a row
                    }
                    let key = if by_stat {
                        c.stat.name(id)
                    } else {
                        c.tmpl.name(id)
                    };
                    insl.execute(rusqlite::params![
                        rid,
                        c.arrays.name(l.arr as u32),
                        key,
                        l.value
                    ])
                    .expect("insl");
                    nlines += 1;
                }
            }
        }
        tx.commit().expect("commit");
    }
    db.execute_batch(
        "CREATE INDEX lines_line ON lines(line, value);
         CREATE INDEX lines_item ON lines(item);
         CREATE INDEX items_frame ON items(frame);
         CREATE INDEX items_tab ON items(tab);
         CREATE INDEX items_ilvl ON items(ilvl);
         CREATE INDEX items_req_level ON items(req_level);
         CREATE INDEX items_armour ON items(armour);
         CREATE INDEX items_base ON items(base);
         ANALYZE;",
    )
    .expect("indexes");
    drop(db);
    let ms = t0.elapsed().as_millis();
    let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    (ms, bytes, nlines)
}

/// Rebuild the in-memory structs from the derived tables instead of from GGG JSON:
/// what a stored projection would cost at load, against the parse (store-as-built Q1, Q5).
fn load_from_projection(path: &str) -> (u128, usize, usize) {
    let before = live();
    let t0 = Instant::now();
    let db = rusqlite::Connection::open(path).expect("open");
    let mut interner = Interner::default();
    let mut items: Vec<Item> = Vec::new();
    {
        let mut st = db
            .prepare("SELECT gid, pretty, base, frame, tab, league, ilvl, req_level, req_str, req_dex, req_int, armour, evasion, energy_shield, quality, identified, corrupted FROM items ORDER BY rid")
            .expect("prep");
        let mut rows = st.query([]).expect("q");
        while let Some(r) = rows.next().expect("row") {
            let g: String = r.get(0).unwrap_or_default();
            let p: String = r.get(1).unwrap_or_default();
            let b: String = r.get(2).unwrap_or_default();
            let f: String = r.get(3).unwrap_or_default();
            let t: String = r.get(4).unwrap_or_default();
            let l: String = r.get(5).unwrap_or_default();
            items.push(Item {
                gid: g.into_boxed_str(),
                pretty: p.into_boxed_str(),
                base: b.into_boxed_str(),
                frame: interner.intern(&f) as u8,
                tab: interner.intern(&t),
                league: interner.intern(&l) as u16,
                ilvl: r.get::<_, i64>(6).unwrap_or(0) as u16,
                req_level: r.get::<_, i64>(7).unwrap_or(0) as u16,
                req_str: r.get::<_, i64>(8).unwrap_or(0) as u16,
                req_dex: r.get::<_, i64>(9).unwrap_or(0) as u16,
                req_int: r.get::<_, i64>(10).unwrap_or(0) as u16,
                armour: r.get::<_, i64>(11).unwrap_or(0) as u32,
                evasion: r.get::<_, i64>(12).unwrap_or(0) as u32,
                energy_shield: r.get::<_, i64>(13).unwrap_or(0) as u32,
                quality: r.get::<_, i64>(14).unwrap_or(0) as u16,
                identified: r.get::<_, i64>(15).unwrap_or(0) != 0,
                corrupted: r.get::<_, i64>(16).unwrap_or(0) != 0,
                lines: Box::new([]),
            });
        }
    }
    let mut per: Vec<Vec<Line>> = vec![Vec::new(); items.len()];
    {
        let mut st = db
            .prepare("SELECT item, arr, line, value FROM lines")
            .expect("prep");
        let mut rows = st.query([]).expect("q");
        while let Some(r) = rows.next().expect("row") {
            let i: i64 = r.get(0).unwrap_or(0);
            let a: String = r.get(1).unwrap_or_default();
            let k: String = r.get(2).unwrap_or_default();
            let v: f64 = r.get(3).unwrap_or(0.0);
            if let Some(slot) = per.get_mut(i as usize) {
                slot.push(Line {
                    arr: interner.intern(&a) as u8,
                    tmpl: interner.intern(&k),
                    stat: NONE,
                    value: v as f32,
                });
            }
        }
    }
    let mut nlines = 0;
    for (it, ls) in items.iter_mut().zip(per) {
        nlines += ls.len();
        it.lines = ls.into_boxed_slice();
    }
    let ms = t0.elapsed().as_millis();
    let resident = live().saturating_sub(before);
    let n = items.len();
    drop(items);
    drop(db);
    let _ = (n, nlines);
    (ms, n, resident)
}

fn run_sql(db: &rusqlite::Connection, q: &str) -> usize {
    let mut st = db.prepare(q).expect("prepare");
    let n = st.column_count();
    let mut rows = st.query([]).expect("query");
    let mut count = 0usize;
    while let Some(r) = rows.next().expect("row") {
        // touch every column so the row is really produced
        for i in 0..n {
            let _ = r.get_ref(i);
        }
        count += 1;
    }
    count
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

// ---------------------------------------------------------------- main

const REPS: usize = 7;

fn main() {
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    let track = here.parent().unwrap().to_path_buf();
    let corpus_path = track.join("raw/corpus.jsonl");
    let tvt = track
        .parent()
        .unwrap()
        .join("repoe/data/template-vs-translation.csv");
    let (sm, mapped, ambiguous) = stat_map(tvt.to_str().unwrap());
    eprintln!("stat map: {mapped} templates mapped, {ambiguous} ambiguous (first trade id wins)");

    let raw = std::fs::read_to_string(&corpus_path).expect("corpus.jsonl");
    let raw_bytes = raw.len();

    let mut results = String::new();
    let mut numbers: Vec<String> = Vec::new();

    for &copies in &[1usize, 10usize, 30usize] {
        let before = live();
        let (c, load_ms) = load(&raw, copies, &sm);
        let resident = live().saturating_sub(before);
        let n = c.items.len();
        let nlines: usize = c.items.iter().map(|i| i.lines.len()).sum();
        let unmapped: usize = c
            .items
            .iter()
            .flat_map(|i| i.lines.iter())
            .filter(|l| l.stat == NONE)
            .count();
        eprintln!(
            "scale x{copies}: {n} items, {nlines} lines ({unmapped} without a stat id), \
             load {load_ms} ms, resident {resident} bytes, {} templates, {} stat ids, {} skipped",
            c.tmpl.len(),
            c.stat.len(),
            c.skipped
        );
        numbers.push(format!(
            "x{copies},{n},{nlines},{unmapped},{load_ms},{resident},,{},{},{}",
            c.tmpl.len(),
            c.stat.len(),
            c.skipped
        ));

        for by_stat in [false, true] {
            let ident = if by_stat { "stat" } else { "template" };
            let k = ids(&c, by_stat, &sm);

            // ---- engine A: the scan
            for (qi, qname) in QUERIES.iter().enumerate() {
                let t = Instant::now();
                let rows = scan(&c, qi, &k, by_stat);
                let cold = t.elapsed().as_secs_f64() * 1000.0;
                let mut warm = Vec::new();
                for _ in 0..REPS {
                    let t = Instant::now();
                    let r2 = scan(&c, qi, &k, by_stat);
                    warm.push(t.elapsed().as_secs_f64() * 1000.0);
                    assert_eq!(r2, rows);
                }
                let _ = writeln!(
                    results,
                    "{qname},scan,{ident},x{copies},{n},{:.3},{:.3},{rows}",
                    cold,
                    median(warm)
                );
            }

            // ---- engine B: SQLite over the derived schema
            let path = track.join(format!("raw/bench-{ident}-x{copies}.db"));
            let p = path.to_str().unwrap().to_string();
            let (build_ms, bytes, rows_written) = build_db(&c, &p, by_stat);
            eprintln!("  sqlite {ident} x{copies}: build {build_ms} ms, {bytes} bytes, {rows_written} line rows");
            numbers.push(format!(
                "sqlite-{ident}-x{copies},{n},{rows_written},,{build_ms},,{bytes},,,"
            ));
            if !by_stat {
                let (pms, pn, pres) = load_from_projection(&p);
                eprintln!("  projection load x{copies}: {pms} ms, {pn} items, {pres} bytes resident");
                numbers.push(format!("projection-load-x{copies},{pn},{rows_written},,{pms},{pres},,,,"));
            }
            for (qi, qname) in QUERIES.iter().enumerate() {
                let q = sql(qi, &k, &c, by_stat);
                // cold: a fresh connection, so SQLite's own page cache is empty
                let db = rusqlite::Connection::open(&p).expect("open");
                let t = Instant::now();
                let rows = run_sql(&db, &q);
                let cold = t.elapsed().as_secs_f64() * 1000.0;
                let mut warm = Vec::new();
                for _ in 0..REPS {
                    let t = Instant::now();
                    let r2 = run_sql(&db, &q);
                    warm.push(t.elapsed().as_secs_f64() * 1000.0);
                    assert_eq!(r2, rows);
                }
                drop(db);
                let _ = writeln!(
                    results,
                    "{qname},sqlite,{ident},x{copies},{n},{:.3},{:.3},{rows}",
                    cold,
                    median(warm)
                );

                // the same predicate, counted rather than delivered: index work without row plumbing
                let cq = format!("SELECT count(*) FROM ({q})");
                let db = rusqlite::Connection::open(&p).expect("open");
                let t = Instant::now();
                let _ = run_sql(&db, &cq);
                let ccold = t.elapsed().as_secs_f64() * 1000.0;
                let mut cwarm = Vec::new();
                for _ in 0..REPS {
                    let t = Instant::now();
                    let _ = run_sql(&db, &cq);
                    cwarm.push(t.elapsed().as_secs_f64() * 1000.0);
                }
                drop(db);
                let _ = writeln!(
                    results,
                    "{qname},sqlite-count,{ident},x{copies},{n},{:.3},{:.3},{rows}",
                    ccold,
                    median(cwarm)
                );
            }
        }
    }

    let out = track.join("data/results.csv");
    let mut f = std::fs::File::create(&out).expect("results.csv");
    write!(
        f,
        "# generated by bench/src/main.rs (cargo run --release) from raw/corpus.jsonl \
         and ../repoe/data/template-vs-translation.csv\n\
         query,engine,identity,scale,items,cold_ms,warm_ms_median,rows\n{results}"
    )
    .expect("write");

    let out = track.join("data/numbers.csv");
    let mut f = std::fs::File::create(&out).expect("numbers.csv");
    write!(
        f,
        "# generated by bench/src/main.rs (cargo run --release) from raw/corpus.jsonl \
         ({raw_bytes} bytes) and ../repoe/data/template-vs-translation.csv\n\
         what,items,lines,lines_without_stat_id,load_or_build_ms,resident_bytes,file_bytes,templates,stat_ids,skipped\n"
    )
    .expect("write");
    for l in numbers {
        writeln!(f, "{l}").expect("write");
    }
}
