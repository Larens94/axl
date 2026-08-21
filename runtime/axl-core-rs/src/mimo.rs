use crate::ir::Value;
use crate::llm::{LlmBackend, LlmError};

pub struct MiMoBackend {
    api_key: String,
    base_url: String,
    model: String,
}

impl MiMoBackend {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.xiaomimimo.com/v1".to_string(),
            model: "mimo-v2.5-pro".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    fn call_api(&self, endpoint: &str, body: &serde_json::Value) -> Result<serde_json::Value, LlmError> {
        let url = format!("{}{endpoint}", self.base_url);
        let body_str = body.to_string();

        let response = ureq::post(&url)
            .set("api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_string(&body_str)
            .map_err(|e| LlmError::NetworkError(format!("{e}")))?;

        let mut response_body = String::new();
        response.into_reader().read_to_string(&mut response_body)
            .map_err(|e| LlmError::NetworkError(format!("read: {e}")))?;

        serde_json::from_str(&response_body).map_err(|e| LlmError::ProviderError(format!("JSON parse: {e}")))
    }
}

impl LlmBackend for MiMoBackend {
    fn generate(&self, system: &str, messages: &[(String, String)]) -> Result<String, LlmError> {
        let mut msg_list = vec![serde_json::json!({"role": "system", "content": system})];
        for (role, content) in messages {
            msg_list.push(serde_json::json!({"role": role, "content": content}));
        }

        let body = serde_json::json!({
            "model": self.model,
            "messages": msg_list,
            "temperature": 0.7,
            "max_tokens": 4096,
        });

        let parsed = self.call_api("/chat/completions", &body)?;
        parsed["choices"][0]["message"]["content"]
            .as_str()
            .map(String::from)
            .ok_or(LlmError::InvalidResponse)
    }

    fn generate_stream(
        &self,
        system: &str,
        messages: &[(String, String)],
        on_chunk: &dyn Fn(&str) -> Result<(), LlmError>,
    ) -> Result<String, LlmError> {
        let result = self.generate(system, messages)?;
        on_chunk(&result)?;
        Ok(result)
    }

    fn embed(&self, text: &str) -> Result<Vec<i64>, LlmError> {
        let body = serde_json::json!({
            "model": self.model,
            "input": text,
        });

        let parsed = self.call_api("/embeddings", &body)?;
        let embedding = parsed["data"][0]["embedding"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| (f * 1000.0) as i64)).collect())
            .unwrap_or_default();

        Ok(embedding)
    }

    fn generate_json(
        &self,
        instruction: &str,
        input: &str,
        schema: &serde_json::Value,
    ) -> Result<serde_json::Value, LlmError> {
        let system = format!(
            "You must respond with valid JSON matching this schema: {schema}\n\n{instruction}"
        );
        let result = self.generate(&system, &[("user".into(), input.to_string())])?;
        serde_json::from_str(&result).map_err(|e| LlmError::ProviderError(format!("JSON parse: {e}")))
    }
}
