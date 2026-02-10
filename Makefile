.PHONY: dev build build-linux bundle-linux clean

dev:
	cargo tauri dev

build:
	cargo tauri build

build-linux:
	./scripts/build-linux.sh

bundle-linux:
	./scripts/build-linux.sh --bundles appimage,deb,rpm

clean:
	rm -rf target dist
