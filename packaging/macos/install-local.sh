#!/usr/bin/env bash
# Build WordCraft.app for this Mac, install it to /Applications and delete the build output.
#
# Usage: packaging/macos/install-local.sh [--keep-target] [--dest DIR]
#
# The build goes to a throwaway CARGO_TARGET_DIR that is removed at the end (also on failure),
# so repeated installs don't pile up gigabytes under target/. --keep-target keeps it for faster
# rebuilds. The app is ad-hoc signed: fine on this Mac, not for distribution (use package.sh).
set -euo pipefail

KEEP_TARGET=0
DEST=/Applications
while [ $# -gt 0 ]; do
  case "$1" in
    --keep-target) KEEP_TARGET=1; shift ;;
    --dest) DEST="$2"; shift 2 ;;
    -h | --help) sed -n '2,8p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

# Homebrew's rustup is keg-only; make cargo findable without editing the shell profile.
if ! command -v cargo >/dev/null && [ -x /opt/homebrew/opt/rustup/bin/cargo ]; then
  export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/wordcraft-local-build}"
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/macos"

cleanup() {
  if [ "$KEEP_TARGET" = 0 ]; then
    echo "==> removing build output $CARGO_TARGET_DIR"
    rm -rf "$CARGO_TARGET_DIR"
  fi
  # env.sh creates an empty dist/release; drop it if nothing was put there.
  rmdir "$DIST" "$(dirname "$DIST")" 2>/dev/null || true
}
trap cleanup EXIT

echo "==> building WordCraft $VERSION (release) in $CARGO_TARGET_DIR"
(cd "$ROOT" && cargo build --release --locked -p wordcraft)

STAGE="$CARGO_TARGET_DIR/WordCraft.app"
rm -rf "$STAGE"
mkdir -p "$STAGE/Contents/MacOS" "$STAGE/Contents/Resources"
cp "$CARGO_TARGET_DIR/release/wordcraft" "$STAGE/Contents/MacOS/WordCraft"
cp "$ROOT/assets/app-icon/wordcraft.icns" "$STAGE/Contents/Resources/WordCraft.icns"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/${VERSION%%-*}/g" \
  -e "s/@BUILD_SHA@/${WORDCRAFT_BUILD_SHA:-unknown}/g" \
  "$HERE/Info.plist.in" >"$STAGE/Contents/Info.plist"
plutil -lint "$STAGE/Contents/Info.plist"
printf 'APPL????' >"$STAGE/Contents/PkgInfo"
codesign --force --sign - --timestamp=none --options runtime \
  --entitlements "$HERE/entitlements.plist" "$STAGE"

echo "==> installing to $DEST/WordCraft.app"
rm -rf "$DEST/WordCraft.app"
ditto "$STAGE" "$DEST/WordCraft.app"
echo "==> done: open -a \"$DEST/WordCraft.app\""
