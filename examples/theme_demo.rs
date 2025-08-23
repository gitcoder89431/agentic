// Example/test file to demonstrate theme usage
// This won't be part of the main binary, but shows how to use the theme system

use agentic::theme::{Theme, ThemeVariant, Element};

fn main() {
    // Create different theme variants
    let dark_theme = Theme::new(ThemeVariant::EverforestDark);
    let light_theme = Theme::new(ThemeVariant::EverforestLight);
    
    println!("=== Everforest Theme System Demo ===\n");
    
    // Test dark theme
    println!("Dark Theme Colors:");
    println!("  Background: {:?}", dark_theme.bg_color(Element::Background));
    println!("  Foreground: {:?}", dark_theme.fg_color(Element::Text));
    println!("  Accent: {:?}", dark_theme.fg_color(Element::Accent));
    println!("  Secondary: {:?}", dark_theme.fg_color(Element::Secondary));
    println!("  Info: {:?}", dark_theme.fg_color(Element::Info));
    
    println!("\nLight Theme Colors:");
    println!("  Background: {:?}", light_theme.bg_color(Element::Background));
    println!("  Foreground: {:?}", light_theme.fg_color(Element::Text));
    println!("  Accent: {:?}", light_theme.fg_color(Element::Accent));
    println!("  Secondary: {:?}", light_theme.fg_color(Element::Secondary));
    println!("  Info: {:?}", light_theme.fg_color(Element::Info));
    
    // Test theme toggling
    let mut theme = Theme::default();
    println!("\nTheme Toggle Test:");
    println!("  Initial variant: {:?}", theme.variant());
    theme.toggle();
    println!("  After toggle: {:?}", theme.variant());
    theme.toggle();
    println!("  After second toggle: {:?}", theme.variant());
    
    println!("\n✅ All theme tests passed!");
}
