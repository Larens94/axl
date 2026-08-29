# Source from scripts/*.sh — non-login shells often omit ~/.cargo/bin.
if ! command -v cargo >/dev/null 2>&1; then
  export PATH="${HOME}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:${PATH}"
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found. Install Rust: https://rustup.rs" >&2
  exit 127
fi

# Gate 8 secret refs for portal/auth/vendite demo skills (never plaintext in IR).
export AXL_AUTH_PEPPER="${AXL_AUTH_PEPPER:-axl-auth-demo-pepper}"
export AXL_AUTH_JWT="${AXL_AUTH_JWT:-axl-auth-demo-jwt}"
export AXL_VENDITE_BEARER="${AXL_VENDITE_BEARER:-axl-vendite-demo}"
export AXL_VENDITE_JWT="${AXL_VENDITE_JWT:-axl-vendite-demo-jwt}"
