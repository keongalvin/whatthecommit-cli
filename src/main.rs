use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use rand::prelude::IndexedRandom;
use rand::Rng;
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

    Ok(substitute_placeholders(template, name))
}

fn substitute_placeholders(template: &str, name: &str) -> String {
    // Apply all substitutions in order of specificity
    template
        .replace("XUPPERNAMEX", &name.to_ascii_uppercase())
        .replace("XLOWERNAMEX", &name.to_ascii_lowercase())
        .replace("XNAMEX", name)
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
    use crate::*;

    #[test]
    fn t_substitute_single_placeholder() {
        let original = "Fixed a bug cause XNAMEX said to";
        let expected = "Fixed a bug cause John said to";
        assert_eq!(substitute_placeholders(original, "John"), expected);
    }

    #[test]
    fn t_substitute_upper_placeholder() {
        let original = "XUPPERNAMEX, WE WENT OVER THIS. CHECK WHAT COPILOT PRODUCES FIRST.";
        let expected = "ALEX, WE WENT OVER THIS. CHECK WHAT COPILOT PRODUCES FIRST.";
        assert_eq!(substitute_placeholders(original, "Alex"), expected);
    }

    #[test]
    fn t_substitute_lower_placeholder() {
        let original = "blame it on XLOWERNAMEX";
        let expected = "blame it on john";
        assert_eq!(substitute_placeholders(original, "John"), expected);
    }

    #[test]
    fn t_substitute_multiple_placeholders() {
        let original = "XNAMEX told XLOWERNAMEX that XUPPERNAMEX was wrong";
        let expected = "Bob told bob that BOB was wrong";
        assert_eq!(substitute_placeholders(original, "Bob"), expected);
    }

    #[test]
    fn t_no_placeholders() {
        let original = "This is just a regular commit message";
        let expected = "This is just a regular commit message";
        assert_eq!(substitute_placeholders(original, "John"), expected);
    }
}
