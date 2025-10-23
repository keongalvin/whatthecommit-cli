use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use rand::prelude::IndexedRandom;
use rand::Rng;
use regex_lite::Regex;
use std::fs;
use std::path::PathBuf;

fn default_names() -> Vec<String> {
    include_str!("names.txt")
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn default_commit_messages() -> Vec<String> {
    include_str!("commit_messages.txt")
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional path to a custom names file
    #[arg(short = 'n', long = "names", value_name = "FILE")]
    names: Option<PathBuf>,

    /// Optional path to a custom commit messages template file
    #[arg(short = 'c', long = "commit-messages-template", value_name = "FILE")]
    commit_messages_template: Option<PathBuf>,
}

/// Load lines from a file or return defaults
fn load_lines_or_default(
    file_path: &Option<PathBuf>,
    default_fn: fn() -> Vec<String>,
    file_type: &str,
) -> Result<Vec<String>> {
    match file_path {
        None => {
            debug!("Using default {}", file_type);
            Ok(default_fn())
        }
        Some(path) => {
            debug!("Loading {} from: {:?}", file_type, path);
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read {} file: {:?}", file_type, path))?;

            let lines: Vec<String> = content
                .lines()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .collect();

            if lines.is_empty() {
                anyhow::bail!("{} file is empty or contains only empty lines", file_type);
            }

            info!("Loaded {} {} from {:?}", lines.len(), file_type, path);
            Ok(lines)
        }
    }
}

fn generate_commit_message<R>(
    names: &[String],
    commit_messages: &[String],
    rng: &mut R,
) -> Result<String>
where
    R: Rng + ?Sized,
{
    let name = names.choose(rng).context("Failed to select any names")?;
    let template = commit_messages
        .choose(rng)
        .context("Failed to select any commit messages")?;

    Ok(substitute_placeholders(template, name, rng))
}

/// Parses a number range specification from XNUM...X placeholders.
///
/// # Returns
/// A tuple of (start, end) for the range.
///
/// # Examples
/// - "" -> (1, 999) - default range
/// - "10" -> (1, 10) - simple upper limit
/// - "1,5" -> (1, 5) - explicit range
/// - ",5" -> (1, 5) - range with default start
/// - "5," -> (5, 999) - range with default end
fn parse_number_range(value_str: &str) -> (u32, u32) {
    if value_str.is_empty() {
        // XNUMX - default range
        return (1, 999);
    }

    if !value_str.contains(',') {
        // XNUM10X - simple number, range from 1 to specified value
        let end = value_str.parse::<u32>().unwrap_or(999);
        return (1, end);
    }

    // Handle comma-separated values as ranges
    let comma_pos = value_str.find(',').unwrap(); // Safe because we checked contains above

    if comma_pos == 0 {
        // XNUM,5X - range from 1 to specified end
        let end_str = &value_str[1..];
        let end = end_str.parse::<u32>().unwrap_or(999);
        (1, end)
    } else if comma_pos == value_str.len() - 1 {
        // XNUM5,X - range from specified start to 999
        let start_str = &value_str[..comma_pos];
        let start = start_str.parse::<u32>().unwrap_or(1);
        (start, 999)
    } else {
        // XNUM1,5X - treat as range (start,end)
        let before_comma = &value_str[..comma_pos];
        let after_comma = &value_str[comma_pos + 1..];
        let start = before_comma.parse::<u32>().unwrap_or(1);
        let end = after_comma.parse::<u32>().unwrap_or(999);
        (start, end)
    }
}

/// Generates a random number within the specified range.
///
/// If start > end, automatically adjusts end to start * 2.
fn generate_random_in_range<R>(start: u32, end: u32, rng: &mut R) -> u32
where
    R: Rng + ?Sized,
{
    let final_end = if start > end { start * 2 } else { end };

    if final_end > start {
        rng.random_range(start..=final_end)
    } else {
        start
    }
}

/// Substitutes number placeholders (XNUM...X) in a template string.
fn substitute_number_placeholders<R>(template: &str, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let num_re = Regex::new(r"XNUM([0-9,]*)X").unwrap();

    num_re
        .replace_all(template, |caps: &regex_lite::Captures| {
            let value_str = &caps[1];
            let (start, end) = parse_number_range(value_str);
            let random_num = generate_random_in_range(start, end, rng);
            random_num.to_string()
        })
        .into_owned()
}

/// Substitutes name placeholders in a template string.
fn substitute_name_placeholders(template: &str, name: &str) -> String {
    template
        .replace("XUPPERNAMEX", &name.to_ascii_uppercase())
        .replace("XLOWERNAMEX", &name.to_ascii_lowercase())
        .replace("XNAMEX", name)
}

/// Substitutes placeholders in a template string with actual values.
///
/// # Placeholder Types
///
/// ## Number Placeholders (XNUM...X)
/// Generates random numbers within specified ranges. Supports multiple formats:
///
/// - `XNUMX` - Random number from 1 to 999 (default)
/// - `XNUM10X` - Random number from 1 to 10
/// - `XNUM1,5X` - Random number from 1 to 5 (range syntax with comma)
/// - `XNUM,5X` - Random number from 1 to 5 (start defaults to 1)
/// - `XNUM5,X` - Random number from 5 to 999 (end defaults to 999)
///
/// Note: Commas are always treated as range separators. `XNUM1,000X` means range 1 to 0,
/// which gets adjusted to 1 to 2 (since start > end triggers end = start * 2).
/// Use `XNUM1000X` for 1 to 1000.
///
/// If start > end, end is automatically set to start * 2.
///
/// ## Name Placeholders
/// - `XNAMEX` - Replaced with the name as-is
/// - `XUPPERNAMEX` - Replaced with the name in UPPERCASE
/// - `XLOWERNAMEX` - Replaced with the name in lowercase
///
/// # Arguments
/// * `template` - The template string containing placeholders
/// * `name` - The name to substitute into name placeholders
/// * `rng` - Random number generator for number placeholders
fn substitute_placeholders<R>(template: &str, name: &str, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    // First handle number placeholders
    let with_numbers = substitute_number_placeholders(template, rng);

    // Then apply name substitutions
    substitute_name_placeholders(&with_numbers, name)
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let names = load_lines_or_default(&args.names, default_names, "names")?;
    let commit_messages = load_lines_or_default(
        &args.commit_messages_template,
        default_commit_messages,
        "commit messages",
    )?;

    let mut rng = rand::rng();
    let message = generate_commit_message(&names, &commit_messages, &mut rng)?;

    println!("{}", message);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn t_substitute_single_placeholder() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Fixed a bug cause XNAMEX said to";
        let expected = "Fixed a bug cause John said to";
        assert_eq!(
            substitute_placeholders(original, "John", &mut rng),
            expected
        );
    }

    #[test]
    fn t_substitute_upper_placeholder() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "XUPPERNAMEX, WE WENT OVER THIS. CHECK WHAT COPILOT PRODUCES FIRST.";
        let expected = "ALEX, WE WENT OVER THIS. CHECK WHAT COPILOT PRODUCES FIRST.";
        assert_eq!(
            substitute_placeholders(original, "Alex", &mut rng),
            expected
        );
    }

    #[test]
    fn t_substitute_lower_placeholder() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "blame it on XLOWERNAMEX";
        let expected = "blame it on john";
        assert_eq!(
            substitute_placeholders(original, "John", &mut rng),
            expected
        );
    }

    #[test]
    fn t_substitute_multiple_placeholders() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "XNAMEX told XLOWERNAMEX that XUPPERNAMEX was wrong";
        let expected = "Bob told bob that BOB was wrong";
        assert_eq!(substitute_placeholders(original, "Bob", &mut rng), expected);
    }

    #[test]
    fn t_substitute_number_placeholders() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Fixed XNUM10X bugs";
        let expected = "Fixed 2 bugs";
        assert_eq!(
            substitute_placeholders(original, "John", &mut rng),
            expected
        );
    }

    #[test]
    fn t_substitute_number_with_comma() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Deleted XNUM1,000X lines of code";
        // 1,000 is parsed as range 1 to 0, which becomes 1 to 2 (start*2)
        let result = substitute_placeholders(original, "John", &mut rng);
        let num: u32 = result.split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!(num >= 1 && num <= 2);
        // With seed 42, it should generate either 1 or 2
        assert_eq!(result, "Deleted 1 lines of code");
    }

    #[test]
    fn t_substitute_number_default() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Improved performance by XNUMX%";
        let result = substitute_placeholders(original, "John", &mut rng);
        // With default range 1-999, we need to check what value it actually generates
        assert!(result.contains("Improved performance by "));
        assert!(result.contains("%"));
        // Extract and verify the number is in the valid range
        let num_str = result
            .strip_prefix("Improved performance by ")
            .unwrap()
            .strip_suffix("%")
            .unwrap();
        let num: u32 = num_str.parse().unwrap();
        assert!(num >= 1 && num <= 999);
    }

    #[test]
    fn t_substitute_mixed_placeholders() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "XNAMEX fixed XNUM50X bugs that XLOWERNAMEX found";
        let expected = "Alice fixed 7 bugs that alice found";
        assert_eq!(
            substitute_placeholders(original, "Alice", &mut rng),
            expected
        );
    }

    #[test]
    fn t_no_placeholders() {
        let mut rng = StdRng::seed_from_u64(42);
        let original = "This is just a regular commit message";
        let expected = "This is just a regular commit message";
        assert_eq!(
            substitute_placeholders(original, "John", &mut rng),
            expected
        );
    }

    #[test]
    fn t_substitute_number_range_syntax() {
        // Test XNUM1,5X - range from 1 to 5
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Fixed XNUM1,5X bugs";
        let result = substitute_placeholders(original, "John", &mut rng);
        // Extract the number to verify it's in range
        let num: u32 = result.split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!(num >= 1 && num <= 5);
        assert_eq!(result, "Fixed 1 bugs"); // With seed 42, should be 1
    }

    #[test]
    fn t_substitute_number_range_start_only() {
        // Test XNUM5,X - range from 5 to 999
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Fixed XNUM5,X bugs";
        let result = substitute_placeholders(original, "John", &mut rng);
        // Extract the number to verify it's in range
        let num: u32 = result.split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!(num >= 5 && num <= 999);
    }

    #[test]
    fn t_substitute_number_range_end_only() {
        // Test XNUM,5X - range from 1 to 5
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Fixed XNUM,5X bugs";
        let result = substitute_placeholders(original, "John", &mut rng);
        // Extract the number to verify it's in range
        let num: u32 = result.split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!(num >= 1 && num <= 5);
        assert_eq!(result, "Fixed 1 bugs"); // With seed 42, should be 1
    }

    #[test]
    fn t_substitute_number_inverted_range() {
        // Test when start > end, end should become start * 2
        let mut rng = StdRng::seed_from_u64(42);
        let original = "Fixed XNUM10,5X bugs";
        let result = substitute_placeholders(original, "John", &mut rng);
        // With start=10, end=5, it should become start=10, end=20
        let num: u32 = result.split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!(num >= 10 && num <= 20);
    }
}
