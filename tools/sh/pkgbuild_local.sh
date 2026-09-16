#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
I18N_DIR="$(cd -- "$ROOT_DIR/../argvus-i18n" && pwd)"
BUILD_DIR="$ROOT_DIR/build"
ARTIFACTS_DIR="$BUILD_DIR/artifacts"
DIST_DIR="$BUILD_DIR/dist"
PKGBUILD_DIR="$ROOT_DIR/packaging/arch/local"
PKGBUILD="$PKGBUILD_DIR/PKGBUILD"

[[ -d "$I18N_DIR" ]] || { echo "Sibling crate argvus-i18n not found at $I18N_DIR." >&2; exit 1; }

read -r pkgname pkgver <<<"$(bash -c 'source "$1"; printf "%s %s" "$pkgname" "$pkgver"' bash "$PKGBUILD")"
mkdir -p "$ARTIFACTS_DIR" "$DIST_DIR"
find "$DIST_DIR" -maxdepth 1 -type f -name "${pkgname}-*.pkg.tar.*" -delete

archive="$ARTIFACTS_DIR/${pkgname}-${pkgver}.tar.gz"
staging="$(mktemp -d)"
cleanup() { rm -rf "$staging"; }
trap cleanup EXIT
mkdir -p "$staging/${pkgname}-${pkgver}" "$staging/argvus-i18n"

tar -cf - \
  --exclude='./.git' --exclude='./build' --exclude='./dist' --exclude='./target' \
  --exclude='./packaging/arch/ci/src' --exclude='./packaging/arch/ci/pkg' \
  --exclude='./packaging/arch/local/src' --exclude='./packaging/arch/local/pkg' \
  --exclude='./packaging/arch/local/PKGBUILD.local' \
  --exclude='./packaging/arch/*.tar.gz' --exclude='./tools' \
  -C "$ROOT_DIR" . | tar -xf - -C "$staging/${pkgname}-${pkgver}"
tar -cf - --exclude='./.git' --exclude='./target' --exclude='./dist' \
  -C "$I18N_DIR" . | tar -xf - -C "$staging/argvus-i18n"
tar -czf "$archive" -C "$staging" "${pkgname}-${pkgver}" argvus-i18n

cp "$PKGBUILD" "$PKGBUILD_DIR/PKGBUILD.local"
trap 'rm -f "$PKGBUILD_DIR/PKGBUILD.local"; rm -rf "$staging"' EXIT
sha256="$(sha256sum "$archive" | awk '{print $1}')"
sed -i "s/^sha256sums=.*/sha256sums=(\"${sha256}\")/" "$PKGBUILD_DIR/PKGBUILD.local"

if [[ -n "${MAKEPKG_FLAGS:-}" ]]; then
  # shellcheck disable=SC2206
  flags=(${MAKEPKG_FLAGS})
else
  flags=(--nodeps --noconfirm --needed --cleanbuild --clean --force --check)
fi
cd "$PKGBUILD_DIR"
export BUILDDIR="$ARTIFACTS_DIR" SRCDEST="$ARTIFACTS_DIR" PKGDEST="$DIST_DIR"
makepkg -p PKGBUILD.local "${flags[@]}" "$@"
printf 'Packages created in %s:\n' "$DIST_DIR"
find "$DIST_DIR" -maxdepth 1 -type f -name "${pkgname}-*.pkg.tar.zst" -printf '  %f\n' | sort
