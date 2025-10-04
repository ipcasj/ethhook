# Contributing to EthHook

First off, thank you for considering contributing to EthHook! 🎉

## Code of Conduct

This project and everyone participating in it is governed by our commitment to creating a welcoming and inclusive environment for all contributors.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues. When you create a bug report, include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples**
- **Describe the behavior you observed and what you expected**
- **Include logs and error messages**
- **Specify your environment** (OS, Rust version, etc.)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- **Use a clear and descriptive title**
- **Provide a detailed description of the proposed feature**
- **Explain why this enhancement would be useful**
- **Include examples of how it would work**

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following our coding standards
3. **Add tests** for any new functionality
4. **Ensure all tests pass**: `cargo test`
5. **Run formatting**: `cargo fmt`
6. **Run linting**: `cargo clippy`
7. **Update documentation** if needed
8. **Write a clear commit message** describing your changes

## Development Setup

See [SETUP_GUIDE.md](SETUP_GUIDE.md) for detailed instructions on setting up your development environment.

Quick start:
```bash
# Clone the repository
git clone https://github.com/yourusername/ethhook.git
cd ethhook

# Install dependencies
cargo build

# Run tests
cargo test

# Start services with Docker
docker compose up -d
```

## Coding Standards

### Rust Style Guide

- Follow the [official Rust style guide](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write idiomatic Rust code

### Code Organization

- Keep functions small and focused
- Write descriptive variable and function names
- Add comments for complex logic
- Document public APIs with doc comments (`///`)

### Testing

- Write unit tests for all public functions
- Add integration tests for service interactions
- Aim for >80% code coverage
- Test error cases, not just happy paths

### Commit Messages

Write clear, concise commit messages:

```
Add support for Polygon chain

- Implement Polygon RPC configuration
- Add chain ID 137 to supported chains
- Update documentation with Polygon examples
```

## Project Structure

```
ethhook/
├── crates/
│   ├── common/           # Shared utilities
│   ├── domain/           # Domain models
│   ├── config/           # Configuration
│   ├── event-ingestor/   # Event ingestion service
│   ├── message-processor/# Message processing
│   ├── webhook-delivery/ # Webhook delivery
│   └── admin-api/        # REST API
├── migrations/           # Database migrations
├── docs/                 # Additional documentation
└── monitoring/           # Monitoring configs
```

## Questions?

Feel free to open an issue with your question or reach out to the maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to EthHook!** 🦀🚀
