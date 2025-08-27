//! # Agentic Core Library
//!
//! This crate provides the core functionality for the Agentic TUI application.
//! It contains all the business logic, data structures, and communication protocols
//! that are independent of any specific user interface.
//!
//! ## Modules
//!
//! - `models`: Data structures and validation logic for AI models
//! - `settings`: Application configuration management
//! - `theme`: UI theming system

pub mod cloud;
pub mod models;
pub mod orchestrator;
pub mod settings;
pub mod theme;

#[cfg(test)]
mod tests {
    use crate::models::ModelValidator;
    use crate::settings::Settings;
    use crate::theme::ThemeVariant;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.theme, ThemeVariant::EverforestDark);
        assert_eq!(settings.endpoint, "localhost:11434");
        assert_eq!(settings.local_model, "[SELECT]");
        assert_eq!(settings.api_key, "sk-or-v1-982...b52");
        assert_eq!(settings.cloud_model, "[SELECT]");
    }

    #[test]
    fn test_settings_validation() {
        let settings = Settings::default();
        let result = settings.is_valid();

        // Default settings should fail validation (placeholder values)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_openrouter_integration() {
        let validator = ModelValidator::new();
        let api_key = "sk-or-v1-test-key-redacted";

        let result = validator.fetch_openrouter_models(api_key).await;

        match result {
            Ok(models) => {
                println!("✅ OpenRouter test passed! Found {} models", models.len());
                assert!(!models.is_empty(), "Should find at least some models");

                // Count free vs paid models
                let free_models: Vec<_> = models
                    .iter()
                    .filter(|m| m.pricing.prompt == "0" && m.pricing.completion == "0")
                    .collect();
                let paid_models: Vec<_> = models
                    .iter()
                    .filter(|m| m.pricing.prompt != "0" || m.pricing.completion != "0")
                    .collect();

                println!(
                    "  📊 Found {} free models, {} paid models",
                    free_models.len(),
                    paid_models.len()
                );
                assert!(
                    !free_models.is_empty(),
                    "Should have at least some free models"
                );
                assert!(
                    !paid_models.is_empty(),
                    "Should have at least some paid models"
                );

                // Print first few free models for verification
                println!("  🆓 Free models:");
                for (i, model) in free_models.iter().enumerate().take(3) {
                    println!(
                        "    {}. {} ({}k context)",
                        i + 1,
                        model.name,
                        model.context_length / 1000
                    );
                }

                // Print first few paid models for verification
                println!("  💰 Paid models:");
                for (i, model) in paid_models.iter().enumerate().take(2) {
                    println!(
                        "    {}. {} ({}k context)",
                        i + 1,
                        model.name,
                        model.context_length / 1000
                    );
                }
            }
            Err(e) => {
                println!("❌ OpenRouter test failed: {}", e);
                panic!("OpenRouter integration test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_ollama_integration() {
        let validator = ModelValidator::new();
        let endpoint = "localhost:11434";

        let result = validator.fetch_ollama_models(endpoint).await;

        match result {
            Ok(models) => {
                println!("✅ Ollama test passed! Found {} local models", models.len());

                // Print available models
                for (i, model) in models.iter().enumerate() {
                    println!("  {}. {} ({})", i + 1, model.name, model.size);
                }

                // If we have models, test validation with the first one
                if let Some(first_model) = models.first() {
                    println!("🔍 Testing validation with model: {}", first_model.name);
                    let validation_result = validator
                        .validate_local_endpoint(endpoint, &first_model.name)
                        .await;
                    match validation_result {
                        Ok(()) => println!("✅ Local endpoint validation passed!"),
                        Err(e) => println!("❌ Local endpoint validation failed: {}", e),
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Ollama test skipped - not running or accessible: {}", e);
                // Don't panic here since Ollama might not be running in all test environments
            }
        }
    }

    #[tokio::test]
    async fn test_dual_endpoint_validation() {
        use crate::settings::Settings;

        println!("🔄 Testing dual endpoint validation...");

        let validator = ModelValidator::new();

        // First, get actual working models
        let ollama_models = validator.fetch_ollama_models("localhost:11434").await;
        let api_key = "sk-or-v1-test-key-redacted";
        let openrouter_models = validator.fetch_openrouter_models(api_key).await;

        match (ollama_models, openrouter_models) {
            (Ok(local_models), Ok(cloud_models))
                if !local_models.is_empty() && !cloud_models.is_empty() =>
            {
                println!("📦 Testing with real models:");
                println!("  Local: {}", local_models[0].name);
                println!("  Cloud: {}", cloud_models[0].name);

                // Create settings with actual working models
                let mut settings = Settings::default();
                settings.local_model = local_models[0].name.clone();
                settings.cloud_model = cloud_models[0].id.clone();
                settings.api_key = api_key.to_string();

                // Test local-only validation
                match settings.validate_local_only().await {
                    Ok(()) => println!("✅ Local-only validation passed!"),
                    Err(e) => println!("❌ Local-only validation failed: {:?}", e),
                }

                // Test cloud-only validation
                match settings.validate_cloud_only().await {
                    Ok(()) => println!("✅ Cloud-only validation passed!"),
                    Err(e) => println!("❌ Cloud-only validation failed: {:?}", e),
                }

                // Test full dual validation
                match settings.validate_endpoints().await {
                    Ok(()) => println!("✅ Dual endpoint validation passed!"),
                    Err(e) => println!("❌ Dual endpoint validation failed: {:?}", e),
                }
            }
            (Ok(local_models), Ok(cloud_models)) => {
                // Handle case where models exist but might be empty
                println!("📦 Models found but lists might be empty:");
                println!("  Local models: {}", local_models.len());
                println!("  Cloud models: {}", cloud_models.len());
            }
            (Err(local_err), Ok(cloud_models)) => {
                println!("⚠️ Local endpoint not available: {}", local_err);
                println!(
                    "✅ Cloud endpoint working with {} models",
                    cloud_models.len()
                );
            }
            (Ok(local_models), Err(cloud_err)) => {
                println!(
                    "✅ Local endpoint working with {} models",
                    local_models.len()
                );
                println!("⚠️ Cloud endpoint not available: {}", cloud_err);
            }
            (Err(local_err), Err(cloud_err)) => {
                println!("⚠️ Both endpoints unavailable:");
                println!("  Local: {}", local_err);
                println!("  Cloud: {}", cloud_err);
            }
        }
    }

    #[tokio::test]
    async fn test_modal_model_loading() {
        println!("🧪 Testing modal model loading scenarios...");

        let validator = ModelValidator::new();
        let api_key = "sk-or-v1-test-key-redacted";

        // Test OpenRouter models for modal display
        match validator.fetch_openrouter_models(api_key).await {
            Ok(models) => {
                println!("✅ Cloud models loaded for modal: {} models", models.len());
                assert!(!models.is_empty(), "Should have models for display");

                // Count free vs paid models and verify sorting
                let mut free_count = 0;
                let mut paid_count = 0;
                let mut found_paid_after_free = false;

                for model in models.iter() {
                    let is_free = model.pricing.prompt == "0" && model.pricing.completion == "0";
                    if is_free {
                        free_count += 1;
                        // If we already found paid models, this breaks the sorting
                        assert!(
                            !found_paid_after_free,
                            "Free models should come before paid models"
                        );
                    } else {
                        paid_count += 1;
                        found_paid_after_free = true;
                    }
                }

                println!(
                    "  📊 Model breakdown: {} free, {} paid",
                    free_count, paid_count
                );
                assert!(free_count > 0, "Should have at least some free models");
                assert!(paid_count > 0, "Should have at least some paid models");

                // Verify the first few models have the required fields for UI display
                for model in models.iter().take(3) {
                    let pricing_type =
                        if model.pricing.prompt == "0" && model.pricing.completion == "0" {
                            "(free)"
                        } else {
                            "(paid)"
                        };
                    println!(
                        "  - {} {} ({} tokens)",
                        model.name, pricing_type, model.context_length
                    );
                    assert!(!model.name.is_empty(), "Model name should not be empty");
                    assert!(!model.id.is_empty(), "Model ID should not be empty");
                    assert!(
                        model.context_length > 0,
                        "Context length should be positive"
                    );
                }
            }
            Err(e) => {
                panic!("Failed to load OpenRouter models for modal: {}", e);
            }
        }

        // Test Ollama models for modal display
        match validator.fetch_ollama_models("localhost:11434").await {
            Ok(models) => {
                println!("✅ Local models loaded for modal: {} models", models.len());
                for model in models.iter().take(3) {
                    println!("  - {} ({})", model.name, model.size);
                    assert!(!model.name.is_empty(), "Model name should not be empty");
                    assert!(!model.size.is_empty(), "Model size should not be empty");
                }
            }
            Err(e) => {
                println!("⚠️ Local models test skipped (Ollama not running): {}", e);
            }
        }
    }

    #[test]
    fn test_api_key_truncation() {
        // Test the API key display formatting (simulating the settings modal function)
        fn format_api_key_display(api_key: &str) -> String {
            if api_key.is_empty() {
                return String::new();
            }

            if api_key.len() <= 21 {
                api_key.to_string()
            } else {
                format!("{}...{}", &api_key[..15], &api_key[api_key.len() - 3..])
            }
        }

        let test_key = "sk-or-v1-test-key-redacted";
        let formatted = format_api_key_display(test_key);

        println!("🧪 Testing API key truncation:");
        println!("  Original: {} (len: {})", test_key, test_key.len());
        println!("  Formatted: {} (len: {})", formatted, formatted.len());

        assert_eq!(formatted, "sk-or-v1-7d9200...3ac");
        assert_eq!(formatted.len(), 21); // 15 chars + "..." + 3 chars = 21 total

        // Test shorter key (should not be truncated)
        let short_key = "sk-test-123";
        let short_formatted = format_api_key_display(short_key);
        assert_eq!(short_formatted, short_key);

        // Test empty key
        let empty_formatted = format_api_key_display("");
        assert_eq!(empty_formatted, "");

        println!("✅ API key truncation test passed!");
    }
}
