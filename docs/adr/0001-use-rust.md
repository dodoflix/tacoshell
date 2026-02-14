# 1. Use Rust as Primary Language

Date: 2025-05-18

## Status

Accepted

## Context

We are initiating the development of the `tacoshell` project. As this is a greenfield project (or a complete rewrite) that has not yet been published to any environment, we have the opportunity to select the most appropriate technology stack without the constraints of legacy code or migration paths.

We need a systems programming language that offers:
*   High performance and low resource footprint.
*   Memory safety to prevent common vulnerabilities (e.g., buffer overflows, null pointer dereferences).
*   Strong concurrency support.
*   A robust tooling ecosystem (package management, build system).

## Decision

We will use **Rust** as the primary programming language for the `tacoshell` project.

## Consequences

### Positive
*   **Memory Safety:** Rust's ownership model guarantees memory safety at compile time without a garbage collector.
*   **Performance:** Comparable to C/C++, making it suitable for high-performance tasks.
*   **Tooling:** `cargo` provides a unified modern experience for dependency management, building, testing, and documentation.
*   **Correctness:** The strict type system and compiler checks catch many classes of bugs before runtime.

### Negative
*   **Learning Curve:** The ownership and borrowing concepts can be difficult for developers coming from garbage-collected languages.
*   **Compilation Time:** Rust compilation times can be slower compared to some other languages like Go.

## Compliance

*   All new core application logic must be written in Rust.
*   The project will utilize `cargo` for build management.
*   Code must pass `clippy` lints and standard formatting via `rustfmt`.