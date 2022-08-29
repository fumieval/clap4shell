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
        .arg(Arg::new("shell").takes_value(true).required(true))
        .arg(
            Arg::new("output")
                .takes_value(true)
                .required(true)
                .short('o')
                .long("output"),
        );

    let mut app = app_body.subcommand(app_completion);
    app.build();

    let matches = app.clone().get_matches_safe().map_err(|e| e.to_string())?;

    match matches.subcommand() {
        Some(("clap4shell-completion", sub_matches)) => {
            let shell = sub_matches.value_of("shell").unwrap();
            let path = sub_matches.value_of("output").unwrap();
            let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
            shell
                .parse::<clap_complete::Shell>()?
                .generate(&app, &mut file);
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
