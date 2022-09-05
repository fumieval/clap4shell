use clap::ValueEnum;
use clap::{App, Arg, ArgMatches};
use clap_complete::Generator;
use std::io::Read;

fn app() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let app_body: clap::Command = serde_yaml::from_str::<clap_serde::CommandWrap>(&input)
        .map_err(|e| e.to_string())?
        .into();

    let app_completion = App::new("clap4shell-completion")
        .about("Generate an autocompletion script")
        .arg(
            Arg::new("shell")
                .takes_value(true)
                .help("target shell")
                .possible_values(
                    clap_complete::Shell::value_variants()
                        .iter()
                        .map(|x| x.to_possible_value().unwrap()),
                ),
        )
        .arg(
            Arg::new("output")
                .takes_value(true)
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Write to file"),
        );

    let mut app = app_body.subcommand(app_completion);
    app.build();

    let matches = app.clone().get_matches_safe().map_err(|e| e.to_string())?;

    match matches.subcommand() {
        Some(("clap4shell-completion", sub_matches)) => {
            let shell = match sub_matches.value_of("shell") {
                Some(x) => x.to_string(),
                None => {
                    let var =
                        std::env::var_os("SHELL").expect("SHELL environment variable is not set");
                    let path = std::path::Path::new(&var);
                    path.file_name().unwrap().to_str().unwrap().to_string()
                }
            }
            .parse::<clap_complete::Shell>()?;
            match sub_matches.value_of("output") {
                Some(path) => {
                    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
                    shell.generate(&app, &mut file);
                }
                None => {
                    let mut buffer = Vec::new();
                    shell.generate(&app, &mut buffer);
                    let script = String::from_utf8(buffer).map_err(|e| e.to_string())?;
                    println!("echo $'{}'", script.replace('\'', "\\'"));
                    println!("exit 0")
                }
            }
        }
        _ => print_matches(vec![], &matches, &app),
    }

    Ok(())
}

fn print_matches(parents: Vec<&str>, matches: &ArgMatches, app: &App) {
    for arg in app.get_arguments() {
        let k = arg.get_name();
        if arg.is_takes_value_set() || arg.is_positional() {
            if let Some(v) = matches.values_of(k) {
                let values: Vec<&str> = v.collect();
                println!("{}='{}'", k, values.join("\n").replace('\'', "\\'"))
            }
        } else {
            println!("{}={}", k, matches.occurrences_of(k));
        }
    }
    if let Some((name, sub_app)) = matches.subcommand() {
        let sub_info = app
            .get_subcommands()
            .find(|s| s.get_name() == name)
            .expect("subcommand info");
        let mut subcommands = parents.clone();
        subcommands.push(name);
        println!(
            "{}={}",
            std::iter::once("subcommand")
                .chain(parents)
                .collect::<Vec<&str>>()
                .join("_"),
            name
        );
        print_matches(subcommands, sub_app, sub_info);
    }
}

fn main() {
    if let Err(msg) = app() {
        eprintln!("clap4shell: {}", msg);
        std::process::exit(1);
    }
}
