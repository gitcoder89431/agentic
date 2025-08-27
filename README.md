# Agentic

```
    █████╗  ██████╗ ███████╗███╗   ██╗████████╗██╗ ██████╗
   ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝
   ███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║   ██║██║     
   ██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║   ██║██║     
   ██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║   ██║╚██████╗
   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝ ╚═════╝
```

[![Build Status](https://github.com/gitcoder89431/agentic/workflows/CI/badge.svg)](https://github.com/gitcoder89431/agentic/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Crate](https://img.shields.io/crates/v/agentic.svg)](https://crates.io/crates/agentic)

**Agentic :: The agent you work WITH**  
**AI Model Orchestrator & Agent Framework**

<!-- GIF Demo placeholder - add your demo GIF here -->
![Demo](demo.gif)

## Core Philosophy

Agentic transforms the typical command-response dynamic into true collaboration. Instead of barking orders at an AI, you work together through thoughtful query refinement and synthesis. Our "Karesansui" design philosophy creates a zen garden of computational thought - clean, minimalist, and purposeful. Every interaction flows like carefully placed stones in sand, building toward profound understanding rather than quick answers.

## Key Features

• **Collaborative Query Refinement** via a Local AI Orchestrator  
• **Seamless Integration** with Powerful Cloud Models (via OpenRouter)  
• **Minimalist, Keyboard-Driven** "Zen Garden" TUI  
• **Creates Structured, "Atomic Notes"** (Markdown + YAML) for your Knowledge Base  
• **Built in Rust** 🦀 for a Fast, Native Experience

## Installation

### Download Release Binaries

**macOS (Intel/Apple Silicon)**
```bash
curl -L https://github.com/gitcoder89431/agentic/releases/download/v0.1.0/agentic-macos.tar.gz | tar xz
sudo mv agentic /usr/local/bin/
```

**Linux (x86_64)**  
```bash
curl -L https://github.com/gitcoder89431/agentic/releases/download/v0.1.0/agentic-linux.tar.gz | tar xz
sudo mv agentic /usr/local/bin/
```

**Windows**
```powershell
# Download from releases page and add to PATH
# https://github.com/gitcoder89431/agentic/releases/download/v0.1.0/agentic-windows.zip
```

### Build from Source
```bash
git clone https://github.com/gitcoder89431/agentic.git
cd agentic
cargo build --release
./target/release/agentic-tui
```

## Configuration

⚠️ **Important**: Agentic requires **BOTH** a local AI model (for query orchestration) and a cloud model (for synthesis) to function. The local model privately refines your questions, then the cloud model creates the final insights.

### Complete Setup Guide

Follow these steps in order - you need both components:

#### Step 1: Local AI Setup (Required)

1. **Install Ollama** (Free, runs on your computer)
   ```bash
   # macOS
   brew install ollama
   
   # Or download from: https://ollama.ai
   ```

2. **Download a Local Model**
   ```bash
   # Start Ollama service
   ollama serve
   
   # In another terminal, pull a model (this may take a few minutes)
   ollama pull llama3.2:3b    # Good balance of speed/quality
   # or
   ollama pull qwen2.5:7b     # Higher quality, needs more RAM
   ```

3. **Configure in Agentic**
   - In Settings, set "Local Endpoint" to `localhost:11434`
   - Select your downloaded model from the list
   - This handles initial query refinement privately on your machine

#### Step 2: Cloud Setup (Required)

1. **Get an OpenRouter Account**
   - Visit [openrouter.ai](https://openrouter.ai) and sign up (takes 2 minutes)
   - Generate an API key from your dashboard
   - Add $5-10 credit OR use free models (see guide below)

2. **Configure in Agentic**
   - Run `agentic` in your terminal
   - Press `s` to open Settings
   - Navigate to "Cloud API Key" and paste your OpenRouter key
   - Browse available models and select one (see model selection guide below)
   - Press `s` to save

### 🎯 Model Selection Guide

When choosing a cloud model in Agentic's settings, look for these indicators:

**💰 Cost Structure:**
- Models with `:free` suffix = Completely free (perfect for learning)
- Models with pricing = Pay per token (~$0.50-10 per 1M tokens)
- Check the "pricing" column to see prompt/completion costs

**🧠 Model Types:**
- Look for `:instruct` or `:chat` in the name = Good for conversations (what you want)
- Avoid `:base` models = Raw models without instruction training
- Avoid `:embed` models = For embeddings only, not chat

**📏 Context Length:**
- 4k-8k tokens = Good for short conversations  
- 32k-128k tokens = Better for longer discussions
- 1M+ tokens = Can handle very long contexts (costs more)

**🏷️ Model Families:**
- `anthropic/claude-*` = Excellent reasoning and safety
- `openai/gpt-*` = Well-rounded performance
- `meta-llama/*` = Open source, good quality
- `google/gemini-*` = Strong at analysis and coding
- `deepseek/*` = Often have free versions available

**💡 Beginner Tips:**
- Start with any `:free` model to test the system
- If you have credits, try `anthropic/claude-3.5-sonnet` for quality
- Higher context length = more expensive but can handle longer discussions
- The model list updates frequently - newer models often perform better

#### Step 3: Ready to Collaborate!

- Type your question naturally
- Watch the local model orchestrate thoughtful proposals
- Choose a proposal for the cloud model to synthesize
- Save the resulting "atomic note" to your knowledge base
- **Files are automatically saved** to `~/Documents/ruixen/` as Markdown with YAML metadata

### Why Both Models?

The **local model** (Ollama) handles query orchestration privately on your machine, while the **cloud model** (OpenRouter) provides powerful synthesis capabilities. This hybrid approach gives you both privacy and cutting-edge AI performance!

### Troubleshooting

**"Local endpoint not accessible"**
- Make sure Ollama is running: `ollama serve`
- Check the endpoint in settings: `localhost:11434`

**"OpenRouter API key invalid"**  
- Verify your key starts with `sk-or-v1-`
- Check you have credits or selected a free model

**"Model not found"**
- For local: ensure model is downloaded with `ollama list`
- For cloud: verify model name exactly matches OpenRouter's list

## Usage

**Navigation**
- `Tab/Shift+Tab` - Navigate between UI elements
- `↑/↓ or j/k` - Move through lists and proposals  
- `Enter` - Select/Confirm actions
- `Esc` - Return to previous screen
- `q` - Quit application

**Slash Commands**
- `/settings` - Open configuration modal
- `/about` - View application information  
- `/quit` - Exit the application

**Key Bindings**
- `s` - Quick access to Settings
- `a` - Quick access to About
- `Left/Right` - Scroll through About page content

## Architecture

Agentic follows the RuixenOS workspace architecture:

```
agentic/
├── crates/
│   ├── agentic-core/     # The "motor" - reusable AI logic
│   ├── agentic-tui/      # The "drill" - terminal interface  
│   └── starlit-gui/      # Future graphical interface
└── Cargo.toml            # Workspace configuration
```

This modular design allows the same AI capabilities to power multiple interfaces while maintaining clean separation between logic and presentation.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Built with constitutional Rust patterns and love. Issues and PRs welcome!

---

*The curiosity machine doesn't just process queries - it awakens wonder.*