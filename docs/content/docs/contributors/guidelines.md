+++
title = "Contributing Guidelines"
description = "How to contribute to the GQLForge project."
+++

# Contributing Guidelines

Thank you for your interest in contributing to GQLForge. This page describes the workflow for submitting changes.

## Prerequisites

- Rust toolchain (stable, latest version recommended)
- Git

Install Rust via rustup if you have not already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Workflow

1. **Fork** the repository on GitHub.

2. **Clone** your fork locally:

```bash
git clone https://github.com/your-username/gqlforge.git
cd gqlforge
```

3. **Create a branch** for your change:

```bash
git checkout -b feature/my-improvement
```

4. **Make your changes** and ensure they compile:

```bash
cargo build
```

5. **Run linting**:

```bash
cargo clippy -p gqlforge
```

Fix any warnings before submitting.

6. **Run the test suite**:

```bash
cargo test
```

7. **Commit** with a clear message describing what and why:

```bash
git commit -m "feat: add support for custom headers in @http directive"
```

8. **Push** and open a pull request against the `main` branch.

## Commit Message Format

Use conventional commit prefixes:

- `feat:` for new features
- `fix:` for bug fixes
- `refactor:` for code restructuring
- `docs:` for documentation changes
- `test:` for test additions or modifications
- `chore:` for maintenance tasks

## Code Style

- Follow standard Rust formatting (`cargo fmt`).
- All public APIs should have doc comments.
- Avoid introducing new clippy warnings.

## Pull Request Checklist

- [ ] Code compiles without warnings
- [ ] All existing tests pass
- [ ] New tests added for new functionality
- [ ] Clippy reports no new warnings
- [ ] Commit messages follow the conventional format
