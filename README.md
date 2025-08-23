# 🚀 Agentic - Production-Ready TUI Framework

[![Rust CI](https://github.com/gitcoder89431/agentic/actions/workflows/rust.yml/badge.svg)](https://github.com/gitcoder89431/agentic/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Version:** 0.1.0  
> **Status:** Production Ready ✅

A beautiful, responsive terminal user interface (TUI) application built with modern Rust architecture. Features Everforest theming, Taffy flexbox layouts, and comprehensive event handling.

## 🎨 Features

- **🌲 Everforest Theme System**: Dark/Light theme variants with runtime switching
- **📐 Responsive Layouts**: Taffy-powered flexbox-style layout engine
- **⚡ Event-Driven Architecture**: Clean async/await with proper state management  
- **🔧 Production Ready**: Comprehensive CI/CD pipeline with quality gates
- **🎯 Zero Dependencies**: Minimal, focused dependency tree
- **✨ Beautiful ASCII Art**: Elegant logo with centered presentation

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+** (tested with latest stable)
- **Terminal** with true color support recommended

### Installation & Running

```bash
# Clone the repository
git clone https://github.com/gitcoder89431/agentic.git
cd agentic

# Run the application
cargo run

# Or build for release
cargo build --release
./target/release/agentic
```

### Controls

- **ESC / q**: Quit application  
- **t / T**: Toggle between Dark/Light themes
- **Ctrl+C**: Force quit with signal handling
- **Terminal Resize**: Automatic layout recalculation

## 🛠️ Development

### Local Development

Before pushing changes, run these checks locally:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run lints  
cargo clippy -- -D warnings

# Run tests
cargo test

# Check compilation
cargo check
```

### 🔧 CI/CD Pipeline

Our GitHub Actions CI pipeline ensures code quality with multiple validation layers:

#### Pipeline Jobs

1. **🏗️ Build & Test** (`build_and_test`)
   - Code formatting validation (`cargo fmt --check`)
   - Lint checking with zero warnings (`cargo clippy -- -D warnings`)
   - Compilation verification (`cargo build --verbose`)
   - Test suite execution (`cargo test --verbose`)

2. **🛡️ Security Audit** (`security_audit`)
   - Vulnerability scanning with `cargo audit`
   - Checks for known security issues in dependencies

3. **📦 Dependency Check** (`check_dependencies`)
   - Validates dependency freshness with `cargo outdated`
   - Ensures we're using up-to-date packages

#### Trigger Conditions

- **Pushes to `main`**: Full pipeline execution
- **Pull Requests**: All jobs run before merge approval
- **Manual triggers**: Available via GitHub Actions UI

#### Performance Optimizations

- **Cargo Registry Cache**: Speeds up dependency downloads
- **Target Directory Cache**: Accelerates compilation 
- **Hash-Based Invalidation**: Efficient cache management with `Cargo.lock`

#### Common Failure Scenarios & Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| `cargo fmt --check failed` | Inconsistent formatting | Run `cargo fmt` locally |
| `cargo clippy warnings` | Lint violations | Fix warnings or use `#[allow(...)]` |
| `tests failed` | Broken functionality | Fix failing tests |
| `security vulnerabilities` | Outdated dependencies | Run `cargo audit fix` |

### 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    AGENTIC TUI                          │
├─────────────────────────────────────────────────────────┤
│  Event System     │  Theme Engine    │  Layout Engine   │
│  ──────────────   │  ─────────────   │  ─────────────   │
│  • AppEvent       │  • Everforest    │  • Taffy 3-Layer │
│  • AppState       │  • Dark/Light    │  • Header/Body   │
│  • Async Runtime  │  • Runtime Switch│  • Footer/Flex   │
├─────────────────────────────────────────────────────────┤
│                    Ratatui Core                         │
│                  Crossterm Backend                      │
└─────────────────────────────────────────────────────────┘
```

### 📁 Project Structure

```
agentic/
├── .github/workflows/    # CI/CD pipeline configuration
├── src/
│   ├── events.rs        # Event handling & state management
│   ├── layout.rs        # Taffy flexbox layout engine
│   ├── theme.rs         # Everforest theme system  
│   ├── ui/              # User interface components
│   │   ├── app.rs       # Main application logic
│   │   └── mod.rs       # UI module declarations
│   ├── lib.rs           # Library root
│   └── main.rs          # Application entry point
├── examples/            # Demo applications
├── Cargo.toml          # Project dependencies
└── README.md           # Project documentation
```

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Manual Testing

```bash
# Theme demo
cargo run --example theme_demo

# Layout demo  
cargo run --example layout_demo

# Input handling demo
cargo run --example issue_4_demo
```

## 📦 Dependencies

### Core Dependencies
- **ratatui** (0.27): Terminal UI framework
- **crossterm** (0.27): Cross-platform terminal handling
- **tokio** (1.0): Async runtime  
- **taffy** (0.4): Flexbox layout engine

### Development Dependencies
- **GitHub Actions**: Automated CI/CD
- **cargo-audit**: Security vulnerability scanning
- **cargo-outdated**: Dependency freshness checks

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Run** local checks (`cargo fmt && cargo clippy && cargo test`)
4. **Commit** changes (`git commit -m 'Add amazing feature'`)
5. **Push** to branch (`git push origin feature/amazing-feature`)
6. **Open** a Pull Request

### Pull Request Process

- ✅ All CI checks must pass (formatting, lints, tests, security)
- ✅ Code must follow Rust best practices
- ✅ Include tests for new functionality  
- ✅ Update documentation as needed

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Ratatui Community** for the excellent TUI framework
- **Taffy Team** for the powerful layout engine  
- **Everforest** color scheme for beautiful aesthetics
- **Rust Community** for amazing tooling and ecosystem

---

**Built with ❤️ using Rust** 🦀