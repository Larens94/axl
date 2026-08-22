use crate::ir::Value;
use crate::mimo::MiMoBackend;
use crate::llm::LlmBackend;

const MIMO_API_KEY: &str = "sk-ejmpfhhrc5eyh9n1bwp2yn0dt1vtghqclesto54fnju5my9c";

pub fn llm_generate(args: &[Value]) -> Result<Value, String> {
    let system = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let user_msg = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    
    let backend = MiMoBackend::new(MIMO_API_KEY.to_string());
    let messages = vec![("user".to_string(), user_msg.to_string())];
    
    match backend.generate(system, &messages) {
        Ok(result) => Ok(Value::String(result)),
        Err(e) => Err(format!("llm_generate: {e}")),
    }
}

pub fn llm_reason(args: &[Value]) -> Result<Value, String> {
    let instruction = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let input = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    
    let backend = MiMoBackend::new(MIMO_API_KEY.to_string());
    let system = format!("You are a careful reasoning assistant. {instruction}\n\nThink step by step. Show your reasoning. Then give a final answer.");
    let messages = vec![("user".to_string(), input.to_string())];
    
    match backend.generate(&system, &messages) {
        Ok(result) => Ok(Value::String(result)),
        Err(e) => Err(format!("llm_reason: {e}")),
    }
}

pub fn llm_classify(args: &[Value]) -> Result<Value, String> {
    let instruction = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let input = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let labels = args.get(2).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    
    let backend = MiMoBackend::new(MIMO_API_KEY.to_string());
    let system = format!("{instruction}\n\nClassify into exactly one: [{labels}]\nReply with ONLY the category name.");
    let messages = vec![("user".to_string(), input.to_string())];
    
    match backend.generate(&system, &messages) {
        Ok(result) => Ok(Value::String(result.trim().to_string())),
        Err(e) => Err(format!("llm_classify: {e}")),
    }
}

pub fn llm_extract(args: &[Value]) -> Result<Value, String> {
    let schema = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let input = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    
    let backend = MiMoBackend::new(MIMO_API_KEY.to_string());
    let system = format!("Extract {schema} from the text. One per line.");
    let messages = vec![("user".to_string(), input.to_string())];
    
    match backend.generate(&system, &messages) {
        Ok(result) => {
            let items: Vec<Value> = result.lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| Value::String(l.to_string()))
                .collect();
            Ok(Value::List(items))
        }
        Err(e) => Err(format!("llm_extract: {e}")),
    }
}

pub fn llm_embed(args: &[Value]) -> Result<Value, String> {
    let text = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    
    let backend = MiMoBackend::new(MIMO_API_KEY.to_string());
    
    match backend.embed(text) {
        Ok(embedding) => Ok(Value::List(embedding.into_iter().map(|v| Value::Int(v)).collect())),
        Err(e) => Err(format!("llm_embed: {e}")),
    }
}

pub fn llm_similarity(args: &[Value]) -> Result<Value, String> {
    let a = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None }).ok_or("llm_similarity: first arg must be list")?;
    let b = args.get(1).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).ok_or("llm_similarity: second arg must be list")?;
    
    let a_vals: Vec<i64> = a.iter().filter_map(|v| match v { Value::Int(n) => Some(*n), _ => None }).collect();
    let b_vals: Vec<i64> = b.iter().filter_map(|v| match v { Value::Int(n) => Some(*n), _ => None }).collect();
    
    if a_vals.len() != b_vals.len() || a_vals.is_empty() {
        return Ok(Value::Int(0));
    }
    
    let dot: f64 = a_vals.iter().zip(b_vals.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let norm_a: f64 = a_vals.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    let norm_b: f64 = b_vals.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        Ok(Value::Int(0))
    } else {
        Ok(Value::Int((dot / (norm_a * norm_b) * 1000.0) as i64))
    }
}
