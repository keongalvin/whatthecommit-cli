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

### Options

```bash
Usage: whatthecommitcli [OPTIONS]

Options:
  -n, --names <FILE>                     Optional path to a custom names file
  -c, --commit-messages-template <FILE>  Optional path to a custom commit messages template file
  -h, --help                             Print help
  -V, --version                          Print version
```

### Template String Instructions

The commit message templates support various placeholders that get replaced with dynamic values:

#### Name Placeholders

Names are randomly selected from the names file and substituted in three formats:

- `XNAMEX` - Replaces with the name as-is (e.g., "John")
- `XLOWERNAMEX` - Replaces with lowercase version (e.g., "john")
- `XUPPERNAMEX` - Replaces with uppercase version (e.g., "JOHN")

#### Number Placeholders (XNUM...X)

Generate random numbers within specified ranges. The parser supports multiple formats:

##### Basic Formats

- `XNUMX` - Random number from 1 to 999 (default)
- `XNUM10X` - Random number from 1 to 10
- `XNUM100X` - Random number from 1 to 100
- `XNUM1000X` - Random number from 1 to 1000

##### Range Syntax (using comma)

Commas are used to specify custom ranges:

- `XNUM1,5X` - Random number from 1 to 5
- `XNUM,5X` - Random number from 1 to 5 (start defaults to 1)
- `XNUM5,X` - Random number from 5 to 999 (end defaults to 999)
- `XNUM10,20X` - Random number from 10 to 20

**Note:** If start > end in a range, the end is automatically adjusted to start × 2.

#### Examples

Template strings can combine multiple placeholders:

```
"XNAMEX fixed XNUM50X bugs"          → "Alice fixed 7 bugs"
"blame it on XLOWERNAMEX"            → "blame it on john"
"XUPPERNAMEX BROKE THE BUILD AGAIN"  → "BOB BROKE THE BUILD AGAIN"
"Improved performance by XNUMX%"     → "Improved performance by 42%"
"XNAMEX deleted XNUM1000X lines"     → "Sarah deleted 834 lines"
"Fixed XNUM1,5X critical issues"     → "Fixed 3 critical issues"
```

### Custom Template Files

You can create your own template files with one template per line:

#### Custom Commit Messages Template

Create a file with commit message templates (one per line):

```text
# my-commits.txt
XNAMEX made things XNUM10X% better
Fixed XNUM1,100X bugs that XLOWERNAMEX introduced
XUPPERNAMEX DEMANDS THIS COMMIT
Reverted XNAMEX's last XNUM5X commits
Made the code XNUM42,X% more readable
```

Use it with:

```bash
whatthecommitcli -c my-commits.txt
```

#### Custom Names File

Create a file with names (one per line):

```text
# my-names.txt
Alice
Bob
Charlie
Diana
Eve
```

Use it with:

```bash
whatthecommitcli -n my-names.txt
```

#### Combining Both

```bash
whatthecommitcli -n my-names.txt -c my-commits.txt
```

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).

The default [commit messages](src/commit_messages.txt) and [names](src/names.txt)
are sourced from https://github.com/ngerakines/commitment.
