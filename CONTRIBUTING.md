# Contributing to Procedural Lightning VFX

Thank you for your interest in contributing to this project! We welcome contributions from the community.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue on GitHub with:

- A clear description of the problem
- Steps to reproduce the issue
- Expected vs actual behavior
- Your environment (OS, Rust version, Bevy version)
- Screenshots or videos if applicable

### Suggesting Features

Feature requests are welcome! Please open an issue with:

- A clear description of the feature
- Why it would be useful
- Examples of how it might work
- Any implementation ideas you have

### Pull Requests

We love pull requests! Here's the process:

1. **Fork the repository** and create a branch from `main`
2. **Make your changes** following the coding standards below
3. **Test your changes** - ensure the demo runs and examples work
4. **Update documentation** - update README.md if needed
5. **Commit your changes** with clear, descriptive commit messages
6. **Push to your fork** and submit a pull request

#### Pull Request Guidelines

- **One feature/fix per PR** - keep changes focused
- **Write clear commit messages** - explain what and why
- **Update tests** - add tests for new features
- **Follow code style** - run `cargo fmt` before committing
- **Pass CI checks** - ensure `cargo test` and `cargo clippy` pass
- **Update CONTRIBUTORS.md** - add yourself to the list!

### Branch Protection

The `main` branch is protected and requires:

- At least 1 approving review
- All conversations resolved
- Up-to-date with base branch

Admins can override these protections for emergency fixes.

## Development Setup

### Prerequisites

- Rust 1.82+ (2024 Edition)
- Cargo
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/greysquirr3l/bevy_procedural_lightning.git
cd bevy_procedural_lightning

# Build the project
cargo build

# Run tests
cargo test

# Run the demo
cargo run --example demo --release
```

### Code Style

This project follows standard Rust conventions:

- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Run `cargo clippy` and fix warnings
- **Documentation**: Add doc comments for public APIs
- **Testing**: Add tests for new functionality

### Commit Message Format

We use conventional commit messages:

```text
type(scope): brief description

Longer description if needed

- Bullet points for details
- More context
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:

```text
feat(particles): add corona discharge effect
fix(generation): prevent backward branching
docs(readme): update installation instructions
```

## Project Structure

```text
bevy_procedural_lightning/
├── src/
│   └── lib.rs          # Core library code
├── examples/
│   └── demo.rs         # Interactive demo
├── assets/             # Demo assets
├── .github/            # GitHub config & workflows
├── Cargo.toml          # Dependencies
└── README.md           # Documentation
```

## Code Review Process

1. **Submit PR** - open a pull request with clear description
2. **Automated checks** - CI runs tests and linters
3. **Review** - maintainer reviews code and provides feedback
4. **Revisions** - address feedback and push updates
5. **Approval** - once approved, PR will be merged
6. **Cleanup** - branch is automatically deleted after merge

## Getting Help

- **Issues**: Open a GitHub issue for questions
- **Discussions**: Use GitHub Discussions for broader topics
- **Code**: Check the existing examples and tests

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for everyone.

### Expected Behavior

- Be respectful and constructive
- Accept feedback gracefully
- Focus on what's best for the community
- Show empathy towards others

### Unacceptable Behavior

- Harassment or discrimination
- Trolling or insulting comments
- Public or private harassment
- Publishing others' private information

### Enforcement

Violations may result in:

1. Warning
2. Temporary ban from participation
3. Permanent ban from the project

Report issues to the project maintainers.

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT/Apache-2.0 dual license).

## Recognition

All contributors will be recognized in CONTRIBUTORS.md. Thank you for helping make this project better!

## Questions?

Don't hesitate to ask! Open an issue or discussion if anything is unclear.

---

**Thank you for contributing!** 🎉
