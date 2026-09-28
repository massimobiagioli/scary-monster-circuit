.DEFAULT_GOAL := help

CARGO    ?= cargo
CHIP     ?= RP2040
FIRMWARE ?= firmware
HOST     := $(shell rustc -vV | sed -n 's/host: //p')
LLVM_BIN := $(shell rustc --print sysroot)/lib/rustlib/$(HOST)/bin

SCHEMATICS := $(wildcard schematics/*.md)
SCHEMATICS_PDF := $(SCHEMATICS:.md=.pdf)

.PHONY: help upload-firmware logs test lint lint-fix coverage schematics breadboard-images

help: ## Show this help
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

upload-firmware: ## Build and flash the firmware onto the device
	cd $(FIRMWARE) && $(CARGO) run --release

logs: ## Open the device logs over RTT
	probe-rs attach --chip $(CHIP)

test: ## Run the unit tests on the host
	cd $(FIRMWARE) && $(CARGO) test --lib --target $(HOST)

lint: ## Run the linter on the firmware and on the unit tests
	cd $(FIRMWARE) && $(CARGO) clippy -- -D warnings
	cd $(FIRMWARE) && $(CARGO) clippy --tests --target $(HOST) -- -D warnings

lint-fix: ## Fix lint problems automatically
	cd $(FIRMWARE) && $(CARGO) clippy --fix --allow-dirty --allow-staged
	cd $(FIRMWARE) && $(CARGO) fmt


coverage: ## Report unit-test coverage (needs `rustup component add llvm-tools-preview`)
	cd $(FIRMWARE) && RUSTFLAGS="-C instrument-coverage" \
		$(CARGO) test --lib --target $(HOST) --no-run --message-format=json > target/coverage.json
	cd $(FIRMWARE) && sed -n 's/.*"executable":"\([^"]*\)".*/\1/p' target/coverage.json \
		| tail -1 > target/coverage.bin
	cd $(FIRMWARE) && rm -f target/coverage.profdata && rm -f target/coverage-*.profraw
	cd $(FIRMWARE) && LLVM_PROFILE_FILE="$(CURDIR)/$(FIRMWARE)/target/coverage-%p-%m.profraw" \
		"$$(cat target/coverage.bin)"
	cd $(FIRMWARE) && $(LLVM_BIN)/llvm-profdata merge -sparse target/coverage-*.profraw \
		-o target/coverage.profdata
	cd $(FIRMWARE) && $(LLVM_BIN)/llvm-cov report "$$(cat target/coverage.bin)" \
		-instr-profile=target/coverage.profdata \
		--ignore-filename-regex='(registry|rustlib|/lib/(mod|test_support)\.rs)' \
		--sources src/

schematics: breadboard-images $(SCHEMATICS_PDF) ## Render every schematics Markdown file into a PDF

breadboard-images: ## Draw the breadboard assembly pictures
	tools/breadboard/breadboard schematics/images

schematics/%.pdf: schematics/%.md
	tools/md2pdf/md2pdf $< $@
