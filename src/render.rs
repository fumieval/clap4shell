use std::ffi::OsString;

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
        let id = arg.get_id();
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

// clap copies the winning value of a global argument into every level of
// `ArgMatches`, so the level that declares the argument already holds it.
fn write_arg(arg: &Arg, matches: &ArgMatches, out: &mut String) {
    let id = arg.get_id().as_str();
    match arg.get_action() {
        ArgAction::Help | ArgAction::Version => {}
        ArgAction::Count => {
            let count = get_one::<u8>(matches, id).unwrap_or(0);
            push_line(out, &format!("{id}={count}"));
        }
        ArgAction::SetTrue | ArgAction::SetFalse => {
            let value = get_one::<bool>(matches, id).unwrap_or(false);
            push_line(out, &format!("{id}={}", u8::from(value)));
        }
        action if action.takes_values() => {
            if let Ok(Some(values)) = matches.try_get_many::<String>(id) {
                let joined = values.map(String::as_str).collect::<Vec<_>>().join("\n");
                push_line(out, &format!("{id}={}", shell_single_quote(&joined)));
            }
        }
        _ => {}
    }
}

fn get_one<T: Clone + Send + Sync + 'static>(matches: &ArgMatches, id: &str) -> Option<T> {
    matches.try_get_one::<T>(id).ok().flatten().cloned()
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
    let mut line = format!("set -- {}", path.join(" "));
    for word in extra {
        line.push(' ');
        line.push_str(word);
    }
    push_line(out, &line);
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
