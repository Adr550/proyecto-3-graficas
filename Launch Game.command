#!/bin/zsh
cd "${0:A:h}"
if [[ -d 'dist/Orbit Club.app' ]]; then
  open -n 'dist/Orbit Club.app'
elif [[ -x target/release/crownfall ]]; then
  exec ./target/release/crownfall
else
  cargo_tool="$(command -v cargo)"
  [[ -z "$cargo_tool" && -x "$HOME/.cargo/bin/cargo" ]] && cargo_tool="$HOME/.cargo/bin/cargo"
  if [[ -n "$cargo_tool" ]]; then
    exec "$cargo_tool" run --release --locked
  else
    print 'Install Rust from https://rustup.rs, then run: cargo run --release'
    read '?Press Enter to close.'
  fi
fi
