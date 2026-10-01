PREFIX ?= /usr
DESTDIR ?=
INSTALL ?= install
RM ?= rm -f
CARGO ?= cargo

BIN := target/release/splash

.DEFAULT_GOAL := help

.PHONY: help build build-bin check lint lint-shell fmt fmt-check validate validate-pkgbuild install uninstall reinstall clean

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
	@$(MAKE) package

package: check
	@tools/sh/pkgbuild_local.sh

pkg: package

build-bin:
	$(CARGO) build --workspace --release --locked

check:
	$(CARGO) clippy --workspace --locked --all-targets --all-features -- -D warnings
	$(CARGO) test --workspace --locked

lint-shell:
	@for root in tools packaging/arch/common src; do \
		if [ -d "$$root" ]; then \
			find "$$root" -type f -name '*.sh' -exec shellcheck -e SC1090 -e SC2034 -e SC2154 {} +; \
		fi; \
	done
	@for root in tools packaging/arch/common src; do \
		if [ -d "$$root" ]; then \
			find "$$root" -type f -name '*.sh' -exec bash -n {} +; \
		fi; \
	done
	@git diff --check
	@echo "Lint Shell OK"

lint: lint-shell check

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --all -- --check

validate: fmt-check check validate-pkgbuild

validate-pkgbuild:
	@if command -v makepkg >/dev/null 2>&1; then \
		cd packaging/arch/ci && makepkg -p PKGBUILD --printsrcinfo >/dev/null; \
		cd ../local && makepkg -p PKGBUILD --printsrcinfo >/dev/null; \
	else \
		echo "makepkg not found; skipping PKGBUILD syntax validation"; \
	fi

install: build-bin
	@sudo pacman -U build/dist/argvus*.zst --noconfirm --overwrite="*"

uninstall:
	$(RM) "$(DESTDIR)$(PREFIX)/lib/argvus/loading-theme/splash"
	rm -rf "$(DESTDIR)$(PREFIX)/share/licenses/argvus-loading-theme"

reinstall: uninstall install

clean:
	$(CARGO) clean
	rm -rf dist build
