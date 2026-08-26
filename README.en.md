**Languages:** [Italiano](README.md) · [English](README.en.md)

# AX / AXL — Agent eXecution

**AXL** is the primary source language for building complete applications. Rust,
React/TypeScript, SQL, WebAssembly, and hardware bridges are compilation targets
or runtime details: system, data, API, UI, and agent semantics stay in AXL.

**Website:** [larens94.github.io/axl](https://larens94.github.io/axl/) · **Presentation:** [AXL Claude × Apple](https://larens94.github.io/axl/presentation.html) · **Docs:** [`docs/`](docs/) · **Specification:** [`SPEC.md`](SPEC.md)

```text
AXL Source
→ parser and static analysis
→ typed semantic model
  ├─ Rust/Axum/SeaORM  backend
  ├─ React/Refine/MUI  frontend
  ├─ SQL               schema and migrations
  └─ runtime/bridges   agents, tools, AI, and devices
```

## Full-stack application

```axl
entity Customer {
  field name: String
  field email: String
}

api Customer {
  GET /api/customers → list
  POST /api/customers → create
}
```

The UI uses compact numeric frames, formatted across lines for reviewable source:

```axl
3;60|2;
  61|1000|64;
    62|1|"customers"; 62|3|#25; 62|5|"cards";
    61|1001|65; 62|1|"name"; 62|2|"Name"; 62|4|#1; 99;
    61|1002|65; 62|1|"email"; 62|2|"Email"; 62|4|#2; 99;
  99;
99;
```

## Native Primitives (90+)

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

## Quick Start

```bash
cargo build --workspace
cargo test --workspace
cargo run -p axl-cli -- build examples/crm/crm.axl -o build/crm
```

## Workspace

```text
runtime/
├── axl-core-rs/    # IR, runtime, primitives, and AX-UI
├── axl-cli/        # `axl` CLI
└── axl-compiler/   # Rust/React/SQL application targets
```

## Examples

- **CRM** — 6 entities, 30 CRUD operations, 7 compact views, and Rust/React/SQL targets

AXL `0.1.0-alpha.1` is a working proof of concept: the demo is complete, while
language compatibility, security, and production deployment remain under active development.

## License

Apache-2.0
