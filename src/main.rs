use clap::{App, Arg, ArgMatches};
use clap_complete::Generator;
use std::io::{Read, Write};

fn app() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let mut app_body: clap::Command = serde_yaml::from_str::<clap_serde::CommandWrap>(&input)
        .map_err(|e| e.to_string())?
        .into();
    app_body = app_body.name("parse");

    let app_completion = App::new("clap4shell-completion")
        .arg(Arg::new("shell").takes_value(true).required(true))
        .arg(
            Arg::new("output")
                .takes_value(true)
                .required(true)
                .short('o')
                .long("output"),
        );

    let app = App::new(app_body.get_name())
        .subcommand(app_completion)
        .subcommand(app_body.clone());

    let matches = app.clone().get_matches_safe().map_err(|e| e.to_string())?;

    match matches.subcommand() {
        Some(("clap4shell-completion", sub_matches)) => {
            let shell = sub_matches.value_of("shell").ok_or("shell is required")?;
            let path = sub_matches.value_of("output").ok_or("output is required")?;
            let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
            shell
                .parse::<clap_complete::Shell>()?
                .generate(&app_body, &mut file);
        }
        Some((name, sub_matches)) if name == app_body.get_name() => {
            print_matches(&sub_matches, &app_body)
        }
        _ => panic!("Unexpected subcommand"),
    }

    Ok(())
}

fn print_matches(matches: &ArgMatches, app: &App) {
    for arg in app.get_arguments() {
        let k = arg.get_name();
        if arg.is_takes_value_set() || arg.is_positional() {
            match matches.values_of(k) {
                Some(v) => {
                    let values: Vec<&str> = v.collect();
                    println!("{}='{}'", k, values.join("\n").replace("'", "\\'"))
                }
                None => (),
            }
        } else {
            println!("{}={}", k, matches.occurrences_of(k));
        }
    }
    match matches.subcommand() {
        Some((name, sub_app)) => {
            let sub_info = app
                .get_subcommands()
                .find(|s| s.get_name() == name)
                .expect("subcommand info");
            println!("subcommand={}", name);
            print_matches(&sub_app, &sub_info);
        }
        _ => {}
    }
}

fn main() {
    match app() {
        Err(msg) => {
            writeln!(std::io::stderr(), "clap4shell: {}", msg).unwrap();
            std::process::exit(1);
        }
        Ok(_) => {}
    }
}
