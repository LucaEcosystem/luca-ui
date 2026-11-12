<!--
External code contributions are not accepted for Luca UI.

Luca UI is proprietary software (see LICENSE and CONTRIBUTING.md). The
maintainers do not merge pull requests from outside contributors. Please use
GitHub Issues (bug report / feature request) or Discussions instead.

If you are a maintainer preparing a branch for review, delete this notice and
fill in the sections below.
-->

## Summary

<!-- What does this change do, and why? Link the issue or design note. -->

## Type of change

- [ ] Bug fix (non-breaking)
- [ ] New behavior or capability
- [ ] Documentation only
- [ ] Refactor / internal (no user-visible change)
- [ ] Other: <!-- describe -->

## Compatibility

- [ ] No `.lucu`/`.lucc` language behavior changed
- [ ] Language behavior changed — documented and justified below
- [ ] Diagnostics wording unchanged (frozen catalog)
- [ ] New diagnostics only for new behavior (catalog updated)

## One-way dependency

- [ ] No change to Luca Code
- [ ] Luca UI still depends on Luca Code only; no reverse dependency introduced

## Checks

- [ ] `cargo fmt --check` passes
- [ ] `cargo test --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo doc --no-deps` passes
- [ ] Examples and `luca-ui check` / `render` smoke-tested
- [ ] Docs updated where behavior changed
- [ ] `docs/CHANGELOG.md` entry added under `Unreleased`

## Notes for reviewers

<!-- Anything that needs special attention: assumptions (◐), open items (○), or follow-up work. -->
