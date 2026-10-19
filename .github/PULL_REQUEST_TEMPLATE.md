## Summary

<!-- What user-visible problem does this change solve? -->

## Related issue

<!-- Use `Closes #123` or `Related to #123`, or explain why there is no issue. -->

## Change checklist

- [ ] I kept the change focused and did not modify unrelated functionality.
- [ ] I added or updated behavior tests where appropriate.
- [ ] I updated documentation or examples when the public workflow changed.
- [ ] I considered public API, semver, dependency, and platform impact.
- [ ] I did not claim Windows or macOS support without native validation.

## Checks run

<!-- List exact commands. Mention checks that could not be run and why. -->

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
```

## UI evidence

<!-- Add screenshots or a short recording for visual changes, when useful. -->

## Remaining limitations

<!-- Note follow-up work, known risks, or platform-specific gaps. -->
