use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use clap::error::ErrorKind;

fn eval<I, S>(yaml: &str, args: I) -> Result<String, clap4shell::Error>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    clap4shell::eval(yaml, args)
}

fn bash(script: &str) -> String {
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(script)
        .output()
        .expect("bash is available");
    assert!(
        output.status.success(),
        "bash failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("bash output is utf-8")
}

fn kind(error: &clap4shell::Error) -> ErrorKind {
    error
        .usage_kind()
        .unwrap_or_else(|| panic!("expected a usage error, got {error}"))
}

#[test]
fn minimal_command_prints_an_empty_parameter_list() {
    assert_eq!(eval("name: demo\n", ["demo"]).unwrap(), "set -- \n");
}

#[test]
fn count_flag_defaults_to_zero() {
    let yaml = "name: demo\nargs:\n  - verbose:\n      short: v\n";
    assert_eq!(eval(yaml, ["demo"]).unwrap(), "verbose=0\nset -- \n");
    assert_eq!(eval(yaml, ["demo", "-vv"]).unwrap(), "verbose=2\nset -- \n");
}

#[test]
fn set_true_is_zero_or_one() {
    let yaml = "name: demo\nargs:\n  - force:\n      long: force\n      action: set_true\n";
    assert_eq!(eval(yaml, ["demo"]).unwrap(), "force=0\nset -- \n");
    assert_eq!(
        eval(yaml, ["demo", "--force"]).unwrap(),
        "force=1\nset -- \n"
    );
    assert_eq!(
        kind(&eval(yaml, ["demo", "--force", "--force"]).unwrap_err()),
        ErrorKind::ArgumentConflict
    );
}

#[test]
fn omitted_value_stays_unset_and_default_is_quoted() {
    let yaml = "name: demo\nargs:\n  - host:\n      long: host\n      takes_value: true\n";
    assert_eq!(eval(yaml, ["demo"]).unwrap(), "set -- \n");
    assert_eq!(
        eval(yaml, ["demo", "--host", "example"]).unwrap(),
        "host='example'\nset -- \n"
    );
    let yaml = "name: demo\nargs:\n  - host:\n      long: host\n      default_value: localhost\n";
    assert_eq!(eval(yaml, ["demo"]).unwrap(), "host='localhost'\nset -- \n");
}

#[test]
fn assignments_round_trip_through_bash() {
    let yaml = "name: demo\nargs:\n  - msg:\n      long: msg\n      takes_value: true\n";
    let out = eval(yaml, ["demo", "--msg", "a'b\\c"]).unwrap();
    assert_eq!(bash(&format!("{out}printf %s \"$msg\"")), "a'b\\c");
}

#[test]
fn repeated_values_join_with_newlines() {
    let positional = "name: demo\nargs:\n  - arg:\n      multiple_values: true\n";
    let out = eval(positional, ["demo", "a", "b"]).unwrap();
    assert_eq!(bash(&format!("{out}printf %s \"$arg\"")), "a\nb");

    let append = "name: demo\nargs:\n  - tag:\n      long: tag\n      action: append\n";
    let out = eval(append, ["demo", "--tag", "a", "--tag", "b"]).unwrap();
    assert_eq!(bash(&format!("{out}printf %s \"$tag\"")), "a\nb");

    let delimited =
        "name: demo\nargs:\n  - hosts:\n      long: hosts\n      value_delimiter: ','\n";
    let out = eval(delimited, ["demo", "--hosts", "a,b"]).unwrap();
    assert_eq!(bash(&format!("{out}printf %s \"$hosts\"")), "a\nb");
}

#[test]
fn validators_reject_bad_input() {
    let modes = "name: demo\ncolor: never\nargs:\n  - mode:\n      long: mode\n      possible_values: [herbivore, carnivore]\n      ignore_case: true\n";
    assert!(eval(modes, ["demo", "--mode", "Herbivore"]).is_ok());
    assert_eq!(
        kind(&eval(modes, ["demo", "--mode", "plant"]).unwrap_err()),
        ErrorKind::InvalidValue
    );

    let numbers = "name: demo\nallow_negative_numbers: true\nargs:\n  - n:\n      long: n\n      value_parser: i64\n";
    assert_eq!(
        eval(numbers, ["demo", "--n", "-3"]).unwrap(),
        "n='-3'\nset -- \n"
    );
    assert_eq!(
        kind(&eval(numbers, ["demo", "--n", "nope"]).unwrap_err()),
        ErrorKind::ValueValidation
    );
}

#[test]
fn constraints_reject_invalid_combinations() {
    let required = "name: demo\nargs:\n  - name:\n      long: name\n      required: true\n";
    assert_eq!(
        kind(&eval(required, ["demo"]).unwrap_err()),
        ErrorKind::MissingRequiredArgument
    );

    let conflicts = "name: demo\nargs:\n  - json:\n      long: json\n      action: set_true\n      conflicts_with: yaml\n  - yaml:\n      long: yaml\n      action: set_true\n";
    assert_eq!(
        kind(&eval(conflicts, ["demo", "--json", "--yaml"]).unwrap_err()),
        ErrorKind::ArgumentConflict
    );

    let group = "name: demo\nargs:\n  - json:\n      long: json\n      action: set_true\n  - yaml:\n      long: yaml\n      action: set_true\ngroups:\n  format:\n    args: [json, yaml]\n    multiple: false\n";
    assert_eq!(
        kind(&eval(group, ["demo", "--json", "--yaml"]).unwrap_err()),
        ErrorKind::ArgumentConflict
    );
    assert_eq!(
        eval(group, ["demo", "--json"]).unwrap(),
        "json=1\nyaml=0\nset -- \n"
    );
}

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn env_fills_a_missing_value() {
    let yaml =
        "name: demo\nargs:\n  - token:\n      long: token\n      env: CLAP4SHELL_TEST_TOKEN\n";
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let previous = std::env::var_os("CLAP4SHELL_TEST_TOKEN");
    unsafe { std::env::remove_var("CLAP4SHELL_TEST_TOKEN") };
    let without = eval(yaml, ["demo"]).unwrap();
    unsafe { std::env::set_var("CLAP4SHELL_TEST_TOKEN", "sekret") };
    let with = eval(yaml, ["demo"]);
    match previous {
        Some(value) => unsafe { std::env::set_var("CLAP4SHELL_TEST_TOKEN", value) },
        None => unsafe { std::env::remove_var("CLAP4SHELL_TEST_TOKEN") },
    }
    assert!(!without.contains("token="), "{without}");
    assert_eq!(with.unwrap(), "token='sekret'\nset -- \n");
}

#[test]
fn completion_shell_defaults_to_the_shell_env() {
    let yaml = "name: demo\nbin_name: demo\n";
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let previous = std::env::var_os("SHELL");
    unsafe { std::env::set_var("SHELL", "/usr/local/bin/pwsh") };
    let pwsh = eval(yaml, ["demo", "clap4shell-completion"]);
    unsafe { std::env::set_var("SHELL", "/bin/tcsh") };
    let unknown = eval(yaml, ["demo", "clap4shell-completion"]);
    match previous {
        Some(value) => unsafe { std::env::set_var("SHELL", value) },
        None => unsafe { std::env::remove_var("SHELL") },
    }
    assert!(pwsh.unwrap().contains("Register-ArgumentCompleter"));
    let error = unknown.unwrap_err();
    assert!(error.usage_kind().is_none(), "{error}");
    assert!(error.to_string().contains("/bin/tcsh"), "{error}");
}

#[test]
fn single_value_name_and_default_survive_list_setters() {
    let yaml = "name: demo\ncolor: never\nargs:\n  - host:\n      long: host\n      value_name: HOST\n      default_value: localhost\n";
    assert_eq!(eval(yaml, ["demo"]).unwrap(), "host='localhost'\nset -- \n");
    let help = eval(yaml, ["demo", "--help"]).unwrap_err().to_string();
    assert!(help.contains("--host <HOST>"), "{help}");
}

#[test]
fn global_values_are_printed_once() {
    let yaml = r#"
name: demo
args:
  - verbose:
      short: v
      global: true
  - host:
      long: host
      global: true
      default_value: localhost
subcommands:
  - ls: {}
"#;
    let before = eval(yaml, ["demo", "-v", "--host", "example.com", "ls"]).unwrap();
    assert_eq!(before.matches("verbose=").count(), 1, "{before}");
    assert!(before.contains("verbose=1\n"), "{before}");
    assert!(before.contains("host='example.com'\n"), "{before}");
    assert!(before.contains("set -- ls\n"), "{before}");

    let after = eval(yaml, ["demo", "ls", "-vv", "--host", "example.com"]).unwrap();
    assert_eq!(after.matches("verbose=").count(), 1, "{after}");
    assert!(after.contains("verbose=2\n"), "{after}");
    assert!(after.contains("host='example.com'\n"), "{after}");

    let defaults = eval(yaml, ["demo", "ls"]).unwrap();
    assert!(defaults.contains("verbose=0\n"), "{defaults}");
    assert!(defaults.contains("host='localhost'\n"), "{defaults}");
}

#[test]
fn optional_value_uses_the_missing_default() {
    let yaml = r#"
name: demo
args:
  - color:
      long: color
      num_args: 0..=1
      default_missing_value: auto
      default_value: never
"#;
    assert_eq!(eval(yaml, ["demo"]).unwrap(), "color='never'\nset -- \n");
    assert_eq!(
        eval(yaml, ["demo", "--color"]).unwrap(),
        "color='auto'\nset -- \n"
    );
    assert_eq!(
        eval(yaml, ["demo", "--color", "always"]).unwrap(),
        "color='always'\nset -- \n"
    );
}

#[test]
fn hyphen_values_can_be_allowed() {
    let yaml = "name: demo\nargs:\n  - msg:\n      long: msg\n      allow_hyphen_values: true\n";
    assert_eq!(
        eval(yaml, ["demo", "--msg", "-n"]).unwrap(),
        "msg='-n'\nset -- \n"
    );
}

#[test]
fn positional_order_follows_the_document() {
    let yaml = "name: demo\nargs:\n  - first: {}\n  - second: {}\n";
    let out = eval(yaml, ["demo", "a", "b"]).unwrap();
    assert_eq!(
        bash(&format!(
            "{out}printf %s\\\\n \"$first\"\nprintf %s\\\\n \"$second\""
        )),
        "a\nb\n"
    );
}

#[test]
fn subcommand_becomes_the_first_parameter() {
    let yaml = r#"
name: demo
subcommands:
  - ls:
      args:
        - entity:
            multiple_values: true
"#;
    let out = eval(yaml, ["demo", "ls", "a", "b"]).unwrap();
    assert_eq!(
        bash(&format!(
            "{out}printf %s\\\\n \"$1\"\nprintf %s \"$entity\""
        )),
        "ls\na\nb"
    );
}

#[test]
fn external_subcommand_keeps_trailing_words() {
    let yaml = "name: demo\nallow_external_subcommands: true\n";
    let out = eval(yaml, ["demo", "plugin", "a", "b c"]).unwrap();
    assert_eq!(
        bash(&format!(
            "{out}printf %s\\\\n \"$1\"\nprintf %s\\\\n \"$2\"\nprintf %s\\\\n \"$3\""
        )),
        "plugin\na\nb c\n"
    );
}

#[test]
fn unknown_field_names_the_field() {
    let yaml = "name: demo\nargs:\n  - item:\n      not_a_field: true\n";
    let error = eval(yaml, ["demo"]).unwrap_err();
    assert!(error.usage_kind().is_none(), "{error}");
    assert!(error.to_string().contains("not_a_field"), "{error}");
}

#[test]
fn help_is_not_eval_output() {
    let yaml = "name: demo\nabout: sample\ncolor: never\n";
    let error = eval(yaml, ["demo", "--help"]).unwrap_err();
    assert_eq!(kind(&error), ErrorKind::DisplayHelp);
    assert!(error.to_string().contains("sample"), "{error}");
}

#[test]
fn completion_roundtrips_and_can_be_written() {
    let yaml = r#"
name: demo
bin_name: demo
args:
  - file:
      long: file
      value_hint: file_path
"#;
    let wrapped = eval(yaml, ["demo", "clap4shell-completion", "bash"]).unwrap();
    let script = bash(&wrapped);
    assert!(script.contains("--file"), "{script}");
    assert!(
        script.contains("compgen -f") || script.contains("_filedir"),
        "{script}"
    );
    let powershell = bash(&eval(yaml, ["demo", "clap4shell-completion", "pwsh"]).unwrap());
    assert!(powershell.contains("file"), "{powershell}");

    let path =
        std::env::temp_dir().join(format!("clap4shell-completion-{}.sh", std::process::id()));
    let _cleanup = RemoveFile(path.clone());
    let written = eval(
        yaml,
        [
            "demo",
            "clap4shell-completion",
            "bash",
            "-o",
            path.to_str().unwrap(),
        ],
    )
    .unwrap();
    assert!(written.is_empty(), "{written}");
    let file = std::fs::read_to_string(&path).unwrap();
    assert!(file.contains("--file"), "{file}");
}

struct RemoveFile(PathBuf);

impl Drop for RemoveFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}
