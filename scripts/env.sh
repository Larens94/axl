# Source from scripts/*.sh — non-login shells often omit ~/.cargo/bin.
if ! command -v cargo >/dev/null 2>&1; then
  export PATH="${HOME}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:${PATH}"
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found. Install Rust: https://rustup.rs" >&2
  exit 127
fi
