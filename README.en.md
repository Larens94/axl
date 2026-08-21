**Languages:** [Italiano](README.md) · [English](README.en.md)

# AX / AXL — Agent eXecution

**AXL** is a programming language for agents that wraps Rust and everything Rust can do.

**Website:** [larens94.github.io/axl](https://larens94.github.io/axl/) · **Docs:** [`docs/`](docs/) · **Specification:** [`SPEC.md`](SPEC.md)

```text
AXL Compact Source 3.0
→ deterministic parser
→ AX-IR 2.0 (agent-centric)
→ type-check
→ interpreter
  ├─ 90+ native primitives (Rust)
  ├─ LLM backend (reason, classify, extract)
  ├─ tool system (policy, approval, audit)
  ├─ memory (in-memory, SQLite)
  └─ web server (HTTP, static, API)
```

## Example

```axl
2;10|result|"hello world",!text_upper/1|s;12|$result
```

→ `"HELLO WORLD"`

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
cargo run -p axl-cli -- run examples/compact.axl
cargo run -p netflix-server
cargo run -p ai-platform
```

## Workspace

```text
runtime/
├── axl-core-rs/    # Core library (3500 LOC)
├── axl-cli/        # CLI binary
├── netflix-server/ # Netflix demo
└── ai-platform/    # AI Platform demo
```

## Examples

- **Netflix** — Streaming platform with 20 titles
- **AI Platform** — Content analysis with 6 LLM APIs
- **API Server** — REST CRUD with auth and caching
- **Data Pipeline** — ETL with parsing and transformation
- **Multi-Agent** — Collaborative agents with reasoning

## License

Apache-2.0
