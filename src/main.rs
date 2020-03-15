extern crate clap;
extern crate yaml_rust;
use clap::{App, Arg};
use std::collections::BTreeSet;
use std::io::{Read, Write};
use yaml_rust::{Yaml, YamlLoader};

struct AppInfo<'a> {
    flags: BTreeSet<&'a str>,
    args: BTreeSet<&'a str>,
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

macro_rules! yaml_to_str {
    ($a:ident, $v:ident, $c:ident) => {{
        $a.$c($v
            .as_str()
            .expect(&*format!("expecting string for {}", stringify!($c))))
    }};
}

fn build_arg<'a, 'b>(key: &'a str, takes_value: bool, obj: &'a Yaml) -> Arg<'a, 'b> {
    let mut a = Arg::with_name(key).takes_value(takes_value);
    let arg_settings = obj.as_hash().expect("expecting object");
    for (k, v) in arg_settings.iter() {
        a = match k.as_str().expect("key must be a string") {
            "name" => a,
            "required" => a.required(v.as_bool().expect("expecting bool")),
            "multiple" => a.multiple(v.as_bool().expect("expecting bool")),
            "short" => yaml_to_str!(a, v, short),
            "long" => yaml_to_str!(a, v, long),
            "aliases" => yaml_vec_or_str!(a, v, alias),
            "help" => yaml_to_str!(a, v, help),
            "default_value" => yaml_to_str!(a, v, default_value),
            "possible_values" => yaml_vec_or_str!(a, v, possible_value),
            key => panic!("Unexpected key {} for Arg", key),
        }
    }
    return a;
}

fn build_app<'a, 'b>(obj: &'a Yaml) -> (App<'a, 'b>, AppInfo<'a>) {
    let mut app = App::new("");
    let mut args = BTreeSet::new();
    let mut flags = BTreeSet::new();

    for (key_raw, v) in obj.as_hash().expect("Expecting an object").iter() {
        let key = key_raw.as_str().expect("key must be a string");
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
            "help_short" => yaml_to_str!(app, v, help_short),
            "version_short" => yaml_to_str!(app, v, version_short),
            "help_message" => yaml_to_str!(app, v, help_message),
            "version_message" => yaml_to_str!(app, v, version_message),
            "settings" => {
                for ys in v.as_vec().expect(".settings: expecting a list").iter() {
                    if let Some(s) = ys.as_str() {
                        let setting = s.parse().expect(&format!("unknown AppSetting: {}", s));
                        app = app.setting(setting);
                    }
                }
                app
            }
            "flags" => {
                for (key_raw, arg_obj) in v.as_hash().expect("Expecting an object for args").iter()
                {
                    let key = key_raw.as_str().expect("Key must be a string");
                    let arg = build_arg(key, false, arg_obj);
                    app = app.arg(arg);
                    flags.insert(key);
                }
                app
            }
            "opts" => {
                for (key_raw, arg_obj) in v.as_hash().expect(".opts: expecting an object").iter() {
                    let key = key_raw.as_str().expect(".opts: key must be a string");
                    let arg = build_arg(key, true, arg_obj);
                    app = app.arg(arg);
                    args.insert(key);
                }
                app
            }
            "args" => {
                for (i, arg_obj) in v
                    .as_vec()
                    .expect(".args: expecting a list")
                    .iter()
                    .enumerate()
                {
                    let msg = |s: &str| format!(".args.[{}]: {}", i, s);
                    let key = arg_obj
                        .as_hash()
                        .expect(&msg("expecting an object"))
                        .get(&Yaml::String("name".to_string()))
                        .expect(&msg("name not found"))
                        .as_str()
                        .expect(&msg("name must be a string"));
                    let arg = build_arg(key, true, arg_obj);
                    app = app.arg(arg);
                    args.insert(key);
                }
                app
            }
            _ => panic!("Unexpected key {} for App", key),
        }
    }
    return (
        app,
        AppInfo {
            args: args,
            flags: flags,
        },
    );
}

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let docs = YamlLoader::load_from_str(&input).unwrap();

    let (app, info) = build_app(&docs[0]);

    let matches = app.get_matches_safe().unwrap_or_else(|e| {
        writeln!(std::io::stderr(), "{}", e.message).unwrap();
        println!("exit 1");
        std::process::exit(1);
    });

    for k in info.flags {
        println!("{}={}", k, matches.occurrences_of(k));
    }
    for k in info.args {
        match matches.values_of_lossy(k) {
            Some(v) => println!("{}='{}'", k, v.join("\n").replace("'", "\\'")),
            None => println!("{}=", k),
        }
    }
}
