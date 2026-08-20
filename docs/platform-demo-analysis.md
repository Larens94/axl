# Analisi demo applicative e piattaforme

> Stato dell'analisi: 20 agosto 2026. Questo documento distingue capacità implementate, prerequisiti e target dimostrabili. Non dichiara come disponibili backend o bridge ancora inesistenti.

## 1. Decisione

AXL non deve creare sei demo scollegate né delegare tutta la logica a wrapper host. Deve costruire una **vertical slice condivisa**:

```text
AXL Compact Source
→ frontend + tipi
→ AX-HIR applicativa
→ AX-MIR/effect IR
→ runtime Rust
→ capability ABI
   ├─ backend HTTP/database
   ├─ browser DOM/WASM
   ├─ desktop WebView/native host
   └─ iOS SwiftUI / Android Compose
```

Il primo prodotto dimostrativo sarà **AXL Syncboard**: una board collaborativa con account locali, progetti, task, aggiornamenti realtime, cache offline, audit e notifiche. La stessa logica di dominio e lo stesso contratto API alimentano tutti i client.

Due demo di supporto impediscono che il prodotto nasconda lacune:

1. **AXL UI Gallery** — componenti, token, layout adattivo, input, focus, accessibilità, temi e snapshot visuali;
2. **AXL Network Lab** — HTTP client/server, streaming, SSE, WebSocket, timeout, cancellazione, backpressure e reconnect.

## 2. Baseline reale

| Area | Disponibile oggi | Gap bloccante |
|---|---|---|
| Source | Compact Source 2, writer canonico, legacy pack | nuovi tipi e opcode/versione compatibile |
| Core | `string`, `integer`, `boolean`, `list<T>`, binding, flow, funzioni | bytes, float/decimal, option/result, map/record/enum |
| Moduli | import confinati e namespace | package, manifest, lockfile, export pubblico |
| Runtime | interprete Python sincrono e budgeted | async, task, stream, cancellazione, runtime Rust |
| IR | AX-IR JSON 1.0/1.1/1.2 | HIR/MIR, effect IR, ABI valori/risorse |
| Capability | tool host deny-by-default, policy/audit | capability tipizzate, resource handles, lifecycle async |
| Backend | nessuno | socket, HTTP, routing, DB, config, observability |
| UI | nessuna | UI IR, stato, eventi, layout, accessibilità, renderer |
| Packaging | wheel Python/CLI | WASM, exe/app bundle, APK/AAB, Xcode app |

Conclusione: oggi è possibile prototipare adapter host, ma non chiamarli ancora “app scritte in AXL”. Una demo supera il gate solo quando struttura, stato, eventi e logica applicativa sono rappresentati in AXL/HIR; l'host implementa esclusivamente ABI e integrazione piattaforma.

## 3. Prerequisiti del linguaggio

### P0 — Algebra valori general-purpose

Prima di UI e rete servono:

- `bytes`, `float` o decimal esplicito;
- `list<T>`, `map<K,V>`, tuple e record;
- enum, `option<T>`, `result<T,E>`;
- errori strutturati;
- ownership/lifetime dei resource handle;
- serializzazione deterministica JSON e bytes.

Senza collezioni non esistono request headers, liste di task, alberi UI o righe database tipizzate.

### P1 — Effetti asincroni

Il modello deve essere semantico, non copiato da Tokio, JavaScript o Swift:

- `future<T>` e `stream<T>`;
- task group strutturati;
- cancellation token e deadline;
- channel bounded;
- backpressure;
- `select` deterministico dove applicabile;
- cleanup garantito delle risorse;
- errori timeout/cancel distinti.

Tokio può implementare il primo backend Rust, ma non deve comparire nella grammatica o nell'ABI pubblica.

### P2 — HIR, MIR e Capability ABI

Contratto minimo:

```text
capability-id
abi-version
input/output type-id
resource handles
future/stream result
required effects
limits + deadline + cancellation
platform targets
stable error code
```

Le capability devono essere granulari: `net.client`, `net.listen`, `db.query`, `ui.window`, `ui.notify`; non un grant globale `network` o `system`.

## 4. Framework UI/UX AXL

### 4.1 Modello raccomandato

Creare **AX-UI**, un modello dichiarativo indipendente dal renderer:

```text
state + reducer/event
→ immutable semantic UI tree
→ keyed diff
→ renderer adapter
```

AX-UI possiede:

- component tree e identity stabile;
- state binding e one-way data flow;
- event dispatch;
- layout constraints/adaptive breakpoints;
- design token semantici;
- navigation e deep link;
- form/validation;
- animation timeline;
- accessibility semantics;
- focus, keyboard, pointer e touch;
- localization, RTL, dynamic type;
- test tree e snapshot semantici.

Non deve possedere DOM, CSS, SwiftUI, Compose, WinUI o GPU API. Questi sono renderer/bridge.

### 4.2 Renderer e ordine

| Target | Primo renderer | Natura reale | Decisione |
|---|---|---|---|
| Browser | DOM + CSS + Web APIs | web nativo | primo target UI |
| Windows/macOS | Tauri 2 + WebView di sistema | binario nativo, UI web | bootstrap desktop rapido |
| iOS | adapter SwiftUI/UIKit | app e controlli nativi | target mobile canonico |
| Android | adapter Jetpack Compose | app e UI native | target mobile canonico |
| Desktop avanzato | WinUI/AppKit/SwiftUI adapter | controlli nativi | dopo conformance AX-UI |
| Grafica/giochi | `wgpu`/WebGPU renderer | custom GPU UI | non per il primo framework app |

Tauri 2 dichiara supporto Windows, macOS, Linux, Android e iOS con logica Rust e frontend WebView. È adatto a validare packaging e bridge rapidamente, ma **non equivale a controlli UI nativi**. Per questo non sostituisce i renderer SwiftUI e Compose richiesti dal gate mobile.

Compose Multiplatform è production-ready per mobile e desktop, con web ancora indicato come beta. Può essere un adapter opzionale o acceleratore, non il fondamento semantico di AX-UI: renderebbe Kotlin/Compose una dipendenza architetturale di AXL.

### 4.3 Componenti V1

`App`, `Window`, `Screen`, `Nav`, `Stack`, `Grid`, `Scroll`, `Text`, `Image`, `Button`, `TextField`, `Toggle`, `List`, `Dialog`, `Progress`, `Canvas`.

Ogni componente espone proprietà semantiche e accessibilità. Stile libero tipo CSS, API SwiftUI o modifier Compose non entra nel source canonico; viene normalizzato in token/layout HIR.

## 5. Backend completo

### 5.1 Stack di bootstrap

- runtime Rust: Tokio;
- HTTP/routing: Hyper + Axum + Tower;
- TLS: rustls o terminazione reverse-proxy dichiarata;
- database: PostgreSQL in produzione, SQLite per locale/test;
- pool e query tipizzate dietro `db.*` ABI;
- migrazioni versionate;
- logging strutturato, metriche e trace OpenTelemetry;
- OpenAPI generata dal modello route/type;
- graceful shutdown, health/readiness e config typed;
- secrets risolti dall'host, mai nel source/IR/audit.

Queste librerie sono implementazioni sostituibili. AXL definisce semantica HTTP, request/response, route, middleware, transaction e stream.

### 5.2 Gate “backend completo”

Syncboard backend deve dimostrare:

- CRUD con PostgreSQL e migrazioni;
- transazioni e vincoli;
- auth session/OIDC-ready, password hashing solo tramite capability host;
- RBAC per workspace;
- REST JSON documentata OpenAPI;
- SSE per feed attività;
- WebSocket per collaborazione bidirezionale;
- upload bounded e content-type allowlist;
- pagination, idempotency key e rate limit;
- timeout, cancellation e graceful shutdown;
- log/metric/trace correlate;
- test unitari, integrazione DB e test protocollo end-to-end;
- container Linux riproducibile.

## 6. Networking moderno

### V1 obbligatorio

| Capacità | Motivo |
|---|---|
| DNS + TCP | fondazione portabile |
| TLS 1.3 e trust store host | trasporto sicuro |
| HTTP semantics | contratto comune a HTTP/1.1, 2 e 3 |
| HTTP/1.1 + HTTP/2 | interoperabilità server/client immediata |
| JSON UTF-8 | baseline API e debug |
| WebSocket | realtime bidirezionale diffuso |
| SSE | push server semplice, reconnect standard |
| streaming body | file/eventi senza buffering completo |
| timeout/deadline/cancel | sicurezza operativa |
| backpressure e limiti | evitare memoria/queue illimitate |
| proxy, CORS, cookie e redirect policy | funzionamento web reale |

### V2

- HTTP/3 su QUIC;
- Connect RPC e gRPC/Protobuf;
- CBOR per payload compatti;
- mTLS e service identity;
- retry/circuit breaker/load balancing controllati;
- WebTransport dopo un adapter sperimentale e con fallback.

### Non baseline

- MessagePack: utile ma non standard Internet; CBOR è uno standard IETF e copre lo stesso bisogno iniziale;
- raw UDP esposto alle app: solo capability specialistica;
- GraphQL integrato nella grammatica: libreria/bridge, non semantica core;
- protocollo realtime proprietario quando SSE/WebSocket bastano.

HTTP/3 è standard IETF; WebTransport al 20 agosto 2026 è ancora W3C Candidate Recommendation e la stessa specifica avverte che API/protocolli possono cambiare. Quindi va isolato dietro ABI e non usato come requisito della prima demo.

## 7. Demo canoniche

### D1 — Network Lab

**Obiettivo:** provare runtime async e capability ABI prima della UI.

- server `/health`, `/echo`, `/stream`, `/events`, `/socket`;
- client AXL concorrente;
- HTTP JSON, streaming, SSE e WebSocket;
- cancellazione, deadline, limiti body e reconnect;
- test contro reference host e runtime Rust.

**Gate:** nessuna route o state machine codificata nel wrapper Rust/Python; il wrapper registra soltanto capability.

### D2 — Syncboard Web

- backend D1 esteso con PostgreSQL;
- web app AX-UI → DOM;
- login locale di sviluppo;
- board CRUD e realtime;
- responsive/accessibile, keyboard e screen reader;
- PWA/offline cache;
- build WASM/browser quando il runtime è pronto.

**Gate:** Playwright end-to-end su due sessioni browser; un aggiornamento appare realtime e sopravvive al reload.

### D3 — Syncboard Desktop

- stesso dominio e AX-UI;
- Tauri 2 host Windows/macOS;
- file picker, secure storage, notifiche e deep link tramite capability;
- installer firmabile: MSIX/MSI e `.app`/DMG;
- aggiornamenti disattivati finché firma e supply chain non sono definite.

**Gate:** build e smoke su runner Windows e macOS. Linux non può certificare bundle, firma o comportamento OS di entrambi.

### D4 — Syncboard Mobile Native

- libreria runtime Rust con ABI stabile;
- host Xcode + SwiftUI su iOS;
- host Gradle + Jetpack Compose su Android;
- AX-UI semantic tree tradotto in componenti native;
- keychain/keystore, notifiche, lifecycle, background policy, deep link;
- offline SQLite e sync incrementale.

**Gate:** test su simulatore iOS e emulator Android, più almeno un device reale per piattaforma; `.ipa` richiede macOS/Xcode e firma Apple, AAB richiede Android SDK/Gradle e signing configurato.

### D5 — UI Gallery

- catalogo componenti e design token;
- light/dark, locale, RTL, dynamic text;
- keyboard/focus/touch;
- accessibility tree assertions;
- golden screenshots per renderer.

**Gate:** stesso corpus semantico su DOM, SwiftUI e Compose; differenze consentite solo se dichiarate nel profilo piattaforma.

## 8. Struttura repository target

```text
runtime/
  axl-core-rs/
  axl-vm/
  axl-component/
bridges/
  net-rust/
  db-sql/
  ui-dom/
  ui-tauri/
  ui-swiftui/
  ui-compose/
frameworks/
  ax-ui/
demos/
  network-lab/
  syncboard/shared/
  syncboard/backend/
  syncboard/web/
  syncboard/desktop/
  syncboard/ios/
  syncboard/android/
  ui-gallery/
conformance/
  source-ir-runtime/
  capability-abi/
  ax-ui/
```

## 9. Sequenza raccomandata

1. **Core types:** collezioni, record, option/result, bytes, errori.
2. **Async semantics:** future/stream/task/cancel/backpressure.
3. **HIR/MIR + ABI:** resource handle ed effetti tipizzati.
4. **Rust tracer:** stesso corpus Python/Rust.
5. **Network Lab:** HTTP/1.1-2, JSON, SSE, WebSocket.
6. **Backend Syncboard:** DB, auth, OpenAPI, observability.
7. **AX-UI + DOM:** UI Gallery e Syncboard Web.
8. **Tauri desktop:** Windows/macOS packaging e OS bridge.
9. **SwiftUI/Compose:** mobile realmente nativo.
10. **WASM Component/WASI e HTTP/3:** conformance e ottimizzazione.
11. **WebTransport/gRPC/Connect/GPU:** adapter successivi, guidati da casi reali.

Non iniziare contemporaneamente dai cinque target: moltiplicherebbe adapter instabili prima di fissare valori, async, ABI e UI semantics.

## 10. Criteri trasversali

Ogni demo deve avere:

- sorgente AXL canonico come logica primaria;
- build riproducibile e comando unico;
- zero credenziali nel repository;
- capability deny-by-default;
- test reference/Rust equivalenti;
- test end-to-end sul target reale;
- limiti input/output/connessioni/queue;
- cancellazione e cleanup verificati;
- accessibilità e keyboard/touch dove applicabili;
- SBOM, lockfile e artifact hash;
- documentazione che distingue “native binary”, “native UI” e “WebView”.

## 11. Scelta finale

La strategia consigliata è ibrida e progressiva:

- **Rust** sotto il cofano per runtime, async, networking e bridge;
- **WASM/Component Model** per portabilità e sandbox, senza dipendere integralmente da WASI;
- **AX-UI semantic IR** come framework UI/UX del linguaggio;
- **DOM** primo renderer;
- **Tauri 2** bootstrap desktop;
- **SwiftUI e Jetpack Compose** renderer mobile nativi;
- **wgpu/WebGPU** soltanto per canvas, grafica e futuro renderer custom;
- **HTTP + SSE + WebSocket** baseline; HTTP/3, Connect/gRPC, CBOR e WebTransport dopo.

Questa architettura produce demo vere presto senza trasformare una scelta temporanea — WebView, Kotlin, Swift, Tokio o Axum — nel linguaggio AXL.

## Fonti principali

- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110)
- [RFC 9114 — HTTP/3](https://www.rfc-editor.org/rfc/rfc9114)
- [RFC 6455 — WebSocket](https://www.rfc-editor.org/rfc/rfc6455)
- [WHATWG — Server-sent events](https://html.spec.whatwg.org/multipage/server-sent-events.html)
- [W3C — WebTransport](https://www.w3.org/TR/webtransport/)
- [WASI e WebAssembly Component Model](https://wasi.dev/)
- [Tauri 2 e architettura](https://v2.tauri.app/concept/architecture/)
- [Apple — SwiftUI](https://developer.apple.com/documentation/swiftui)
- [Android — Jetpack Compose](https://developer.android.com/compose)
- [Microsoft — WinUI 3](https://learn.microsoft.com/en-us/windows/apps/winui/winui3/)
- [Compose Multiplatform](https://kotlinlang.org/compose-multiplatform/)
- [Tokio](https://tokio.rs/) e [Axum](https://docs.rs/axum/latest/axum/)
- [OpenAPI 3.2](https://spec.openapis.org/oas/latest.html)
- [OpenTelemetry](https://opentelemetry.io/docs/specs/otel/)
- [RFC 8949 — CBOR](https://www.rfc-editor.org/rfc/rfc8949)
