# tacoshell

Tacoshell is an open-source SSH Client.

## About

Tacoshell is designed to provide a seamless and efficient SSH experience for users. It offers a user-friendly interface,
robust security features, and support for various platforms. Whether you're a developer, system administrator, or anyone
who needs to manage remote servers, Tacoshell aims to be your go-to SSH client.

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed phases and future plans.

## Documentation

- [Architecture Decision Records (ADR)](docs/adr/0001-project-programming-language.md)
- [Design Documents](docs/README.md)

## Project Structure

This project follows the [standard Go project layout](https://github.com/golang-standards/project-layout):

- `cmd/`: Main applications for this project.
- `internal/`: Private application and library code.
- `pkg/`: Library code that's ok to use by external applications.
- `api/`: OpenAPI/Swagger specs, JSON schema files, protocol definition files.
- `configs/`: Configuration file templates or default configs.
- `test/`: Additional external test apps and test data.
- `docs/`: Design and user documents.
- `build/`: Packaging and Continuous Integration.
- `scripts/`: Scripts to perform various build, install, analysis, etc operations.

## Usage

make run

