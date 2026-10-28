use std::ffi::OsString;

use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command};

pub(crate) fn render_matches(cmd: &Command, matches: &ArgMatches) -> String {
    let mut out = String::new();
    render_level(cmd, matches, &[], &[], &mut out);
    out
}

fn render_level(
    cmd: &Command,
    matches: &ArgMatches,
    ancestors: &[&Command],
    path: &[String],
    out: &mut String,
) {
    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        if matches!(arg.get_action(), ArgAction::Help | ArgAction::Version) {
            continue;
        }
        if ancestors
            .iter()
            .any(|ancestor| ancestor.get_arguments().any(|other| other.get_id() == id))
        {
            continue;
        }
        write_arg(arg, matches, out);
    }

    match matches.subcommand() {
        Some((name, sub)) => {
            let mut path = path.to_vec();
            path.push(name.to_string());
            if let Some(sub_cmd) = cmd.find_subcommand(name) {
                let mut ancestors = ancestors.to_vec();
                ancestors.push(cmd);
                render_level(sub_cmd, sub, &ancestors, &path, out);
            } else {
                write_set(out, &path, &external_words(sub));
            }
        }
        None => write_set(out, path, &[]),
    }
}

fn write_arg(arg: &Arg, matches: &ArgMatches, out: &mut String) {
    let id = arg.get_id().as_str();
    let global = arg.is_global_set();
    match arg.get_action() {
        ArgAction::Help | ArgAction::Version => {}
        ArgAction::Count => {
            let count = if global {
                best_count(matches, id)
            } else {
                matches
                    .try_get_one::<u8>(id)
                    .ok()
                    .flatten()
                    .copied()
                    .unwrap_or(0)
            };
            push_line(out, &format!("{id}={count}"));
        }
        ArgAction::SetTrue | ArgAction::SetFalse => {
            let value = if global {
                best_flag(matches, id)
            } else {
                matches
                    .try_get_one::<bool>(id)
                    .ok()
                    .flatten()
                    .copied()
                    .unwrap_or(false)
            };
            push_line(out, &format!("{id}={}", u8::from(value)));
        }
        action if action.takes_values() => {
            let values = if global {
                best_strings(matches, id)
            } else {
                current_strings(matches, id)
            };
            if let Some(values) = values {
                push_line(
                    out,
                    &format!("{id}={}", shell_single_quote(&values.join("\n"))),
                );
            }
        }
        _ => {}
    }
}

fn current_strings<'a>(matches: &'a ArgMatches, id: &str) -> Option<Vec<&'a str>> {
    matches
        .try_get_many(id)
        .ok()
        .flatten()
        .map(|values| values.map(String::as_str).collect())
}

fn best_strings<'a>(matches: &'a ArgMatches, id: &str) -> Option<Vec<&'a str>> {
    let mut best: Option<(u8, Vec<&str>)> = None;
    walk_strings(matches, id, &mut best);
    best.map(|(_, values)| values)
}

fn walk_strings<'a>(matches: &'a ArgMatches, id: &str, best: &mut Option<(u8, Vec<&'a str>)>) {
    if let Some(values) = current_strings(matches, id) {
        consider(best, source_rank(matches.value_source(id)), values);
    }
    if let Some((_, sub)) = matches.subcommand() {
        walk_strings(sub, id, best);
    }
}

fn best_count(matches: &ArgMatches, id: &str) -> u8 {
    let mut best: Option<(u8, u8)> = None;
    walk_count(matches, id, &mut best);
    best.map(|(_, value)| value).unwrap_or(0)
}

fn walk_count(matches: &ArgMatches, id: &str, best: &mut Option<(u8, u8)>) {
    if let Some(value) = matches.try_get_one::<u8>(id).ok().flatten().copied() {
        consider(best, source_rank(matches.value_source(id)), value);
    }
    if let Some((_, sub)) = matches.subcommand() {
        walk_count(sub, id, best);
    }
}

fn best_flag(matches: &ArgMatches, id: &str) -> bool {
    let mut best: Option<(u8, bool)> = None;
    walk_flag(matches, id, &mut best);
    best.map(|(_, value)| value).unwrap_or(false)
}

fn walk_flag(matches: &ArgMatches, id: &str, best: &mut Option<(u8, bool)>) {
    if let Some(value) = matches.try_get_one::<bool>(id).ok().flatten().copied() {
        consider(best, source_rank(matches.value_source(id)), value);
    }
    if let Some((_, sub)) = matches.subcommand() {
        walk_flag(sub, id, best);
    }
}

fn consider<T>(best: &mut Option<(u8, T)>, rank: u8, value: T) {
    if best.as_ref().is_none_or(|(score, _)| rank >= *score) {
        *best = Some((rank, value));
    }
}

fn source_rank(source: Option<ValueSource>) -> u8 {
    match source {
        Some(ValueSource::CommandLine) => 3,
        Some(ValueSource::EnvVariable) => 2,
        Some(ValueSource::DefaultValue) => 1,
        _ => 0,
    }
}

fn external_words(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<OsString>("")
        .into_iter()
        .flatten()
        .map(|word| shell_single_quote(&word.to_string_lossy()))
        .collect()
}

fn write_set(out: &mut String, path: &[String], extra: &[String]) {
    let mut line = String::from("set --");
    if path.is_empty() {
        line.push(' ');
    } else {
        line.push(' ');
        line.push_str(&path.join(" "));
    }
    for word in extra {
        line.push(' ');
        line.push_str(word);
    }
    line.push('\n');
    out.push_str(&line);
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

pub(crate) fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn completion_wrapper(script: &str) -> String {
    let mut escaped = String::with_capacity(script.len());
    for ch in script.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            _ => escaped.push(ch),
        }
    }
    format!("echo $'{escaped}'\nexit 0\n")
}
