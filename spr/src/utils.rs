/*
 * Copyright (c) Radical HQ Limited
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
    error::{Error, Result},
    message,
};

use git2::Oid;
use std::{io::Write, process::Stdio};
use unicode_normalization::UnicodeNormalization;

pub fn slugify(s: &str) -> String {
    s.trim()
        .nfd()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .filter(|c| c.is_ascii_alphanumeric() || c == &'_' || c == &'-')
        .map(|c| char::to_ascii_lowercase(&c))
        .scan(None, |last_char, char| {
            if char == '-' && last_char == &Some('-') {
                Some(None)
            } else {
                *last_char = Some(char);
                Some(Some(char))
            }
        })
        .flatten()
        .collect()
}

pub fn parse_name_list(text: &str) -> Vec<String> {
    lazy_regex::regex!(r#"\(.*?\)"#)
        .replace_all(text, ",")
        .split(',')
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .map(String::from)
        .collect()
}

/*
 * Given a PR stack string that looks like:
 *
 * ```
 * https://github.com/mk1123/spr/pull/1 <-- (current PR)
 * https://github.com/mk1123/spr/pull/2
 * https://github.com/mk1123/spr/pull/3
 * ```
 *
 * Returns a vector of PR numbers.
 */
pub fn parse_pr_stack_list(text: &str) -> Vec<u64> {
    text.lines()
        .filter_map(|line| {
            line.split_whitespace()
                .next()
                .and_then(|url| url.split('/').last())
                .and_then(|num| num.parse().ok())
        })
        .collect()
}

pub fn remove_all_parens(text: &str) -> String {
    lazy_regex::regex!(r#"[()]"#).replace_all(text, "").into()
}

pub async fn run_command(cmd: &mut tokio::process::Command) -> Result<()> {
    let cmd_output = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .await?;

    if !cmd_output.status.success() {
        console::Term::stderr().write_all(&cmd_output.stderr)?;
        return Err(Error::new("command failed"));
    }

    Ok(())
}

pub fn get_pr_stack(
    git: &crate::git::Git,
    config: &crate::config::Config,
    pull_request_number: u64,
    parent_oid: Oid,
    cherry_pick: bool,
    directly_based_on_master: bool,
) -> Result<String> {
    if cherry_pick || directly_based_on_master {
        Ok(message::build_pr_stack_message(
            &vec![pull_request_number],
            &config.owner,
            &config.repo,
        ))
    } else {
        let mut pr_stack = git.parse_pr_stack_from_commit(parent_oid)?;
        pr_stack.insert(0, pull_request_number);
        Ok(message::build_pr_stack_message(
            &pr_stack,
            &config.owner,
            &config.repo,
        ))
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(slugify(""), "".to_string());
    }

    #[test]
    fn test_hello_world() {
        assert_eq!(slugify(" Hello  World! "), "hello-world".to_string());
    }

    #[test]
    fn test_accents() {
        assert_eq!(slugify("ĥêlļō ŵöřľď"), "hello-world".to_string());
    }

    #[test]
    fn test_parse_name_list_empty() {
        assert!(parse_name_list("").is_empty());
        assert!(parse_name_list(" ").is_empty());
        assert!(parse_name_list("  ").is_empty());
        assert!(parse_name_list("   ").is_empty());
        assert!(parse_name_list("\n").is_empty());
        assert!(parse_name_list(" \n ").is_empty());
    }

    #[test]
    fn test_parse_name_single_name() {
        assert_eq!(parse_name_list("foo"), vec!["foo".to_string()]);
        assert_eq!(parse_name_list("foo  "), vec!["foo".to_string()]);
        assert_eq!(parse_name_list("  foo"), vec!["foo".to_string()]);
        assert_eq!(parse_name_list("  foo  "), vec!["foo".to_string()]);
        assert_eq!(parse_name_list("foo (Foo Bar)"), vec!["foo".to_string()]);
        assert_eq!(
            parse_name_list("  foo (Foo Bar)  "),
            vec!["foo".to_string()]
        );
        assert_eq!(
            parse_name_list(" () (-)foo (Foo Bar)  (xx)"),
            vec!["foo".to_string()]
        );
    }

    #[test]
    fn test_parse_name_multiple_names() {
        let expected =
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        assert_eq!(parse_name_list("foo,bar,baz"), expected);
        assert_eq!(parse_name_list("foo, bar, baz"), expected);
        assert_eq!(parse_name_list("foo , bar , baz"), expected);
        assert_eq!(
            parse_name_list("foo (Mr Foo), bar (Ms Bar), baz (Dr Baz)"),
            expected
        );
        assert_eq!(
            parse_name_list(
                "foo (Mr Foo) bar (Ms Bar) (the other one), baz (Dr Baz)"
            ),
            expected
        );
    }

    #[test]
    fn test_parse_pr_stack_list_empty() {
        assert!(parse_pr_stack_list("").is_empty());
        assert!(parse_pr_stack_list("\n").is_empty());
    }

    #[test]
    fn test_parse_pr_stack_list_single_pr() {
        assert_eq!(
            parse_pr_stack_list(
                "https://github.com/mk1123/spr/pull/42 <-- (current PR)"
            ),
            vec![42]
        );
    }

    #[test]
    fn test_parse_pr_stack_list_multiple_prs() {
        assert_eq!(
            parse_pr_stack_list(
                "https://github.com/mk1123/spr/pull/1 <-- (current PR)\n\
                 https://github.com/mk1123/spr/pull/2\n\
                 https://github.com/mk1123/spr/pull/3"
            ),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn test_parse_pr_stack_list_with_extra_text() {
        assert_eq!(
            parse_pr_stack_list(
                "https://github.com/mk1123/spr/pull/1 <-- (current PR)\n\
                 https://github.com/mk1123/spr/pull/2 (some extra text)\n\
                 https://github.com/mk1123/spr/pull/3 [more text]"
            ),
            vec![1, 2, 3]
        );
    }
}
