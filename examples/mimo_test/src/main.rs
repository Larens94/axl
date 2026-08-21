use axl_core::mimo::MiMoBackend;
use axl_core::llm::LlmBackend;

fn main() {
    let api_key = "sk-ejmpfhhrc5eyh9n1bwp2yn0dt1vtghqclesto54fnju5my9c";
    let backend = MiMoBackend::new(api_key.to_string());

    println!("Testing MiMo backend...");
    println!("Model: xiaomi/mimo-v2.5-pro");
    println!();

    // Test 1: Generate
    println!("=== Test 1: Generate ===");
    match backend.generate("You are a helpful assistant.", &[("user".into(), "What is AXL?".into())]) {
        Ok(result) => println!("Response: {result}"),
        Err(e) => println!("Error: {e}"),
    }
    println!();

    // Test 2: Reason
    println!("=== Test 2: Reason ===");
    let system = "You are a careful reasoning assistant. Think step by step.";
    match backend.generate(system, &[("user".into(), "What are the benefits of Rust for LLM applications?".into())]) {
        Ok(result) => println!("Response: {result}"),
        Err(e) => println!("Error: {e}"),
    }
    println!();

    // Test 3: Classify
    println!("=== Test 3: Classify ===");
    let system = "Classify the following text into exactly one category: tutorial, news, opinion. Reply with ONLY the category name.";
    match backend.generate(system, &[("user".into(), "Rust is becoming the go-to language for building LLM infrastructure due to its memory safety and performance.".into())]) {
        Ok(result) => println!("Category: {result}"),
        Err(e) => println!("Error: {e}"),
    }
    println!();

    // Test 4: JSON
    println!("=== Test 4: Generate JSON ===");
    let system = "Respond with valid JSON: {\"name\": string, \"score\": int}";
    match backend.generate(system, &[("user".into(), "Extract: John got 95 points".into())]) {
        Ok(result) => println!("JSON: {result}"),
        Err(e) => println!("Error: {e}"),
    }

    println!();
    println!("All tests completed!");
}
