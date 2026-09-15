PREFIX ?= /usr
DESTDIR ?=
INSTALL ?= install
RM ?= rm -f
CARGO ?= cargo

BIN_NAME := argvus-theme-splash
BIN := target/release/$(BIN_NAME)

.DEFAULT_GOAL := help

.PHONY: help build build-bin check lint fmt fmt-check validate validate-pkgbuild install uninstall reinstall clean

help:
	@echo "Available targets:"
	@echo "  make build"
	@echo "  make build-bin"
	@echo "  make check"
	@echo "  make fmt"
	@echo "  make fmt-check"
	@echo "  make validate"
	@echo "  make validate-pkgbuild"
	@echo "  make install"
	@echo "  make uninstall"
	@echo "  make reinstall"
	@echo "  make clean"

build:
	@tools/build-local-package.sh

build-bin:
	$(CARGO) build --release --locked

check:
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings
	$(CARGO) test --locked

lint: check

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --check

validate: fmt-check check validate-pkgbuild

validate-pkgbuild:
	@if command -v makepkg >/dev/null 2>&1; then \
		cd packaging/arch && makepkg -p PKGBUILD --printsrcinfo >/dev/null && makepkg -p PKGBUILD.local --printsrcinfo >/dev/null; \
	else \
		echo "makepkg not found; skipping PKGBUILD syntax validation"; \
	fi

install: build-bin
	$(INSTALL) -Dm755 "$(BIN)" "$(DESTDIR)$(PREFIX)/bin/$(BIN_NAME)"
	$(INSTALL) -Dm644 LICENSE "$(DESTDIR)$(PREFIX)/share/licenses/argvus-theme-splash/LICENSE"

uninstall:
	$(RM) "$(DESTDIR)$(PREFIX)/bin/$(BIN_NAME)"
	rm -rf "$(DESTDIR)$(PREFIX)/share/licenses/argvus-theme-splash"

reinstall: uninstall install

clean:
	$(CARGO) clean
	rm -rf dist
	rm -f packaging/arch/*.zst packaging/arch/*.tar.gz