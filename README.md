# What The Commit CLI

[whatthecommit.com](https://whatthecommit.com) but local.

## Installation

```bash
cargo install whatthecommitcli
```

## Usage

### Default

Generate a random commit message using built-in defaults:

```bash
whatthecommitcli
```

Use it directly with git:

```bash
git commit -m "$(whatthecommitcli)"
```

### Customization Options

```bash
Usage: whatthecommitcli [OPTIONS]

Options:
  -n, --names <FILE>                     Optional path to a custom names file
  -c, --commit-messages-template <FILE>  Optional path to a custom commit messages template file
  -h, --help                             Print help
  -V, --version                          Print version
```

### Template Placeholders

The commit message templates support three placeholders for name substitution:

- `XNAMEX` - Replaces with the name as-is (e.g., "John")
- `XLOWERNAMEX` - Replaces with lowercase version (e.g., "john")
- `XUPPERNAMEX` - Replaces with uppercase version (e.g., "JOHN")

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).

The default [commit messages](src/commit_messages.txt) and [names](src/names.txt)
are sourced from https://github.com/ngerakines/commitment.
