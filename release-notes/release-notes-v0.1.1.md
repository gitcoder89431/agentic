## 🐛 Critical Hotfix: Terminal.app Color Compatibility

This patch release fixes a critical issue where Ruixen displayed incorrectly in pink/magenta colors on macOS Terminal.app, while working perfectly in iTerm2 and other modern terminals.

### 🔧 What's Fixed
- **Terminal Detection**: Added smart terminal detection using `$TERM_PROGRAM` environment variable
- **Color Fallback**: Terminal.app now uses indexed colors instead of RGB colors for perfect compatibility
- **Preserved Quality**: iTerm2, VS Code terminal, and other modern terminals still get beautiful Everforest RGB colors

### 🚀 Installation
Install the latest version:
```bash
cargo install ruixen
```

Or download platform-specific binaries from the release assets below.

### 💡 Technical Details
- Detects Terminal.app via `TERM_PROGRAM=Apple_Terminal`
- Provides indexed color fallback (Color::Green, Color::White, etc.)
- Maintains full RGB color support for modern terminals
- No performance impact or feature degradation

### 🎯 First Impressions Matter
This ensures Ruixen works beautifully across all macOS terminal environments, providing the best possible first experience for new users.

---
🦀 Built with Rust | 🤖 AI-Collaborative Development