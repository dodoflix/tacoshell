# 1. Projects programming language

Date: 2026-02-14

## Status

Accepted

## Context

_This project will be an open-source SSH Client. We want to choose a programming language that is suitable for building a
performant and secure SSH client while also being accessible to contributors. Eligible languages include Go, Rust, and
Python, among others. Each language has its own strengths and weaknesses in terms of performance, security, and ease of
use._

## Decision

After evaluating several programming languages, we have decided to use Go for this project. Go is known for its
performance, concurrency support, and strong community, making it a good fit for building an SSH client.

## Consequences

Using Go will allow us to leverage its standard library for networking and security, which can help us build a robust
SSH client. Additionally, Go's simplicity and readability will make it easier for contributors to understand and
contribute to the codebase. However, we may face some limitations in terms of available libraries for certain features,
and we will need to ensure that we follow the best practices for security and performance when implementing our SSH
client in Go.
