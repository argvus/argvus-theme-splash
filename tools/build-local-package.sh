#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
I18N_DIR="$ROOT_DIR/../argvus-i18n"

if [[ ! -d "$I18N_DIR" ]]; then
  echo "Sibling crate argvus-i18n not found at $I18N_DIR." >&2
  exit 1
fi

if [[ -f "$ROOT_DIR/packaging/arch/PKGBUILD.local" ]]; then
  PACKAGING_DIR="$ROOT_DIR/packaging/arch"
else
  echo "PKGBUILD.local not found under packaging/arch." >&2
  exit 1
fi

BUILD_SCRIPT="$PACKAGING_DIR/PKGBUILD.local"
TEMP_BUILD_SCRIPT=""
cleanup() {
  [[ -z "$TEMP_BUILD_SCRIPT" ]] || rm -f "$TEMP_BUILD_SCRIPT"
}
trap cleanup EXIT

if [[ -f "$ROOT_DIR/Cargo.toml" ]]; then
  cargo_version="$(awk -F '"' '/^version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")"
  if [[ -n "$cargo_version" ]]; then
    TEMP_BUILD_SCRIPT="$(mktemp)"
    sed "s/^pkgver=.*/pkgver=${cargo_version}/" "$BUILD_SCRIPT" > "$TEMP_BUILD_SCRIPT"
    BUILD_SCRIPT="$TEMP_BUILD_SCRIPT"
  fi
fi

metadata="$({ cd "$PACKAGING_DIR" && bash -c 'source "$1"; printf "%s\n%s\n" "$pkgname" "$pkgver"' bash "$BUILD_SCRIPT"; })"
pkgname="$(printf '%s\n' "$metadata" | sed -n '1p')"
pkgver="$(printf '%s\n' "$metadata" | sed -n '2p')"
archive="$PACKAGING_DIR/${pkgname}-${pkgver}.tar.gz"

if grep -q "${pkgname}-\${pkgver}.tar.gz\|\${pkgname}-\${pkgver}.tar.gz\|${pkgname}-${pkgver}.tar.gz" "$BUILD_SCRIPT"; then
  echo "Creating local source archive: $archive"

  staging="$(mktemp -d)"
  cleanup() {
    rm -rf "$staging"
    [[ -z "$TEMP_BUILD_SCRIPT" ]] || rm -f "$TEMP_BUILD_SCRIPT"
  }
  trap cleanup EXIT

  mkdir -p "$staging/${pkgname}-${pkgver}" "$staging/argvus-i18n"

  tar -cf - \
    --exclude='./.git' \
    --exclude='./.release' \
    --exclude='./packages-repo' \
    --exclude='./dist' \
    --exclude='./tmp' \
    --exclude='./target' \
    --exclude='./pkg' \
    --exclude='./packaging/pkg' \
    --exclude='./packaging/src' \
    --exclude='./packaging/arch/pkg' \
    --exclude='./packaging/arch/src' \
    --exclude='./packaging/*.pkg.tar*' \
    --exclude='./packaging/arch/*.pkg.tar*' \
    --exclude="./${pkgname}-${pkgver}.tar.gz" \
    --exclude="./packaging/arch/${pkgname}-${pkgver}.tar.gz" \
    -C "$ROOT_DIR" . | tar -xf - -C "$staging/${pkgname}-${pkgver}"

  tar -cf - \
    --exclude='./.git' \
    --exclude='./tmp' \
    --exclude='./target' \
    --exclude='./dist' \
    -C "$I18N_DIR" . | tar -xf - -C "$staging/argvus-i18n"

  tar -czf "$archive" -C "$staging" "${pkgname}-${pkgver}" argvus-i18n
fi

if [[ -n "${MAKEPKG_FLAGS:-}" ]]; then
  # shellcheck disable=SC2206
  flags=(${MAKEPKG_FLAGS})
else
  flags=(--nodeps --noconfirm --needed --cleanbuild --clean --force)
fi

cd "$PACKAGING_DIR"
makepkg -p PKGBUILD.local "${flags[@]}" "$@"

packages="$(find "$PACKAGING_DIR" -maxdepth 1 -type f -name "${pkgname}-*.pkg.tar.zst" -print | sort)"
if [[ -n "$packages" ]]; then
  printf 'Packages created:\n%s\n' "$packages"

  DIST_DIR="$ROOT_DIR/dist"
  mkdir -p "$DIST_DIR"
  mv -f $packages "$DIST_DIR/"
  printf 'Moved to %s:\n' "$DIST_DIR"
  printf '%s\n' "$packages" | xargs -I{} basename {}
fi