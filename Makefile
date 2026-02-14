.PHONY: build test run clean

build:
	go build -o bin/tacoshell cmd/tacoshell/main.go

test:
	go test ./...

run:
	go run cmd/tacoshell/main.go

clean:
	rm -rf bin/

