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

## The owner's searches, 2026-09-13, 08:09–09:26 US Central (13:09–14:26 UTC), Standard league, PC realm

Per query: the request body (`-request`), the search response (`-search`), the first `fetch` response (`-fetch`, the ten items the page fetched: carries seller accounts, whisper and hideout tokens — raw only, never `data/`). The owner: "In a couple cases I added additional search terms to make sure we saw results with the kinds of items you were looking for." `q2b` returned no results, so no fetch happened and its fetch file is empty. `q1-request` and `q1-search` were first saved empty and refilled at 09:21; `q3b` (the q3 stat with `max: 25`) was added at 09:25 to close Q3.

| File | Bytes | sha256 |
| --- | --- | --- |
| `raw/searches/q1-fetch.json` | 81608 | `f8ba1198e497527979cceb0e7dfa7f03030d5f5df8820d01a459777a4fbe9443` |
| `raw/searches/q1-request.json` | 361 | `576818b39446eb88eb0fc777cd1373cc385c70df0cd8f006c03bf1a7e84ef019` |
| `raw/searches/q1-search.json` | 7924 | `3698c25aee2651f83dee4d2e246303eb8c90bb8ea3c88077e7d59ae3fb2f8545` |
| `raw/searches/q2-fetch.json` | 96830 | `a308eecb21f4641c367fcef5249a43aa9950ac9810fabaea7fcd82cde7da7458` |
| `raw/searches/q2-request.json` | 253 | `e6791b26623da31f196d114ed5504caf75ec2d3fc14c67f6b343e560d5de83d5` |
| `raw/searches/q2-search.json` | 7882 | `bd534f723f0d41d6c5227d0ee14b71038d23bb48209ea5ad62ab1ac35acccd0c` |
| `raw/searches/q2b-fetch.json` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `raw/searches/q2b-request.json` | 327 | `b4b916edb4ebe65860f4ab0388a1591e8a8aa454658907be1e54927fbcf4847f` |
| `raw/searches/q2b-search.json` | 298 | `de7601320d3e1848c2dc8c755ac4cdb330137a687c70009596af95d78c17efaa` |
| `raw/searches/q3-fetch.json` | 122733 | `c13502e5b8c34e3949277a1c4c909b89d646f110dc4c6bd9718d143d1b88292e` |
| `raw/searches/q3-request.json` | 367 | `162b54961f3b24ab8dd2f18415f35afa4fe5cd5dd3680b5154cc696ef5ca026d` |
| `raw/searches/q3-search.json` | 7931 | `7eac69069e11f43ecd5b84e3c72468e9d11e5510588cffb7a85a1c0280eee8a2` |
| `raw/searches/q3b-fetch.json` | 124967 | `43663cbee9bea9b3f8729cdfa1fba21cf1566b60dfe2f8b92f62ec3d7d14dcf2` |
| `raw/searches/q3b-request.json` | 350 | `44ea51cfdff63cbe6601abbfe0bdbdc20b85df34405993a9991dc1db0ce45d81` |
| `raw/searches/q3b-search.json` | 7930 | `04408301a70e8869458ea06f67835ec8a068a911356b94ddd03f8b335b212664` |
| `raw/searches/q4-fetch.json` | 99257 | `a6fb680367d0491436641cf8064e1c6de94b3b05790fb8f6671d05eff6969665` |
| `raw/searches/q4-request.json` | 806 | `365cfd66a109b33a4b385966b363ada751e33ccc551085d8b9e30c12ef33508a` |
| `raw/searches/q4-search.json` | 8069 | `f87628d7a759679e170a329ffee956b59c05489b10a2ead25f71526783ac1e06` |
| `raw/searches/q5-fetch.json` | 103204 | `299c40ad8f7e1e3b9da9496826614fbf6737a721b8b3bc3735673ac0fe0bc9a8` |
| `raw/searches/q5-request.json` | 455 | `5382ce63c4b4ff517884a9ff7c0b43e634772cf80720f8cbab9c82b70112c190` |
| `raw/searches/q5-search.json` | 2708 | `b49a0e768d53905a745ac7397bed5aed7c0681df749dc555d72cfda646affbf8` |
| `raw/searches/q6-fetch.json` | 113332 | `6ae5cc69ed015a92ea696f2a04156328f7dbc33cbc58e839986852df638343e3` |
| `raw/searches/q6-request.json` | 474 | `0c1009e3808f7c8fd8d785b8198f473d8766d2457c3db1d60fd4cbf6a2d0a5dc` |
| `raw/searches/q6-search.json` | 7973 | `9c1ff9ef774d57b4b360a79b2ac4243de73a71519093b4efe221ff5d9c816607` |
