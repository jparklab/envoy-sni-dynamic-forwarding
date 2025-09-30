all:
	env GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o tcp-plugin.wasm main.go
	# env GOOS=wasip1 GOARCH=wasm go build -ldflags="-s -w" -o tcp-plugin.wasm main.go

clean:
	rm -f tcp-plugin.wasm

.PHONY: all clean
