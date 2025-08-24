///! Integration test for Issue #34 - Complete Provider Configuration Integration
///!
///! Tests the complete end-to-end flow:
///! 1. App starts in WaitingForConfig state
///! 2. Settings modal shows provider configuration
///! 3. Async validation works
///! 4. State transitions correctly
///! 5. UI components integrate properly
use agentic::{
    events::{AppEvent, AppState},
    settings::{AsyncValidationResult, ProviderType, Settings, ValidationEvent, ValidationStatus},
    theme::{Theme, ThemeVariant},
    ui::app::App,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Integration Test: Complete Provider Configuration");
    println!("==================================================");

    // Test 1: App starts in correct state
    test_initial_app_state().await?;

    // Test 2: Settings integration
    test_settings_integration().await?;

    // Test 3: Provider validation result handling
    test_provider_validation_handling().await?;

    // Test 4: State transitions
    test_state_transitions().await?;

    // Test 5: Provider configuration components
    test_provider_configuration_components().await?;

    println!("\n✅ All integration tests passed!");
    println!("🎯 Issue #34 Provider Configuration Integration: COMPLETE");

    Ok(())
}

async fn test_initial_app_state() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 1: Initial App State");
    println!("-----------------------------");

    let theme = Theme::new(ThemeVariant::EverforestDark);
    let app = App::new(theme);

    // App should start in WaitingForConfig since no providers are configured
    assert_eq!(app.state(), &AppState::WaitingForConfig);
    println!("✅ App correctly starts in WaitingForConfig state");

    // Provider readiness should be false
    assert!(!app.settings().has_valid_provider());
    println!("✅ Provider readiness check works correctly");

    // Available providers should be empty
    assert!(app.settings().get_available_providers().is_empty());
    println!("✅ Available providers list is correctly empty");

    Ok(())
}

async fn test_settings_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  Test 2: Settings Integration");
    println!("-------------------------------------");

    let theme = Theme::new(ThemeVariant::EverforestDark);
    let mut app = App::new(theme);

    // Test opening settings
    app.enter_settings();
    assert_eq!(app.state(), &AppState::Settings);
    println!("✅ Settings modal opens correctly");

    // Check that provider sections exist
    let provider_sections = app.settings().get_provider_sections();
    assert!(!provider_sections.is_empty());
    println!("✅ Provider sections are available in settings");

    // Check that both LOCAL and OPENROUTER sections exist
    let section_names: Vec<&str> = provider_sections.iter().map(|s| s.title.as_str()).collect();
    assert!(section_names.contains(&"LOCAL Provider"));
    assert!(section_names.contains(&"OPENROUTER Provider"));
    println!("✅ Both LOCAL and OPENROUTER provider sections exist");

    // Test closing settings
    app.exit_settings();
    assert_eq!(app.state(), &AppState::WaitingForConfig);
    println!("✅ Settings modal closes correctly, returns to WaitingForConfig");

    Ok(())
}

async fn test_provider_validation_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Test 3: Provider Validation Result Handling");
    println!("---------------------------------------------");

    let mut settings = Settings::new();

    // Check initial state
    assert_eq!(
        settings.local_provider.validation_status,
        ValidationStatus::Unchecked
    );
    assert_eq!(
        settings.openrouter_provider.validation_status,
        ValidationStatus::Unchecked
    );
    println!("✅ Initial provider status is Unchecked");

    // Test validation event handling
    let validation_event = ValidationEvent::ValidationComplete {
        provider: ProviderType::Local,
        result: AsyncValidationResult {
            status: ValidationStatus::Valid,
            message: Some("Connection successful".to_string()),
            response_time: Some(Duration::from_millis(250)),
        },
    };

    settings.handle_validation_event(validation_event);
    assert_eq!(
        settings.local_provider.validation_status,
        ValidationStatus::Valid
    );
    println!("✅ Validation event correctly updates provider status");

    // Test provider readiness after validation
    assert!(settings.has_valid_provider());
    println!("✅ Provider readiness correctly reflects valid provider");

    let available = settings.get_available_providers();
    assert!(available.contains(&ProviderType::Local));
    assert!(!available.contains(&ProviderType::OpenRouter));
    println!("✅ Available providers list correctly reflects valid providers");

    Ok(())
}

async fn test_state_transitions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Test 4: State Transitions");
    println!("-----------------------------");

    let theme = Theme::new(ThemeVariant::EverforestDark);
    let mut app = App::new(theme);

    // Initial state should be WaitingForConfig
    assert_eq!(app.state(), &AppState::WaitingForConfig);
    println!("✅ Initial state: WaitingForConfig");

    // Simulate a provider becoming valid
    let validation_event = ValidationEvent::ValidationComplete {
        provider: ProviderType::Local,
        result: AsyncValidationResult {
            status: ValidationStatus::Valid,
            message: Some("Connection successful".to_string()),
            response_time: Some(Duration::from_millis(150)),
        },
    };

    app.update_provider_status(validation_event);
    // State should transition to Main
    assert_eq!(app.state(), &AppState::Main);
    println!("✅ State correctly transitions to Main when provider becomes valid");

    // Simulate provider becoming invalid
    let invalid_event = ValidationEvent::ValidationComplete {
        provider: ProviderType::Local,
        result: AsyncValidationResult {
            status: ValidationStatus::Invalid,
            message: Some("Connection failed".to_string()),
            response_time: Some(Duration::from_millis(5000)),
        },
    };

    app.update_provider_status(invalid_event);
    // State should transition back to WaitingForConfig
    assert_eq!(app.state(), &AppState::WaitingForConfig);
    println!(
        "✅ State correctly transitions back to WaitingForConfig when provider becomes invalid"
    );

    Ok(())
}

async fn test_provider_configuration_components() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Test 5: Provider Configuration Components");
    println!("--------------------------------------------");

    let settings = Settings::new();

    // Test provider status summary
    let status_summary = settings.get_provider_status_summary();
    assert_eq!(status_summary.len(), 2); // LOCAL and OPENROUTER
    println!("✅ Provider status summary includes both providers");

    // Check initial status icons in provider sections
    let provider_sections = settings.get_provider_sections();
    for section in &provider_sections {
        assert_eq!(section.status_icon, "⚪"); // Unchecked
    }
    println!("✅ Initial status icons show unchecked state (⚪)");

    // Test field structure for LOCAL provider
    let local_section = provider_sections
        .iter()
        .find(|s| s.title == "LOCAL Provider")
        .expect("LOCAL Provider section should exist");

    assert!(!local_section.fields.is_empty());
    let _endpoint_field = local_section
        .fields
        .iter()
        .find(|f| f.label.contains("Endpoint"))
        .expect("Endpoint field should exist");

    println!("✅ LOCAL provider has endpoint configuration field");

    // Test field structure for OPENROUTER provider
    let openrouter_section = provider_sections
        .iter()
        .find(|s| s.title == "OPENROUTER Provider")
        .expect("OPENROUTER Provider section should exist");

    assert!(!openrouter_section.fields.is_empty());
    let _api_key_field = openrouter_section
        .fields
        .iter()
        .find(|f| f.label.contains("API Key"))
        .expect("API Key field should exist");

    println!("✅ OPENROUTER provider has API key configuration field");

    // Test API key masking functionality
    let test_key = "sk-or-v1-1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef00e";
    let masked = agentic::settings::mask_api_key(test_key);
    assert!(masked.starts_with("sk-or-v1-"));
    assert!(masked.ends_with("00e"));
    assert!(masked.contains("***"));
    println!("✅ API key masking works correctly");

    println!("\n🎉 All provider configuration components verified!");

    Ok(())
}
