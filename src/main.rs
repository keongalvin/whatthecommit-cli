use anyhow::Context;
use rand::prelude::IndexedRandom;

fn load_names() -> Vec<String> {
    include_str!("names.txt")
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn load_commit_messages() -> Vec<String> {
    include_str!("commit_messages.txt")
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn main() -> Result<(), anyhow::Error> {
    let mut rng = rand::rng();
    let commit_messages = load_commit_messages();
    let names = load_names();
    let name_of_the_day = names
        .choose(&mut rng)
        .context("failed to select any names")?;
    let msg = commit_messages
        .choose(&mut rng)
        .context("failed to select any commit messages")?;
    let output = if msg.contains("XNAMEX") {
        substitute_name(msg, name_of_the_day)
    } else if msg.contains("XLOWERNAMEX") {
        substitute_lower(msg, name_of_the_day)
    } else if msg.contains("XUPPERNAMEX") {
        substitute_upper(msg, name_of_the_day)
    } else {
        msg.clone()
    };

    println!("{}", output); // output of the program
    Ok(())
}

fn substitute_name<'a>(text: &str, name: &str) -> String {
    text.replace("XNAMEX", name)
}

fn substitute_lower<'a>(text: &str, name: &str) -> String {
    text.replace("XLOWERNAMEX", name.to_ascii_lowercase().as_ref())
}
fn substitute_upper<'a>(text: &str, name: &str) -> String {
    text.replace("XUPPERNAMEX", name.to_ascii_uppercase().as_ref())
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn t_substitute_name() {
        let original = "Fixed a bug cause XNAMEX said to";
        let expected = "Fixed a bug cause John said to";
        assert_eq!(substitute_name(original, "John"), expected);
    }

    #[test]
    fn t_substitute_name_upper() {
        let original = "XUPPERNAMEX, WE WENT OVER THIS. CHECK WHAT COPILOT PRODUCES FIRST.";
        let expected = "ALEX, WE WENT OVER THIS. CHECK WHAT COPILOT PRODUCES FIRST.";
        assert_eq!(substitute_upper(original, "Alex"), expected);
    }
}
