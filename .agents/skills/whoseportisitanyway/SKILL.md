```markdown
# whoseportisitanyway Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill covers the development patterns and conventions used in the `whoseportisitanyway` Rust repository. It documents file naming, import/export styles, commit message conventions, and testing patterns, providing clear examples and suggested commands for common workflows. This guide is intended for contributors who want to maintain consistency and quality in the codebase.

## Coding Conventions

### File Naming
- Use **camelCase** for file names.
  - Example: `portScanner.rs`, `userInputHandler.rs`

### Import Style
- Use **relative imports** within the project.
  - Example:
    ```rust
    mod utils;
    use crate::utils::parsePort;
    ```

### Export Style
- Use **named exports** for modules and functions.
  - Example:
    ```rust
    pub fn parsePort(port_str: &str) -> Result<u16, ParseIntError> { ... }
    ```

### Commit Message Conventions
- Follow **conventional commit** format.
- Supported prefixes: `feat`, `fix`, `docs`, `ci`
- Keep commit messages concise (average: 50 characters).
  - Example:
    ```
    feat: add port range validation to input parser
    fix: correct off-by-one error in port scanner
    docs: update README with usage instructions
    ci: add Rustfmt check to workflow
    ```

## Workflows

### Commit Code Changes
**Trigger:** When making any code changes that need to be committed.
**Command:** `/commit`

1. Stage your changes:
    ```
    git add .
    ```
2. Write a commit message using the conventional commit format:
    ```
    git commit -m "feat: add port range validation to input parser"
    ```
3. Push your changes:
    ```
    git push
    ```

### Add Documentation
**Trigger:** When updating or adding documentation.
**Command:** `/docs-update`

1. Edit or add documentation files (e.g., `README.md`).
2. Commit your changes with a `docs:` prefix:
    ```
    git commit -m "docs: update usage section in README"
    ```
3. Push your changes.

### Continuous Integration (CI) Updates
**Trigger:** When modifying CI configuration or scripts.
**Command:** `/ci-update`

1. Edit CI configuration files (e.g., `.github/workflows/*`).
2. Commit your changes with a `ci:` prefix:
    ```
    git commit -m "ci: add Rustfmt check to workflow"
    ```
3. Push your changes.

## Testing Patterns

- Test files use the `*.test.*` pattern (e.g., `portScanner.test.rs`).
- The specific test framework is not detected, but use Rust's built-in test module conventions.
  - Example:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_port_valid() {
            assert_eq!(parsePort("8080"), Ok(8080));
        }
    }
    ```
- Place tests in the same file as the code or in separate `*.test.rs` files.

## Commands
| Command       | Purpose                                         |
|---------------|-------------------------------------------------|
| /commit       | Commit code changes using conventional commits   |
| /docs-update  | Update or add documentation                     |
| /ci-update    | Update continuous integration configuration     |
```
