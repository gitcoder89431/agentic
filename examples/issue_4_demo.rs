#!/usr/bin/env cargo

//! # Issue #4: Input Handling & State Management Demo
//! 
//! This demo showcases the event-driven input handling and state management
//! system implemented for Issue #4. It demonstrates:
//! 
//! - Clean event-driven architecture with AppEvent enum
//! - Proper async/await runtime with tokio
//! - State management through AppState enum
//! - Clean application lifecycle (Setup -> Running -> Shutdown)
//! - Input handling with keyboard events
//! - Terminal resize detection and handling
//! - Error state management
//! 
//! ## Features Demonstrated:
//! - **ESC/q**: Quit application
//! - **t/T**: Toggle between Everforest Dark/Light themes
//! - **Ctrl+C**: Force quit with signal handling
//! - **Terminal Resize**: Automatic layout recalculation
//! 
//! ## Architecture:
//! ```
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   EventHandler  │───▶│   AppEvent      │───▶│   AppState      │
//! │  (Input Loop)   │    │  (Commands)     │    │  (Lifecycle)    │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!           │                       │                       │
//!           ▼                       ▼                       ▼
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │    Terminal     │    │   App::run()    │    │   UI Render     │
//! │   (Crossterm)   │    │  (Main Loop)    │    │  (Ratatui)      │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```

use agentic::{
    events::{AppEvent, AppState},
    theme::Theme,
    ui::app::App,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Issue #4: Input Handling & State Management Demo");
    println!("═══════════════════════════════════════════════════");
    println!("🚀 Starting event-driven TUI application...");
    println!("📊 Features: Async runtime, State management, Input handling");
    println!("🎨 Theme: Everforest Dark/Light with toggle support");
    println!("📐 Layout: Taffy 3-layer responsive system");
    println!();
    println!("Controls:");
    println!("  • ESC/q: Quit application");
    println!("  • t/T: Toggle theme variant");
    println!("  • Ctrl+C: Force quit");
    println!("  • Resize terminal: Test responsive layout");
    println!();
    println!("Press Enter to launch TUI...");
    
    // Wait for user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    // Initialize theme and application
    let theme = Theme::default();
    let mut app = App::new(theme);
    
    // Setup terminal
    let mut terminal = App::setup_terminal()?;
    
    // Run the application
    let result = app.run(&mut terminal).await;
    
    // Always restore terminal
    App::restore_terminal(&mut terminal)?;
    
    match result {
        Ok(_) => {
            println!("\n✅ Application exited successfully!");
            println!("🎯 Issue #4 Implementation Complete:");
            println!("   ✓ Event-driven architecture");
            println!("   ✓ Async/await runtime");
            println!("   ✓ Clean state management");
            println!("   ✓ Input handling system");
            println!("   ✓ Terminal lifecycle management");
        }
        Err(e) => {
            eprintln!("\n❌ Application error: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
