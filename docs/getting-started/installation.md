# 📦 Installation Matrix

Choose your preferred installation method:

## 1. Cargo (Rust)

Install directly from source with native CPU vectorization:

```bash
cargo install cartograph-cli
```

## 2. Homebrew (macOS & Linux)

Install via official Tap:

```bash
brew tap PankajRedekar/cartograph
brew install cartograph
```

Or once available in `homebrew/core`:

```bash
brew install cartograph
```

## 3. Python Wheels (PyPI)

Install the Python SDK with native AST engine:

```bash
pip install cartograph-python
```

Or with Graph Data Science extras:

```bash
pip install "cartograph-python[all]"
```

In your Python code:

```python
import cartograph
```

## 4. Precompiled GitHub Release Binaries

Download precompiled binaries for your architecture:
- Linux x86_64 / aarch64
- macOS Apple Silicon (`arm64`) / Intel (`x86_64`)
- Windows x86_64

Verify installation:

```bash
cartograph --version
```
