mod render;
mod schema;

use render::render_matches;

use std::ffi::OsString;

use clap::builder::PossibleValuesParser;
use clap::{Arg, Command, ValueEnum};

#[derive(Debug)]
pub enum Error {
    Message(String),
    Usage(clap::error::Error),
}

impl Error {
    pub fn usage_kind(&self) -> Option<clap::error::ErrorKind> {
        match self {
            Error::Usage(error) => Some(error.kind()),
            Error::Message(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(message) => formatter.write_str(message),
            Error::Usage(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for Error {}

pub fn eval<I, S>(yaml: &str, args: I) -> Result<String, Error>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let cmd = schema::command_from_yaml(yaml).map_err(Error::Message)?;
    let cmd = attach_completion(cmd)?;
    let matches = cmd
        .clone()
        .try_get_matches_from(args)
        .map_err(Error::Usage)?;
    match matches.subcommand() {
        Some(("clap4shell-completion", sub)) => render_completion(&cmd, sub),
        _ => Ok(render_matches(&cmd, &matches)),
    }
}

fn attach_completion(cmd: Command) -> Result<Command, Error> {
    if cmd.find_subcommand("clap4shell-completion").is_some() {
        return Err(Error::Message(
            "subcommand `clap4shell-completion` is reserved".into(),
        ));
    }
    let shells = shell_names().join(", ");
    Ok(cmd.subcommand(
        Command::new("clap4shell-completion")
            .about("Generate an autocompletion script")
            .arg(
                Arg::new("shell")
                    .value_name("SHELL")
                    .help(format!("target shell ({shells})"))
                    .value_parser(PossibleValuesParser::new(shell_names())),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Write to file"),
            ),
    ))
}

fn shell_names() -> Vec<String> {
    let mut names: Vec<String> = clap_complete::Shell::value_variants()
        .iter()
        .filter_map(|shell| {
            shell
                .to_possible_value()
                .map(|value| value.get_name().to_string())
        })
        .collect();
    names.push("pwsh".to_string());
    names
}

fn render_completion(cmd: &Command, matches: &clap::ArgMatches) -> Result<String, Error> {
    let shell = match matches.get_one::<String>("shell") {
        Some(name) => parse_shell(name)?,
        None => {
            let path = std::env::var_os("SHELL")
                .ok_or_else(|| Error::Message("SHELL environment variable is not set".into()))?;
            parse_shell(&path)?
        }
    };
    let bin_name = cmd
        .get_bin_name()
        .unwrap_or_else(|| cmd.get_name())
        .to_string();
    let mut generated = cmd.clone();
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut generated, bin_name, &mut buf);
    let script = String::from_utf8(buf).map_err(|error| Error::Message(error.to_string()))?;
    if let Some(path) = matches.get_one::<String>("output") {
        std::fs::write(path, &script).map_err(|error| Error::Message(error.to_string()))?;
        Ok(String::new())
    } else {
        Ok(render::completion_wrapper(&script))
    }
}

fn parse_shell(path: impl AsRef<std::path::Path>) -> Result<clap_complete::Shell, Error> {
    let path = path.as_ref();
    clap_complete::Shell::from_shell_path(path).ok_or_else(|| {
        Error::Message(format!(
            "unknown shell `{}`; expected {}",
            path.display(),
            shell_names().join(", ")
        ))
    })
}
