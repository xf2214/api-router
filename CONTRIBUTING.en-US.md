# Contributing Guide

<p align="center">
  <a href="CONTRIBUTING.md">中文</a> | English
</p>

Thank you for your interest in API Router! We welcome all forms of contribution, including but not limited to:

- Filing Issues to report bugs or suggest features
- Submitting Pull Requests to fix issues or implement features
- Improving documentation
- Sharing usage experience

## Code of Conduct

Please read and follow the community standards in [CODE_OF_CONDUCT.en-US.md](CODE_OF_CONDUCT.en-US.md). Be kind, respectful, and inclusive.

## Getting Started

1. Fork this repository.
2. Clone your fork locally.
3. Create a new branch: `git checkout -b feat/your-feature-name` or `fix/your-bug-description`.
4. Set up your development environment following the instructions in [README.en-US.md](README.en-US.md).

## Development Conventions

### Commit Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. Commit messages should use the following format:

```
<type>(<scope>): <subject>

<body>
```

Common `type` values:

- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `test`: Tests
- `docs`: Documentation
- `chore`: Build / tooling
- `perf`: Performance optimization

Example:

```
feat(router): add weighted backend selection

Implement weighted random strategy for multi-backend routing.
```

### Rust Code Guidelines

- Use `snake_case` for functions and variables, `PascalCase` for types and enums, `SCREAMING_SNAKE_CASE` for constants.
- Propagate errors with `?`; custom error types live in `src-tauri/src/error.rs`.
- Every `async` function must be cancellation-safe.
- Sensitive data must be wrapped in `secrecy::SecretString`; never print secrets via `println!` / `dbg!`.
- Avoid `unsafe` blocks; if absolutely necessary, annotate each line explaining why it is required.

### TypeScript / Vue Code Guidelines

- Use `camelCase` for variables and functions, `PascalCase` for components and types, `UPPER_SNAKE_CASE` for constants.
- Explicitly annotate return types for all variables and functions; `any` is forbidden.
- Component files must use Single File Components (`.vue`).
- Encapsulate API calls under `src/services/`; never call `fetch` directly inside components.
- Use the unified Toast / Notification wrapper for user-facing errors; raw `alert` is forbidden.

## Quality Gates

Before submitting a PR, ensure all of the following checks pass locally:

```bash
# Rust
cd src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Frontend
cd ..
npm run type-check
npm run build
```

## Submitting a Pull Request

1. Make sure your branch is rebased on top of the latest `main`.
2. In the PR description, clearly explain what changed, why it changed, and how it was tested.
3. Link any related Issues.
4. Wait for a maintainer review.

## Reporting Bugs

When filing an Issue, please include as much of the following as possible:

- Operating system and version
- Rust version: `rustc --version`
- Node.js version: `node --version`
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or screenshots

## Suggesting Features

Feature suggestions are welcome! Please describe in the Issue:

- Use case and motivation
- Expected interface or behavior
- Whether you are willing to implement it yourself

## Licensing

By submitting code to this project, you agree that your contributions will be released under the [MIT License](LICENSE).
