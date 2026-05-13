# CI/CD Workflows for snapd-rs

This directory contains GitHub Actions workflows that automate testing and quality checks for the snapd-rs project.

## Available Workflows

### 1. **ci.yml** - Continuous Integration
Runs on every push and pull request to validate the codebase.

**Jobs:**
- **test** - Runs the full test suite on stable and beta Rust versions
- **integration-tests** - Runs integration tests with verbose output
- **lint** - Checks code formatting and runs clippy linter
- **coverage** - Generates code coverage reports and uploads to codecov
- **build-check** - Verifies the project compiles on stable and nightly
- **docs** - Ensures documentation builds without warnings

**Triggers:**
- `push` to `main`, `doardo/**`, `artie/**` branches
- `pull_request` to `main`, `artie/**` branches
- Only when `snapd-rs/` directory changes

### 2. **security.yml** - Security Auditing
Checks for known vulnerabilities in dependencies.

**Jobs:**
- **audit** - Runs `cargo audit` to find security vulnerabilities

**Triggers:**
- `push` to `main`, `doardo/**`, `artie/**` branches (on Cargo changes)
- `pull_request` to `main`, `artie/**` branches
- Weekly schedule (Sundays at 00:00 UTC)

## Running Tests Locally

Before pushing, run tests locally:

```bash
# Run all tests
cd snapd-rs
cargo test

# Run integration tests with output
cargo test --test integration_tests -- --nocapture

# Run with specific toolchain
cargo +stable test
cargo +beta test
cargo +nightly test

# Check code formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings

# Generate coverage
cargo tarpaulin --out Html
```

## Viewing Results

- **GitHub Actions**: https://github.com/canonical/test-team-please-ignore/actions
- **Pull Request**: Check the "Checks" tab on your PR
- **Code Coverage**: View in PR or codecov.io dashboard

## Environment Variables

The workflows use these environment variables:
- `RUST_BACKTRACE=1` - Show full backtraces on panics
- `CARGO_TERM_COLOR=always` - Colorize cargo output

## Cache

Cargo build artifacts are cached using `Swatinem/rust-cache@v2` for faster builds.

## Troubleshooting

### Tests fail locally but pass in CI
- Try running with `RUST_BACKTRACE=full`
- Check the Rust version: `rustc --version`
- Clear local cache: `cargo clean && cargo test`

### Linting errors
- Run `cargo fmt` to auto-fix formatting
- Check clippy suggestions: `cargo clippy --fix`

### Coverage not uploading
- Ensure codecov token is configured in repository settings
- Check workflow logs for error details

## Adding New Workflows

To add a new workflow:
1. Create a new `.yml` file in `.github/workflows/`
2. Define triggers, jobs, and steps
3. Test locally with `act` (GitHub Actions local runner)
4. Commit and push to validate

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Toolchain Action](https://github.com/dtolnay/rust-toolchain)
- [Cargo Audit Action](https://github.com/rustsec/audit-check-action)
- [Codecov Action](https://github.com/codecov/codecov-action)
