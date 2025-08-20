# Contributing to rucat

First off, thank you for considering contributing to `rucat`! We welcome any
contributions that help make this tool better. Whether it's reporting a bug,
proposing a new feature, or submitting a pull request, your help is appreciated.

## How to Contribute

We encourage you to open an issue before starting work on any significant
changes. This allows us to discuss the potential impact and design of the
change, ensuring it aligns with the project's goals and preventing wasted
effort.

For small fixes (typos, bug fixes), feel free to submit a pull request directly.

## Pull Request Process

1. **Fork the repository** and create your branch from `main`.
1. **Make your changes**. Please ensure your code follows the existing style.
1. **Ensure all checks pass locally**. Before submitting your pull request,
   please run our comprehensive local CI checks to ensure your changes don't
   break anything.
   ```bash
   # Run all linting, security, and test checks
   make ci
   ```
1. **Update documentation** if you have changed the public-facing API or
   behavior. This includes the `README.md` and any relevant help text.
1. **Regenerate assets** if you have changed the command-line interface.
   ```bash
   make generate-assets
   ```
1. **Commit your work** using the Conventional Commits standard. This practice
   is vital as it helps us automate changelog generation and versioning. Your
   commit message should follow the format:
   ```
   <type>(<scope>): <description>
   ```
   Common types include `feat`, `fix`, `docs`, `style`, `refactor`, `test`, and
   `ci`. For example: `feat(cli): add new --output-file flag` or
   `fix(formatters): correct padding in ansi mode`. Please refer to the
   [Conventional Commits specification](https://www.conventionalcommits.org/en/v1.0.0/)
   for more details.
1. **Submit a Pull Request** to the `main` branch of the `rucat` repository.
   Provide a clear description of the changes in your PR.

## Reporting Bugs

If you find a bug, please open an issue on our GitHub issue tracker. Please
provide as much detail as possible to help us reproduce and fix the issue:

- A clear and descriptive title.
- Your operating system and version.
- The version of `rucat` you are using (`rucat --version`).
- Steps to reproduce the bug.
- The expected behavior and what actually happened.
- Any relevant error messages or logs.

## Suggesting Enhancements

If you have an idea for a new feature or an improvement to an existing one,
please open an issue to discuss it. We are always open to new ideas that align
with the project's vision.

## Code of Conduct

While this project does not have a formal Code of Conduct, we expect all
contributors to interact in a respectful and constructive manner. Please be
considerate of others in all discussions and contributions.
