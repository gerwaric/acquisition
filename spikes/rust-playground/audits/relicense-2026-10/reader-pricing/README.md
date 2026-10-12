# Reader: pricing (currency, buyout, shop)

- **currency → currency.rs / currency-v1.toml: clean at HEAD (FACT + IDEA); dirty in history.** Draft commit `51a33751` copied all 19 C++ currencies in enum order with all 52 C++ alias strings; `10ef2374` replaced it the next day.
- **buyout / buyoutmanager → price.rs, game_side.rs, listing.rs: clean (FACT + IDEA).** Same prefixes (the game's own), a different kind set, a different parser, and a different inheritance design.
- **shop → shop.rs: clean in substance (FACT + IDEA), with two de-minimis EXPRESSION carry-overs:** the leading space in ` ~price …` spoiler titles and the `[items]` template token. Both were kept on purpose for compatibility and tested by the owner on the site.
- Paths: C++ is at the repo root; Rust is under `spikes/rust-playground/crates/acquisition-plan/`.

## 1. Currency

**The table the commit message names.** `51a33751` says "19 rows carried over from the C++ tables". That table is `reference/currency-v1.toml`.

- **The draft at `51a33751` (history only).** It holds exactly the 19 currencies of `src/currency.cpp:31-52`, in C++ enum order (alt, fuse, alch, chaos, gcp, exa, chrom, jew, chance, chisel, scour, blessed, regret, regal, divine, vaal, coin, mirror, silver). It uses the C++ display names (`:8-29`) as `display`. Its `aliases` are the 52 keys of `m_string_to_currency_type` (`:77-130`). I checked them with `comm` and found no differences: `{"coin" "coins" "p" "perandus" "shekel"}`, `{"jew" "jewel" "jewels" "jeweler" "jewelers"}` and the rest. The draft's own header says "the C++ tables stand as the only cited source … aliases = `Currency::FromString`".
  - That is the C++ author's selection and ordering, reproduced whole. I classed it EXPRESSION, medium. It is medium rather than high because each element is a game name or player shorthand, and a short word list's selection is thin expression.
- **HEAD (`10ef2374`, then owner review `d3a56507`).** The table has 42 rows.
  - 39 rows are the words the in-game price dialog wrote into notes. Each has a `game:` evidence line and a display name the owner read in the dialog (`currency-v1.toml:59-512`). They are in the dialog's order.
  - 3 are retired rows: `chisel`, `coin`, `silver` (`:513-547`). They exist only because C++ had them, and they keep the C++ display names. Those names are the game's item names.
  - All 19 C++ currencies appear. 16 of them are independently re-sourced from the game. 23 rows are new. Order is not preserved.
  - Four C++ tags (`exa chrom fuse jew`) survive as aliases cited `cpp:`. This is interop for 0.18 data.
  - The owner added the shorthand `divines alchs alts mir mirr mirrors`, cited `browser:`. Of these, `divines alchs alts mir` are also C++ keys, which is unsurprising for common player shorthand.
  - 44 of the 52 C++ aliases are gone.
  - I classed HEAD as FACT (the active rows) and IDEA (the retired rows and the legacy aliases).
- **No counterparts.** The C++ rank table (`currency.cpp:54-75`) has no counterpart in Rust. The store crate (`acquisition-store/src/lib.rs`) holds no currency table: its only hits are test fixtures.
- **The loader.** `src/currency.rs` is a strict TOML loader with one `resolve`. It bears no structural resemblance to the C++ `std::map` getters with their warn-and-return-error pattern.

## 2. Buyout

- **Prefixes.** C++ `m_buyout_type_as_prefix` (`buyout.cpp:15-20`) holds `" ~b/o "` and `" ~price "`. Rust `game_side.rs:161-163` holds `~price`, `~b/o` and `~skip`. These are the game's note grammar (T10, from an observed corpus): FACT.
- **Kinds.** C++ has `IGNORE, BUYOUT, FIXED, CURRENT_OFFER, NO_PRICE, INHERIT` (`buyout.h:19-26`). Rust has `exact | negotiable | no_price | skip` (`price.rs:10-22`). Rust has no `c/o`, and inheritance is not a type. The names follow the game's meanings: IDEA, medium.
- **Parser.** C++ `StringToBuyout` (`buyoutmanager.cpp:425-443`) is one regex, `(~\S+)\s+(\d+\.?\d*)\s+(\w+)`, plus a 4-entry map. Rust `read` (`game_side.rs:168-247`) works differently:
  - it trims the end and requires a leading `~`;
  - it splits on the first space and then on the next one;
  - it applies a per-source rule (a ratio is invalid in a tab name);
  - it uses the four-place `Amount` grammar;
  - it returns invalid outcomes with reasons, and tolerates text after the word.

  It names `~c/o` and `~gb/o` only to reject them as "the C++ app's, never observed" (`game_side.rs:43-45`). That is knowledge of behaviour, not code. IDEA, low.
- **Inheritance.** C++ has an `inherited` flag (`buyout.h:53,65`). Rust has the C70 chain item → substash → tab → folder, which stops on an unreadable row (`listing.rs:78-87`). IDEA, low.
- **Not posting game prices.** C++ `IsPostable` is `source != GAME && …` (`buyout.cpp:50-53`). Rust omits game-decided items as `Cell::GameLists` (`shop.rs:594-596`), for a stated reason of its own (C81: the site already lists it). IDEA, medium.

## 3. Shop

- **FACT.**
  - The 50,000 page size: C++ `kMaxCharactersInPost` (`shop.cpp:34`), Rust `DEFAULT_PAGE_SIZE` (`shop.rs:145-147`). The owner confirmed it as the forum's hard limit (T23).
  - The price-as-spoiler-title shape `[spoiler="…"]…[/spoiler]` (`shop.cpp:214-219`, `shop.rs:942-947`). It is forum BBCode, and T22 confirms the site reads the title as the price.
- **EXPRESSION, low: the leading space.** C++ `SpoilerBuyout` writes `"[spoiler=\"" + " ~b/o " + value + " " + tag + "\"]"`, and a no-price row gets an empty title. Rust `shop.rs:602` writes `" ~price"` / `" ~b/o"` plus ` <amount> <emit>`, and `[spoiler=""]` for no price. Its doc calls this "the C++ app's form (T15) with its leading space" (`shop.rs:78-84`). The space is an arbitrary byte that came from the C++ prefix map. It is one character, though, and the owner validated it on the live site (T21, T22).
- **EXPRESSION, low: the `[items]` token.** C++ `kShopTemplateItems = "[items]"` (`shop.cpp:33`) is substituted per page (`:489-491`). Rust `ITEMS_TOKEN = "[items]"` (`shop.rs:151`) must appear exactly once. It is the C++ author's arbitrary name, kept so existing templates still work.
- **IDEA, medium: grouping and cutting.**
  - **Sort key.** C++ `buyoutLess` stable-sorts by (type enum, currency enum, value) (`shop.cpp:49-62`), so b/o comes before price, then c/o, then no price. Rust's `group: (u8, String, Option<Amount>)` (`shop.rs:529-531`) sorts price before b/o before no price, with currency by tag string.
  - **Cutting.** C++ appends strings, and on overflow writes `[/spoiler]`, pushes the page and reopens `SpoilerBuyout` (`:466-480`). Rust `cut_pages` (`shop.rs:850-940`) does more:
    - it partitions entries too large to fit any page;
    - it reserves the widest page title;
    - it builds `PageBuild`/`GroupBuild` structures, then renders each page as a titled spoiler `Shop Post n of N (k items)`.
  - **Not carried.** C++ `kSpoilerOverhead = 19`, the MD5 page hash, and the `index + 1` stash number are not in Rust; Rust ranks tabs as the website does, per T24.
- **Not ported.** Fetching, posting and editing on the forum, and the session and rate-limit regexes (`shop.cpp:67-420, 497-803`), have no counterpart (NONE).
