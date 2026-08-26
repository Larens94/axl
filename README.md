# AX / AXL — Agent eXecution

**AXL** è il linguaggio sorgente principale per costruire applicazioni complete.
Rust, React/TypeScript, SQL, WebAssembly e i bridge hardware sono target o dettagli
di runtime: lo sviluppatore descrive sistema, dati, API, UI e agenti in AXL.

**Sito:** [larens94.github.io/axl](https://larens94.github.io/axl/) · **Docs:** [`docs/`](docs/) · **Specifica:** [`SPEC.md`](SPEC.md)

```text
AXL Source
→ parser e analisi statica
→ AX-IR tipizzata
→ capability e target check
  ├─ Rust              backend, CLI, sistemi e IoT
  ├─ React/TypeScript  frontend web
  ├─ SQL               schema e migrazioni
  ├─ WebAssembly       browser ed edge (roadmap)
  └─ AI/host bridges   modelli, tool e hardware
```

## Applicazione full-stack

```axl
entity Customer {
  field name: String
  field email: String
}

api Customer {
  GET /api/customers -> list
  POST /api/customers -> create
}

ui Customers {
  table: Customer[name, email]
}
```

La build produce backend Rust, frontend React/TypeScript e migrazioni SQL.

Il formato Compact Source resta disponibile come rappresentazione canonica a
basso livello per runtime, trasporto e generazione automatica. Nei file `.axl`
viene formattato su più righe; spazi, rientri e newline non cambiano la semantica.

```axl
2;
50|worker;
  10|x|#1|i;
  30|$x,#0,>;
    12|$x;
  99;
99;
```

## Primitiva Native (90+)

```axl
# I/O
!file_read/1       !file_write/2      !file_exists/1

# Text
!text_upper/1      !text_split/2      !text_find/2

# Collections
!list_push/2       !list_sort/1       !map_get/2

# Math
!math_add/2        !math_mul/2        !math_random/0

# Crypto
!hash_sha256/1     !encode_base64/1   !crypto_random_bytes/1

# JSON
!json_parse/1      !json_stringify/1

# Network
!http_get/1        !http_post/2

# System
!env_get/1         !time_now/0        !path_join/1

# LLM
!reason/2          !classify/3        !extract/2
```

## Avvio Rapido

```bash
cargo build --workspace
cargo test --workspace
cargo run -p axl-cli -- build examples/crm/crm.axl -o build/crm
```

Dopo l'installazione del binario:

```bash
axl build examples/crm/crm.axl -o build/crm
axl check examples/crm/crm.axl
axl fmt program.axl
axl fmt program.axl --width 80 --check
```

## Workspace

```text
runtime/
├── axl-core-rs/    # IR, runtime, primitive e AX-UI
├── axl-cli/        # CLI `axl`
└── axl-compiler/   # Analisi e target Rust/React/SQL
```

## Esempi

- **CRM** — definizione AXL di entità, API e interfaccia, compilabile nei target applicativi

## Licenza

Apache-2.0
