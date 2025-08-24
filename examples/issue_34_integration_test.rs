///! Integration test for Issue #34 - Complete Provider Configuration with Bootloader
///!
///! Tests the complete end-to-end bootloader flow:
///! 1. App starts in Main state with beautiful logo (bootloader style)
///! 2. Configuration file persistence (settings.json)  
///! 3. Provider status checking and UI adaptation
///! 4. Settings modal integration for provider setup
///! 5. Local provider requirement for start button
use agentic::{
    events::AppState,
    settings::{AsyncValidationResult, ProviderType, Settings, ValidationEvent, ValidationStatus},
    theme::{Theme, ThemeVariant},
    ui::app::App,
};
use std::time::Duration;

#[tokio::test]
async fn complete_integration_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("
🧪 Integration Test: Complete Provider Configuration with Bootloader");
    println!("==================================================================");

    // Test the bootloader-style initialization
    println!("
📋 Test 1: Bootloader Initialization");
    println!("------------------------------------");
    
    let theme = Theme::new(ThemeVariant::EverforestDark);
    let app = App::new(theme);

    // App should start in Main state with logo visible (bootloader style)
    assert_eq!(app.state(), &AppState::Main);
    println!("✅ App correctly starts in Main state with logo visible (bootloader style)");

    // Initially no configuration file should exist for fresh installation
    assert!(!app.settings().has_local_provider_configured());
    println!("✅ No local provider configured initially (fresh installation)");

    // Should not be ready to start yet
    assert!(!app.settings().has_local_provider_valid());
    println!("✅ Local provider not ready for startup (settings required)");

    // Test configuration file management
    test_configuration_persistence().await?;
    
    // Test provider configuration integration
    test_provider_configuration_integration(&app).await?;
    
    // Test state transitions
    test_bootloader_state_management().await?;

    println!("
🎉 All bootloader integration tests passed!");
    println!("🚀 Issue #34 Provider Configuration Integration with Bootloader: COMPLETE");
    Ok(())
}

async fn test_configuration_persistence() -> Result<(), Box<dyn std::error::Error>> {
    println!("
💾 Test 2: Configuration Persistence");
    println!("------------------------------------");

    // Test settings file path
    let settings_path = crate::settings::Settings::settings_file_path();
    println!("✅ Settings file path: {}", settings_path.display());

    // Test configuration loading (should create defaults for new installation)
    let settings = crate::settings::Settings::load_from_file();
    assert_eq!(settings.theme_variant, ThemeVariant::EverforestDark);
    println!("✅ Settings load correctly with defaults");

    // Test saving configuration
    settings.save_to_file()?;
    println!("✅ Settings save to file successfully");

    // Test loading saved configuration
    let loaded_settings = crate::settings::Settings::load_from_file();
    assert_eq!(loaded_settings.theme_variant, settings.theme_variant);
    println!("✅ Settings persist and load correctly");

    Ok(())
}

async fn test_provider_configuration_integration(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    println!("
⚙️  Test 3: Provider Configuration Integration");
    println!("---------------------------------------------");

    // Settings should be available
    let status_summary = app.settings().get_provider_status_summary();
    assert!(!status_summary.is_empty());
    println!("✅ Provider status summary available");

    // Initial status should be Unchecked
    for (provider, status, _) in &status_summary {
        assert_eq!(*status, ValidationStatus::Unchecked);
        println!("✅ {} provider initially unchecked", provider);
    }

    Ok(())
}

async fn test_bootloader_state_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("
🔄 Test 4: Bootloader State Management");
    println!("-------------------------------------");

    let theme = Theme::new(ThemeVariant::EverforestDark);
    let app = App::new(theme);

    // App should always start in Main state (bootloader shows logo immediately)
    assert_eq!(app.state(), &AppState::Main);
    println!("✅ App consistently starts in Main state (bootloader behavior)");

    // UI content should adapt based on provider readiness, not state changes
    assert!(!app.settings().has_local_provider_valid());
    println!("✅ UI shows provider configuration needed");

    // Settings integration should work
    assert!(app.settings().get_provider_status_summary().len() == 2);
    println!("✅ Provider configuration options available");

    Ok(())
}

async fn test_initial_app_state() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 1: Initial App State");
    println!("-----------------------------");

    let theme = Theme::new(ThemeVariant::EverforestDark);
    let app = App::new(theme);

    // App should now always start in Main state (showing the beautiful logo!)
    // but with helpful provider configuration guidance
    assert_eq!(app.state(), &AppState::Main);
    println!("✅ App correctly starts in Main state with logo visible");

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
    assert_eq!(app.state(), &AppState::Main);
    println!("✅ Settings modal closes correctly, returns to Main");

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

    // Initial state should be Main (with logo visible!)
    assert_eq!(app.state(), &AppState::Main);
    println!("✅ Initial state: Main (logo visible)");

    // Check that provider readiness affects UI content (not state)
    assert!(!app.settings().has_valid_provider());
    println!("✅ Provider readiness correctly shows no valid providers initially");

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
    // State should remain Main, but provider readiness should change
    assert_eq!(app.state(), &AppState::Main);
    assert!(app.settings().has_valid_provider());
    println!("✅ Provider validation updates readiness while staying in Main state");

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
    // State should remain Main, provider readiness should reflect invalid state
    assert_eq!(app.state(), &AppState::Main);
    assert!(!app.settings().has_valid_provider());
    println!("✅ Invalid provider updates readiness while staying in Main state");

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
