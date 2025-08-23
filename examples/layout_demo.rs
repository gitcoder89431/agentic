// Example/test file to demonstrate Taffy layout usage
// This shows how the layout system computes responsive layouts

use agentic::layout::AppLayout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Taffy Layout System Demo ===\n");
    
    // Create a new layout
    let mut layout = AppLayout::new()?;
    
    // Test different terminal sizes
    let test_sizes = [
        (80, 24),   // Standard terminal
        (120, 30),  // Larger terminal
        (40, 15),   // Small terminal
        (200, 50),  // Very large terminal
    ];
    
    for (width, height) in test_sizes {
        println!("Terminal Size: {}x{}", width, height);
        
        let rects = layout.compute((width, height))?;
        
        println!("  Header: {}x{} at ({}, {})", 
                 rects.header.width, rects.header.height,
                 rects.header.x, rects.header.y);
                 
        println!("  Body:   {}x{} at ({}, {})", 
                 rects.body.width, rects.body.height,
                 rects.body.x, rects.body.y);
                 
        println!("  Footer: {}x{} at ({}, {})", 
                 rects.footer.width, rects.footer.height,
                 rects.footer.x, rects.footer.y);
                 
        // Verify layout constraints
        assert_eq!(rects.header.height, 3, "Header should be 3 rows high");
        assert_eq!(rects.footer.height, 3, "Footer should be 3 rows high");
        assert_eq!(rects.header.width, width, "Header should span full width");
        assert_eq!(rects.body.width, width, "Body should span full width");
        assert_eq!(rects.footer.width, width, "Footer should span full width");
        
        // Check that body fills remaining space
        let expected_body_height = height - 6; // Total - header(3) - footer(3)
        assert_eq!(rects.body.height, expected_body_height, 
                   "Body should fill remaining space");
        
        println!("  ✅ Layout constraints verified\n");
    }
    
    println!("🎉 All Taffy layout tests passed!");
    println!("The layout system correctly:");
    println!("  • Maintains fixed header/footer heights (3 rows each)");
    println!("  • Flexibly grows body to fill remaining space");
    println!("  • Responds to different terminal sizes");
    println!("  • Uses Taffy's flexbox-style layout engine");
    
    Ok(())
}
