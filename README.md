# Depup
Depup is a tool for mass updating dependencies and creating backups. It can take a `package.json` file and update all dependencies to the latest version, and it can do the same for other package types like `Cargo.toml` and `composer.json`. It can also create backups of the original files before updating them. This is useful for keeping track of changes and reverting to previous versions if needed.

# Features
- Update all dependencies to the latest version
- Create backups of original files
- Support for multiple package types (npm, Cargo, composer)
- Easy to use command line interface
- Reverse changes using backup files

# Installation
You can install Depup using Cargo:

```bash
cargo install depup
```

# Usage
To use Depup, run:

```bash
depup -h
```
This will display the help message with all available options.
