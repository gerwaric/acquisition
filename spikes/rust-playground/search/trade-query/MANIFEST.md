# Evidence manifest — trade-query

Every file this track read, with what it is and how it was obtained (index rule 4). `data/` files are committed; `raw/` files are local only and described here. Captured by the owner in a browser on 2026-09-12, 21:38–21:40 US Central (2026-09-13 02:38–02:40 UTC), signed in, PoE 1 site, Standard league selected (the saved-from URL is `/trade/search/Standard/<search id>`). Access method: `browser` (SURFACES.md, the trade-site row; C79).

| File | Bin | Bytes | Source | sha256 |
| --- | --- | --- | --- | --- |
| `data/stats-2026-09-12.json` | data | 2099466 | `https://www.pathofexile.com/api/trade/data/stats` | `b983f1ddf4d999a15db5ad4dbd5eb30d02bf0f1b4a8e23b4fb251f726af3063d` |
| `data/items-2026-09-12.json` | data | 344615 | `https://www.pathofexile.com/api/trade/data/items` | `89b722ada424665e5d714cc431c504b00a936d2d19df5f21b6ba24109ac74ead` |
| `data/static-2026-09-12.json` | data | 199012 | `https://www.pathofexile.com/api/trade/data/static` | `57c3c33751d95a1180d3d145b315dd1068fc5087e977bf69e4c700d520e7a382` |
| `data/filters-2026-09-12.json` | data | 17234 | `https://www.pathofexile.com/api/trade/data/filters` | `5f30047668bb1c9def5972c7160aed4cdefb09a68ba83a8717b07d0246962de1` |
| `raw/Trade - Path of Exile.html` | raw | 686189 | the search page, "Webpage, Complete" | `1471773eb45d746c9722cae00c9d5e5e009c72d481c94ea9eacb2366115fb846` |
| `raw/Trade - Path of Exile_files/config.js` | raw | 4360 | `https://web.poecdn.com/dist/legacy/config.js` (page bundle) | `e29a0e016da83dcdb1798334ff338bba00c337edd27adf080e2f13bfa10a0c78` |
| `raw/Trade - Path of Exile_files/main.a1ed07be8ff39876254ffefe1642d35d8b074db1.js` | raw | 548797 | `https://web.poecdn.com/dist/legacy/main.a1ed07be8ff39876254ffefe1642d35d8b074db1.js` (page bundle) | `c63575ba6a49f8c8164d063b61106d2aff865d9cce4ca1bce735d77d812f8af5` |
| `raw/Trade - Path of Exile_files/plugins.2e640ff226bcccd708839df10c93faa915bc8de9.js` | raw | 1026834 | `https://web.poecdn.com/dist/legacy/plugins.2e640ff226bcccd708839df10c93faa915bc8de9.js` (page bundle) | `70127029229cd9873190b47b5286a81b1fefea155b537adfb6f705c53c8c32d7` |
| `raw/Trade - Path of Exile_files/require-2.3.2.js` | raw | 17824 | `https://web.poecdn.com/dist/legacy/require-2.3.2.js` (page bundle) | `e3b7faebc9c83d40bb8c017a5242ed65e110054245f928a36e410c1d716a4b54` |
| `raw/Trade - Path of Exile_files/trade.d17c272e9c6a43635ff3d9779e4f14924da8fcf5.js` | raw | 161153 | `https://web.poecdn.com/dist/legacy/trade.d17c272e9c6a43635ff3d9779e4f14924da8fcf5.js` (page bundle) | `513aded86661e7f5aa842928c42ab63d4703fa2922d9b6fa17062a503deb29e3` |

Still to capture (the README, "Inputs"): one search request body and its response, and one `fetch` response for a rare item.
