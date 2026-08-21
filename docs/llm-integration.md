# Integrazione LLM

AXL 3.0 ha il reasoning LLM come primitiva nativa, non come tool esterno.

## Backend Trait

```rust
pub trait LlmBackend: Send + Sync {
    fn generate(&self, system: &str, messages: &[(String, String)]) -> Result<String, LlmError>;
    fn generate_stream(&self, system: &str, messages: &[(String, String)], on_chunk: &dyn Fn(&str) -> Result<(), LlmError>) -> Result<String, LlmError>;
    fn embed(&self, text: &str) -> Result<Vec<i64>, LlmError>;
    fn generate_json(&self, instruction: &str, input: &str, schema: &serde_json::Value) -> Result<serde_json::Value, LlmError>;
}
```

## Primitive LLM

### Reasoning

```axl
# Chain-of-thought
!reason/2 "step by step" problem
```

### Classification

```axl
# Classifica testo
!classify/3 "instruction" text "label1,label2,label3"
```

### Extraction

```axl
# Estrai entità
!extract/2 "person, org, location" text
```

### Generation

```axl
# Genera testo
!generate/2 "system prompt" messages
```

### Embedding

```axl
# Genera embedding
!embed/1 "text to vector"
```

## MockBackend (Testing)

```rust
let backend = MockBackend::new(vec!["response".into()]);
let result = llm::reason(&backend, "instruction", "input").unwrap();
```

## Implementazione Reale

Per usare un LLM reale, implementare `LlmBackend`:

```rust
struct OpenAiBackend { api_key: String }

impl LlmBackend for OpenAiBackend {
    fn generate(&self, system: &str, messages: &[(String, String)]) -> Result<String, LlmError> {
        // Chiamata API OpenAI
    }
    fn embed(&self, text: &str) -> Result<Vec<i64>, LlmError> {
        // OpenAI embeddings API
    }
    // ...
}
```

## Esempio Completo

```axl
2;
10|query|"what is AXL?",!reason/2,"explain step by step",$query|s;
12|$query;
10|category|text,!classify/3,"categorize",text,"tutorial,news,opinion"|s;
12|$category
```
