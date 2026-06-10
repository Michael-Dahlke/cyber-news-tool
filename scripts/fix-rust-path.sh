#!/usr/bin/env bash
#
# fix-rust-path.sh
#
# Repairs the rustup proxy shims in ~/.cargo/bin when they become broken
# (symptom: `pnpm tauri dev` fails with "failed to run 'cargo metadata'"
# or "cargo: No such file or directory").
#
# Cause: Homebrew's rustup can leave the ~/.cargo/bin/{cargo,rustc,...} shims
# pointing at a non-existent rustup-init, and that rustup build does not do
# argv[0] multi-call dispatch. This relinks the shims directly to the real
# installed toolchain binaries.
#
# Usage:
#   ./scripts/fix-rust-path.sh
#
set -euo pipefail

CARGO_BIN="$HOME/.cargo/bin"
RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"

# Pick the active/default toolchain, falling back to the first one found.
TOOLCHAINS_DIR="$RUSTUP_HOME/toolchains"
if [ ! -d "$TOOLCHAINS_DIR" ]; then
  echo "error: no rustup toolchains found at $TOOLCHAINS_DIR" >&2
  echo "Install one with: rustup default stable" >&2
  exit 1
fi

# Prefer a stable toolchain for the current arch; otherwise take the first.
TOOLCHAIN_BIN=""
for d in "$TOOLCHAINS_DIR"/stable-* "$TOOLCHAINS_DIR"/*; do
  if [ -x "$d/bin/cargo" ]; then
    TOOLCHAIN_BIN="$d/bin"
    break
  fi
done

if [ -z "$TOOLCHAIN_BIN" ]; then
  echo "error: could not find a toolchain containing a cargo binary" >&2
  exit 1
fi

echo "Using toolchain bin: $TOOLCHAIN_BIN"
mkdir -p "$CARGO_BIN"

# Relink the cargo/rustc family directly to the real toolchain binaries.
for tool in cargo rustc rustdoc rustfmt cargo-clippy cargo-fmt clippy-driver; do
  if [ -x "$TOOLCHAIN_BIN/$tool" ]; then
    rm -f "$CARGO_BIN/$tool"
    ln -s "$TOOLCHAIN_BIN/$tool" "$CARGO_BIN/$tool"
    echo "  linked $tool"
  fi
done

# Point rustup at the real (Homebrew) rustup binary if available.
REAL_RUSTUP="$(command -v rustup 2>/dev/null || true)"
if [ -z "$REAL_RUSTUP" ] && [ -x /opt/homebrew/bin/rustup ]; then
  REAL_RUSTUP=/opt/homebrew/bin/rustup
fi
if [ -n "$REAL_RUSTUP" ] && [ "$REAL_RUSTUP" != "$CARGO_BIN/rustup" ]; then
  rm -f "$CARGO_BIN/rustup"
  ln -s "$REAL_RUSTUP" "$CARGO_BIN/rustup"
  echo "  linked rustup -> $REAL_RUSTUP"
fi

# Ensure ~/.cargo/bin is on PATH for this verification.
export PATH="$CARGO_BIN:$PATH"

echo
echo "Verifying:"
cargo --version
rustc --version
echo
echo "Done. If your shell still can't find cargo in new terminals, ensure"
echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
echo "is in your shell profile (~/.zshrc or PowerShell \$PROFILE)."
