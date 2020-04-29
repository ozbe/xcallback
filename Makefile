callback.app: $(wildcard src/*.rs) $(wildcard src/**/*.rs) Makefile app
	rm -rf callback.app
	mkdir -p callback.app/Contents/MacOS
	cargo build
	cp target/debug/callback callback.app/Contents/MacOS/
	cp app/Info.plist callback.app/Contents/Info.plist
