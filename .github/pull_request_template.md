## Summary

<!-- What does this PR do? Why is it needed? -->

## Type of Change

- [ ] `feat` — new feature
- [ ] `fix` — bug fix
- [ ] `test` — tests only
- [ ] `refactor` — no behaviour change
- [ ] `docs` — documentation only
- [ ] `chore` — build / tooling / config

## Checklist

- [ ] Tests written before production code (TDD)
- [ ] `cargo test --workspace` passes
- [ ] `pnpm -r run test` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `pnpm -r run lint` passes
- [ ] No `unwrap()` / `expect()` in non-test Rust code
- [ ] No `any` casts in TypeScript
- [ ] `cargo audit` passes (if dependencies changed)
- [ ] `TODO.md` updated (if completing a tracked task)
- [ ] Security implications considered (see `docs/SECURITY.md`)

## Related Issues

<!-- Closes # -->
