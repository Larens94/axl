/// LLM Backend trait — implementato per ogni provider (OpenAI, Anthropic, ecc.)
pub trait LlmBackend: Send + Sync {
    fn generate(&self, system: &str, messages: &[(String, String)]) -> Result<String, LlmError>;
    fn generate_stream(
        &self, system: &str, messages: &[(String, String)],
        on_chunk: &dyn Fn(&str) -> Result<(), LlmError>,
    ) -> Result<String, LlmError>;
    fn embed(&self, text: &str) -> Result<Vec<i64>, LlmError>;
    fn generate_json(
        &self, instruction: &str, input: &str, schema: &serde_json::Value,
    ) -> Result<serde_json::Value, LlmError>;
}

#[derive(Debug)]
pub enum LlmError {
    ProviderError(String),
    RateLimited,
    ContextTooLong,
    InvalidResponse,
    NetworkError(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::ProviderError(msg) => write!(f, "LLM provider error: {msg}"),
            LlmError::RateLimited => write!(f, "rate limited by LLM provider"),
            LlmError::ContextTooLong => write!(f, "context too long for LLM"),
            LlmError::InvalidResponse => write!(f, "invalid LLM response"),
            LlmError::NetworkError(msg) => write!(f, "network error: {msg}"),
        }
    }
}

impl std::error::Error for LlmError {}

pub struct MockBackend {
    responses: Vec<String>,
    index: std::sync::atomic::AtomicUsize,
}

impl MockBackend {
    pub fn new(responses: Vec<String>) -> Self {
        Self { responses, index: std::sync::atomic::AtomicUsize::new(0) }
    }
    pub fn with_default() -> Self {
        Self::new(vec!["mock response".into()])
    }
}

impl LlmBackend for MockBackend {
    fn generate(&self, _system: &str, _messages: &[(String, String)]) -> Result<String, LlmError> {
        let idx = self.index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(self.responses[idx % self.responses.len()].clone())
    }

    fn generate_stream(
        &self, system: &str, messages: &[(String, String)],
        on_chunk: &dyn Fn(&str) -> Result<(), LlmError>,
    ) -> Result<String, LlmError> {
        let full = self.generate(system, messages)?;
        for word in full.split_whitespace() {
            on_chunk(word)?;
            on_chunk(" ")?;
        }
        Ok(full)
    }

    fn embed(&self, text: &str) -> Result<Vec<i64>, LlmError> {
        let mut embedding = Vec::with_capacity(128);
        for i in 0..128i64 {
            embedding.push(((text.len() as f64 * (i as f64 + 1.0)).sin() * 1000.0) as i64);
        }
        Ok(embedding)
    }

    fn generate_json(
        &self, _instruction: &str, _input: &str, _schema: &serde_json::Value,
    ) -> Result<serde_json::Value, LlmError> {
        Ok(serde_json::json!({"mock": true}))
    }
}

// ============================================================================
// Reasoning primitives
// ============================================================================

pub fn reason(backend: &dyn LlmBackend, instruction: &str, input: &str) -> Result<String, LlmError> {
    let system = format!(
        "You are a careful reasoning assistant. {instruction}\n\n\
         Think step by step. Show your reasoning. Then give a final answer."
    );
    let messages = vec![("user".to_string(), input.to_string())];
    backend.generate(&system, &messages)
}

pub fn classify(backend: &dyn LlmBackend, instruction: &str, input: &str, labels: &[String]) -> Result<String, LlmError> {
    let labels_str = labels.join(", ");
    let system = format!(
        "{instruction}\n\n\
         Classify into exactly one: [{labels_str}]\n\
         Reply with ONLY the category name."
    );
    let messages = vec![("user".to_string(), input.to_string())];
    backend.generate(&system, &messages)
}

pub fn extract(backend: &dyn LlmBackend, schema: &str, input: &str) -> Result<Vec<String>, LlmError> {
    let system = format!(
        "Extract {schema} from the text.\n\
         Return each item on a separate line."
    );
    let messages = vec![("user".to_string(), input.to_string())];
    let result = backend.generate(&system, &messages)?;
    Ok(result.lines().map(String::from).filter(|l| !l.is_empty()).collect())
}

pub fn generate(backend: &dyn LlmBackend, system: &str, messages: &[(String, String)]) -> Result<String, LlmError> {
    backend.generate(system, messages)
}

pub fn generate_json(backend: &dyn LlmBackend, instruction: &str, input: &str, schema: &serde_json::Value) -> Result<serde_json::Value, LlmError> {
    backend.generate_json(instruction, input, schema)
}

pub fn embed(backend: &dyn LlmBackend, text: &str) -> Result<Vec<i64>, LlmError> {
    backend.embed(text)
}

pub fn similarity(a: &[i64], b: &[i64]) -> f64 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}
