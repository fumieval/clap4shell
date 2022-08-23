use clap::{App, AppSettings, Arg, ArgMatches};
use clap_complete::Generator;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use yaml_rust::{Yaml, YamlLoader};

struct AppInfo<'a> {
    flags: BTreeSet<&'a str>,
    args: BTreeSet<&'a str>,
    subcommands: BTreeMap<&'a str, AppInfo<'a>>,
}

macro_rules! yaml_vec_or_str {
    ($a:ident, $v:ident, $c:ident) => {{
        let maybe_vec = $v.as_vec();
        if let Some(vec) = maybe_vec {
            for ys in vec {
                if let Some(s) = ys.as_str() {
                    $a = $a.$c(s);
                } else {
                    panic!("Failed to convert YAML value {:?} to a string", ys);
                }
            }
        } else {
            if let Some(s) = $v.as_str() {
                $a = $a.$c(s);
            } else {
                panic!(
                    "Failed to convert YAML value {:?} to either a vec or string",
                    $v
                );
            }
        }
        $a
    }};
}

macro_rules! yaml_to_char {
    ($a:ident, $v:ident, $c:ident) => {{
        $a.$c($v
            .as_str()
            .expect(&*format!("expecting string for {}", stringify!($c)))
            .chars()
            .nth(0)
            .expect("expecting a single character"))
    }};
}

macro_rules! yaml_to_str {
    ($a:ident, $v:ident, $c:ident) => {{
        $a.$c($v
            .as_str()
            .expect(&*format!("expecting string for {}", stringify!($c))))
    }};
}

fn build_arg<'a>(
    context: &str,
    key: &'a str,
    takes_value: bool,
    obj: &'a Yaml,
) -> Result<Arg<'a>, String> {
    let base = match obj.as_str() {
        Some(usage) => return Ok(Arg::from_usage(usage).takes_value(takes_value)),
        None => Arg::with_name(key),
    };
    let mut a = base.takes_value(takes_value);
    let msg = |s: &str| format!(".{}.{}: {}", context, key, s);
    let arg_settings = obj.as_hash().ok_or(msg("expecting object"))?;
    for (k, v) in arg_settings.iter() {
        a = match k.as_str().ok_or("key must be a string")? {
            "name" => a,
            "required" => a.required(v.as_bool().ok_or(msg("expecting bool"))?),
            "multiple" => a.multiple(v.as_bool().ok_or(msg("expecting bool"))?),
            "short" => yaml_to_char!(a, v, short),
            "long" => yaml_to_str!(a, v, long),
            "aliases" => yaml_vec_or_str!(a, v, alias),
            "help" => yaml_to_str!(a, v, help),
            "default_value" => yaml_to_str!(a, v, default_value),
            "possible_values" => yaml_vec_or_str!(a, v, possible_value),
            "value_delimiter" => yaml_to_char!(a, v, value_delimiter),
            "requires" => yaml_vec_or_str!(a, v, possible_value),
            // "env" => yaml_to_str!(a, v, env),
            "index" => a.index(v.as_i64().ok_or(msg("expecting integer"))? as usize),
            key => return Err(format!("Unexpected key {} for Arg", key)),
        }
    }
    Ok(a)
}

fn build_app<'a>(base_name: &str, obj: &'a Yaml) -> Result<(App<'a>, AppInfo<'a>), String> {
    let mut app = App::new(base_name);
    let mut args = BTreeSet::new();
    let mut flags = BTreeSet::new();
    let mut subcommands = BTreeMap::new();

    for (key_raw, v) in obj.as_hash().ok_or("Expecting an object")?.iter() {
        let key = key_raw.as_str().ok_or("key must be a string")?;
        app = match key {
            "author" => yaml_to_str!(app, v, author),
            "bin_name" => yaml_to_str!(app, v, bin_name),
            "about" => yaml_to_str!(app, v, about),
            "long_about" => yaml_to_str!(app, v, long_about),
            "name" => yaml_to_str!(app, v, name),
            "after_help" => yaml_to_str!(app, v, after_help),
            "before_help" => yaml_to_str!(app, v, before_help),
            "version" => yaml_to_str!(app, v, version),
            "long_version" => yaml_to_str!(app, v, long_version),
            "usage" => yaml_to_str!(app, v, usage),
            "help" => yaml_to_str!(app, v, help),
            "help_short" => yaml_to_char!(app, v, help_short),
            "version_short" => yaml_to_char!(app, v, version_short),
            "help_message" => yaml_to_str!(app, v, help_message),
            "version_message" => yaml_to_str!(app, v, version_message),
            "settings" => {
                for (i, ys) in v
                    .as_vec()
                    .ok_or(".settings: expecting a list")?
                    .iter()
                    .enumerate()
                {
                    let s = ys
                        .as_str()
                        .ok_or(format!(".settings[{}]: expecting a string", i))?;
                    app = app.setting(parse_app_settings(s)?);
                }
                app
            }
            "flags" => {
                for (key_raw, arg_obj) in v.as_hash().ok_or(".args: expecting an object")?.iter() {
                    let inner = key_raw.as_str().ok_or(".args: key must be a string")?;
                    let arg = build_arg(key, inner, false, arg_obj)?;
                    app = app.arg(arg);
                    flags.insert(inner);
                }
                app
            }
            "opts" => {
                for (key_raw, arg_obj) in v.as_hash().ok_or(".opts: expecting an object")?.iter() {
                    let inner = key_raw.as_str().ok_or(".opts: key must be a string")?;
                    let arg = build_arg(key, inner, true, arg_obj)?;
                    app = app.arg(arg);
                    args.insert(inner);
                }
                app
            }
            "args" => {
                for (i, arg_obj) in v
                    .as_vec()
                    .ok_or(".args: expecting a list")?
                    .iter()
                    .enumerate()
                {
                    let msg = |s: &str| format!(".args.[{}]: {}", i, s);
                    let inner = arg_obj
                        .as_hash()
                        .ok_or(&msg("expecting an object"))?
                        .get(&Yaml::String("name".to_string()))
                        .ok_or(&msg("name not found"))?
                        .as_str()
                        .ok_or(&msg("name must be a string"))?;
                    let arg = build_arg(key, inner, true, arg_obj)?;
                    app = app.arg(arg);
                    args.insert(inner);
                }
                app
            }
            "subcommands" => {
                for (key_raw, app_obj) in v
                    .as_hash()
                    .ok_or(".subcommands: expecting an object")?
                    .iter()
                {
                    let inner_key = key_raw
                        .as_str()
                        .ok_or(".subcommands: key must be a string")?;
                    let (sub, info) = build_app(inner_key, app_obj)?;
                    app = app.subcommand(sub);
                    subcommands.insert(inner_key, info);
                }
                app
            }
            _ => return Err(format!("Unexpected key {} for App", key)),
        }
    }
    Ok((
        app,
        AppInfo {
            args: args,
            flags: flags,
            subcommands: subcommands,
        },
    ))
}

fn app() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let docs = YamlLoader::load_from_str(&input).map_err(|e| e.to_string())?;
    if docs.len() != 1 {
        return Err("expecting a single YAML document".to_string());
    }
    let (app_body, info) = build_app("", &docs[0])?;

    let app_completion = App::new("clap4shell-completion")
        .arg(Arg::new("shell").takes_value(true).required(true))
        .arg(
            Arg::new("output")
                .takes_value(true)
                .required(true)
                .short('o')
                .long("output"),
        );

    let app = App::new("clap4shell")
        .bin_name("clap4shell")
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
        _ => print_matches(&matches, &info),
    }

    Ok(())
}

fn print_matches<'a>(matches: &ArgMatches, info: &AppInfo<'a>) {
    for k in info.flags.iter() {
        println!("{}={}", k, matches.occurrences_of(k));
    }
    for k in info.args.iter() {
        match matches.values_of_lossy(k) {
            Some(v) => println!("{}='{}'", k, v.join("\n").replace("'", "\\'")),
            None => println!("{}=", k),
        }
    }
    match matches.subcommand() {
        Some((name, sub_app)) => {
            let sub_info = info.subcommands.get(name).expect("subcommand info");
            println!("subcommand={}", name);
            print_matches(sub_app, sub_info);
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

fn parse_app_settings(s: &str) -> Result<AppSettings, String> {
    #[allow(deprecated)]
    #[allow(unreachable_patterns)]
    match &*s.to_ascii_lowercase() {
        "argrequiredelsehelp" => Ok(AppSettings::ArgRequiredElseHelp),
        "subcommandprecedenceoverarg" => Ok(AppSettings::SubcommandPrecedenceOverArg),
        "argsnegatesubcommands" => Ok(AppSettings::ArgsNegateSubcommands),
        "allowexternalsubcommands" => Ok(AppSettings::AllowExternalSubcommands),
        "strictutf8" => Ok(AppSettings::StrictUtf8),
        "allowinvalidutf8forexternalsubcommands" => {
            Ok(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
        }
        "allowhyphenvalues" => Ok(AppSettings::AllowHyphenValues),
        "allowleadinghyphen" => Ok(AppSettings::AllowLeadingHyphen),
        "allownegativenumbers" => Ok(AppSettings::AllowNegativeNumbers),
        "allowmissingpositional" => Ok(AppSettings::AllowMissingPositional),
        "unifiedhelpmessage" => Ok(AppSettings::UnifiedHelpMessage),
        "coloredhelp" => Ok(AppSettings::ColoredHelp),
        "coloralways" => Ok(AppSettings::ColorAlways),
        "colorauto" => Ok(AppSettings::ColorAuto),
        "colornever" => Ok(AppSettings::ColorNever),
        "dontdelimittrailingvalues" => Ok(AppSettings::DontDelimitTrailingValues),
        "dontcollapseargsinusage" => Ok(AppSettings::DontCollapseArgsInUsage),
        "derivedisplayorder" => Ok(AppSettings::DeriveDisplayOrder),
        "disablecoloredhelp" => Ok(AppSettings::DisableColoredHelp),
        "disablehelpsubcommand" => Ok(AppSettings::DisableHelpSubcommand),
        "disablehelpflag" => Ok(AppSettings::DisableHelpFlag),
        "disablehelpflags" => Ok(AppSettings::DisableHelpFlags),
        "disableversionflag" => Ok(AppSettings::DisableVersionFlag),
        "disableversion" => Ok(AppSettings::DisableVersion),
        "propagateversion" => Ok(AppSettings::PropagateVersion),
        "propagateversion" => Ok(AppSettings::GlobalVersion),
        "hidepossiblevalues" => Ok(AppSettings::HidePossibleValues),
        "hidepossiblevaluesinhelp" => Ok(AppSettings::HidePossibleValuesInHelp),
        "helpexpected" => Ok(AppSettings::HelpExpected),
        "hidden" => Ok(AppSettings::Hidden),
        "noautohelp" => Ok(AppSettings::NoAutoHelp),
        "noautoversion" => Ok(AppSettings::NoAutoVersion),
        "nobinaryname" => Ok(AppSettings::NoBinaryName),
        "subcommandsnegatereqs" => Ok(AppSettings::SubcommandsNegateReqs),
        "subcommandrequired" => Ok(AppSettings::SubcommandRequired),
        "subcommandrequiredelsehelp" => Ok(AppSettings::SubcommandRequiredElseHelp),
        "uselongformatforhelpsubcommand" => Ok(AppSettings::UseLongFormatForHelpSubcommand),
        "trailingvararg" => Ok(AppSettings::TrailingVarArg),
        "unifiedhelp" => Ok(AppSettings::UnifiedHelp),
        "nextlinehelp" => Ok(AppSettings::NextLineHelp),
        "ignoreerrors" => Ok(AppSettings::IgnoreErrors),
        "waitonerror" => Ok(AppSettings::WaitOnError),
        "built" => Ok(AppSettings::Built),
        "binnamebuilt" => Ok(AppSettings::BinNameBuilt),
        "infersubcommands" => Ok(AppSettings::InferSubcommands),
        "allargsoverrideself" => Ok(AppSettings::AllArgsOverrideSelf),
        "inferlongargs" => Ok(AppSettings::InferLongArgs),
        _ => Err(format!("unknown AppSetting: `{}`", s)),
    }
}
