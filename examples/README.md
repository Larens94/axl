# AXL Examples — Esempi Funzionanti

Tutti gli esempi sono scritti in AXL e eseguiti dall'interpreter.

## Esempi Base

### hello.axl — Hello World
```bash
axl run examples/compact.axl
```

### test.axl — Test primitives
```bash
axl run examples/test.axl
```

## Esempi Completi

### backend/api.axl — Backend API
File I/O, JSON, hash, base64, timestamps:
```bash
axl run examples/backend/api.axl
```

### data/pipeline.axl — Data Pipeline
CSV parsing, list operations, math analysis:
```bash
axl run examples/data/pipeline.axl
```

### frontend/app.axl — Frontend UI
AX-UI component structure:
```bash
axl run examples/frontend/app.axl
```

### ai-platform/platform.axl — AI Platform
Text operations, math, crypto:
```bash
axl run examples/ai-platform/platform.axl
```

### agents/system.axl — Multi-Agent System
Agent definitions, workflows, execution:
```bash
axl run examples/agents/system.axl
```

## Esecuzione

```bash
# Build
cargo build --workspace

# Run any example
cargo run -p axl-cli -- run examples/backend/api.axl

# Or with axl binary
axl run examples/backend/api.axl
```
