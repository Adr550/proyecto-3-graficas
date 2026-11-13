#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if command -v cargo >/dev/null 2>&1; then cargo_tool=cargo; else cargo_tool="$HOME/.cargo/bin/cargo"; fi
"$cargo_tool" build --release --locked
app='dist/Orbit Club.app'
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/crownfall "$app/Contents/MacOS/crownfall.next"
chmod +x "$app/Contents/MacOS/crownfall.next"
mv -f "$app/Contents/MacOS/crownfall.next" "$app/Contents/MacOS/crownfall"
cp assets/Info.plist "$app/Contents/Info.plist"
cp assets/OrbitPath.icns "$app/Contents/Resources/"
printf 'Built %s\n' "$app"
