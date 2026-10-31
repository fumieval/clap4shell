use std::collections::HashSet;
use std::ffi::OsString;

use clap::builder::{
    BoolishValueParser, OsStringValueParser, PossibleValuesParser, TypedValueParser, ValueParser,
};
use clap::{Arg, ArgAction, ArgGroup, ColorChoice, Command, ValueHint};
use serde::Deserialize;
use serde::de::{self, DeserializeOwned, Deserializer};

macro_rules! apply {
    ($target:ident, $value:expr, $method:ident) => {
        if let Some(value) = &$value {
            $target = $target.$method(value.clone());
        }
    };
}

macro_rules! from_str_parser {
    ($ty:ty) => {{
        fn parse(text: &str) -> Result<String, <$ty as std::str::FromStr>::Err> {
            text.parse::<$ty>().map(|value| value.to_string())
        }
        ValueParser::from(parse)
    }};
}

pub(crate) fn command_from_yaml(yaml: &str) -> Result<Command, String> {
    let spec: CommandSpec = yaml_serde::from_str(yaml).map_err(|err| err.to_string())?;
    let name = spec
        .name
        .clone()
        .ok_or_else(|| "missing field `name`".to_string())?;
    build_command(&name, &spec)
}

fn build_command(name: &str, spec: &CommandSpec) -> Result<Command, String> {
    let mut cmd = Command::new(name.to_owned())
        .aliases(&spec.aliases)
        .visible_aliases(&spec.visible_aliases)
        .long_flag_aliases(&spec.long_flag_aliases)
        .visible_long_flag_aliases(&spec.visible_long_flag_aliases)
        .short_flag_aliases(spec.short_flag_aliases.iter().copied())
        .visible_short_flag_aliases(spec.visible_short_flag_aliases.iter().copied())
        .arg_required_else_help(spec.arg_required_else_help)
        .subcommand_required(spec.subcommand_required)
        .subcommand_negates_reqs(spec.subcommand_negates_reqs)
        .args_conflicts_with_subcommands(spec.args_conflicts_with_subcommands)
        .args_override_self(spec.args_override_self)
        .propagate_version(spec.propagate_version)
        .infer_subcommands(spec.infer_subcommands)
        .infer_long_args(spec.infer_long_args)
        .allow_external_subcommands(spec.allow_external_subcommands)
        .allow_negative_numbers(spec.allow_negative_numbers)
        .trailing_var_arg(spec.trailing_var_arg)
        .dont_delimit_trailing_values(spec.dont_delimit_trailing_values)
        .disable_help_flag(spec.disable_help_flag)
        .disable_help_subcommand(spec.disable_help_subcommand)
        .disable_version_flag(spec.disable_version_flag)
        .disable_colored_help(spec.disable_colored_help)
        .next_line_help(spec.next_line_help)
        .hide(spec.hide);
    apply!(cmd, spec.bin_name, bin_name);
    apply!(cmd, spec.display_name, display_name);
    apply!(cmd, spec.version, version);
    apply!(cmd, spec.long_version, long_version);
    apply!(cmd, spec.about, about);
    apply!(cmd, spec.long_about, long_about);
    apply!(cmd, spec.author, author);
    apply!(cmd, spec.before_help, before_help);
    apply!(cmd, spec.before_long_help, before_long_help);
    apply!(cmd, spec.after_help, after_help);
    apply!(cmd, spec.after_long_help, after_long_help);
    apply!(cmd, spec.help_template, help_template);
    apply!(cmd, spec.override_usage, override_usage);
    apply!(cmd, spec.subcommand_help_heading, subcommand_help_heading);
    apply!(cmd, spec.next_help_heading, next_help_heading);
    apply!(cmd, spec.alias, alias);
    apply!(cmd, spec.visible_alias, visible_alias);
    apply!(cmd, spec.long_flag, long_flag);
    apply!(cmd, spec.long_flag_alias, long_flag_alias);
    apply!(cmd, spec.visible_long_flag_alias, visible_long_flag_alias);
    apply!(cmd, spec.short_flag, short_flag);
    apply!(cmd, spec.short_flag_alias, short_flag_alias);
    apply!(cmd, spec.visible_short_flag_alias, visible_short_flag_alias);
    apply!(cmd, spec.display_order, display_order);
    apply!(cmd, spec.term_width, term_width);
    apply!(cmd, spec.max_term_width, max_term_width);
    if let Some(color) = spec.color {
        cmd = cmd.color(color.into());
    }

    let mut seen = HashSet::new();
    for (name, arg) in spec.args.iter().chain(&spec.args_map) {
        if !seen.insert(name.as_str()) {
            return Err(format!("duplicate argument `{name}`"));
        }
        cmd = cmd.arg(build_arg(name, arg)?);
    }
    for (name, group) in &spec.groups {
        cmd = cmd.group(build_group(name, group)?);
    }
    seen.clear();
    for (name, child) in spec.subcommands.iter().chain(&spec.subcommands_map) {
        if !seen.insert(name.as_str()) {
            return Err(format!("duplicate subcommand `{name}`"));
        }
        if let Some(renamed) = &child.name {
            if renamed != name {
                return Err(format!(
                    "subcommand `{name}` sets name to `{renamed}`; the map key is the command name"
                ));
            }
        }
        cmd = cmd.subcommand(build_command(name, child)?);
    }
    Ok(cmd)
}

fn build_group(name: &str, spec: &GroupSpec) -> Result<ArgGroup, String> {
    if name.is_empty() {
        return Err("group name must not be empty".into());
    }
    let mut group = ArgGroup::new(name.to_owned())
        .args(&spec.args)
        .conflicts_with_all(&spec.conflicts_with_all)
        .requires_all(&spec.requires_all);
    apply!(group, spec.arg, arg);
    apply!(group, spec.multiple, multiple);
    apply!(group, spec.required, required);
    apply!(group, spec.conflicts_with, conflicts_with);
    apply!(group, spec.requires, requires);
    Ok(group)
}

fn build_arg(name: &str, spec: &ArgSpec) -> Result<Arg, String> {
    if name.is_empty() {
        return Err("argument name must not be empty".into());
    }
    if spec.possible_values.is_some() && spec.value_parser.is_some() {
        return Err(format!(
            "argument `{name}` sets both possible_values and value_parser"
        ));
    }
    if spec.ignore_case && spec.possible_values.is_none() {
        return Err(format!(
            "argument `{name}` sets ignore_case without possible_values"
        ));
    }
    if spec.default_value.is_some() && !spec.default_values.is_empty() {
        return Err(format!(
            "argument `{name}` sets both default_value and default_values"
        ));
    }

    // `value_names` and `default_values` replace what `value_name` and
    // `default_value` set, so the list setters must run first.
    let mut arg = Arg::new(name.to_owned())
        .aliases(&spec.aliases)
        .visible_aliases(&spec.visible_aliases)
        .short_aliases(spec.short_aliases.iter().copied())
        .visible_short_aliases(spec.visible_short_aliases.iter().copied())
        .value_names(&spec.value_names)
        .default_values(&spec.default_values)
        .conflicts_with_all(&spec.conflicts_with_all)
        .requires_all(&spec.requires_all)
        .overrides_with_all(&spec.overrides_with_all)
        .required_unless_present_any(&spec.required_unless_present_any)
        .required_unless_present_all(&spec.required_unless_present_all)
        .ignore_case(spec.ignore_case)
        .hide_possible_values(spec.hide_possible_values)
        .hide_env(spec.hide_env)
        .hide_env_values(spec.hide_env_values)
        .global(spec.global)
        .hide(spec.hide)
        .hide_long_help(spec.hide_long_help)
        .hide_short_help(spec.hide_short_help)
        .exclusive(spec.exclusive)
        .last(spec.last)
        .allow_hyphen_values(spec.allow_hyphen_values)
        .require_equals(spec.require_equals)
        .next_line_help(spec.next_line_help)
        .required(spec.required);
    apply!(arg, spec.help, help);
    apply!(arg, spec.long_help, long_help);
    apply!(arg, spec.help_heading, help_heading);
    apply!(arg, spec.long, long);
    apply!(arg, spec.alias, alias);
    apply!(arg, spec.visible_alias, visible_alias);
    apply!(arg, spec.value_name, value_name);
    apply!(arg, spec.env, env);
    apply!(arg, spec.default_value, default_value);
    apply!(arg, spec.default_missing_value, default_missing_value);
    apply!(arg, spec.value_terminator, value_terminator);
    apply!(arg, spec.conflicts_with, conflicts_with);
    apply!(arg, spec.requires, requires);
    apply!(arg, spec.overrides_with, overrides_with);
    apply!(arg, spec.group, group);
    apply!(arg, spec.required_unless_present, required_unless_present);
    apply!(arg, spec.short, short);
    apply!(arg, spec.short_alias, short_alias);
    apply!(arg, spec.visible_short_alias, visible_short_alias);
    apply!(arg, spec.value_delimiter, value_delimiter);
    apply!(arg, spec.index, index);
    apply!(arg, spec.display_order, display_order);
    if let Some(values) = &spec.possible_values {
        arg = arg.value_parser(PossibleValuesParser::new(values.clone()));
    } else if let Some(kind) = spec.value_parser {
        arg = arg.value_parser(kind.parser());
    }
    if let Some(hint) = spec.value_hint {
        arg = arg.value_hint(ValueHint::from(hint));
    }

    if let Some(num_args) = resolved_num_args(name, spec)? {
        arg = apply_num_args(arg, &num_args)?;
    } else if spec.multiple_values {
        arg = arg.num_args(1..);
    }
    arg = arg.action(infer_action(spec));
    Ok(arg)
}

fn takes_values(spec: &ArgSpec) -> bool {
    spec.takes_value
        || spec.multiple_values
        || spec.possible_values.is_some()
        || spec.default_value.is_some()
        || !spec.default_values.is_empty()
        || spec.default_missing_value.is_some()
        || spec.env.is_some()
        || spec.value_name.is_some()
        || !spec.value_names.is_empty()
        || spec.value_delimiter.is_some()
        || spec.value_terminator.is_some()
        || spec.value_hint.is_some()
        || spec.num_args.is_some()
        || spec.number_of_values.is_some()
        || spec.min_values.is_some()
        || spec.max_values.is_some()
        || spec.value_parser.is_some()
        || spec.allow_hyphen_values
        || spec.require_equals
        || spec.short.is_none() && spec.long.is_none()
}

fn infer_action(spec: &ArgSpec) -> ArgAction {
    if let Some(action) = spec.action {
        return action.into();
    }
    let takes_values = takes_values(spec);
    if spec.multiple_values
        || spec.value_delimiter.is_some()
        || (takes_values && spec.multiple_occurrences == Some(true))
    {
        ArgAction::Append
    } else if takes_values {
        ArgAction::Set
    } else if spec.multiple_occurrences == Some(false) {
        ArgAction::SetTrue
    } else {
        ArgAction::Count
    }
}

fn resolved_num_args(name: &str, spec: &ArgSpec) -> Result<Option<NumArgs>, String> {
    let legacy = match (spec.number_of_values, spec.min_values, spec.max_values) {
        (None, None, None) => None,
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            return Err(format!(
                "argument `{name}` combines number_of_values with min_values or max_values"
            ));
        }
        (Some(exact), None, None) => Some(NumArgs::Exact(exact)),
        (None, Some(min), None) => Some(NumArgs::Expr(format!("{min}.."))),
        (None, None, Some(max)) => Some(NumArgs::Expr(format!("..={max}"))),
        (None, Some(min), Some(max)) => Some(NumArgs::Expr(format!("{min}..={max}"))),
    };
    match (&spec.num_args, legacy) {
        (Some(_), Some(_)) => Err(format!(
            "argument `{name}` combines num_args with number_of_values, min_values, or max_values"
        )),
        (Some(num), None) => Ok(Some(num.clone())),
        (None, legacy) => Ok(legacy),
    }
}

fn apply_num_args(arg: Arg, num: &NumArgs) -> Result<Arg, String> {
    match num {
        NumArgs::Exact(exact) => Ok(arg.num_args(to_usize(*exact)?)),
        NumArgs::Expr(expr) => apply_num_args_expr(arg, expr),
    }
}

fn apply_num_args_expr(arg: Arg, expr: &str) -> Result<Arg, String> {
    let expr = expr.trim();
    if let Some((start, end)) = expr.split_once("..=") {
        return match (parse_bound(start)?, parse_bound(end)?) {
            (Some(start), Some(end)) => Ok(arg.num_args(start..=end)),
            (None, Some(end)) => Ok(arg.num_args(..=end)),
            _ => Err(format!("invalid num_args range `{expr}`")),
        };
    }
    if let Some((start, end)) = expr.split_once("..") {
        return match (parse_bound(start)?, parse_bound(end)?) {
            (Some(start), None) => Ok(arg.num_args(start..)),
            (Some(start), Some(end)) => Ok(arg.num_args(start..end)),
            (None, Some(end)) => Ok(arg.num_args(..end)),
            (None, None) => Err(format!("invalid num_args range `{expr}`")),
        };
    }
    let exact: u64 = expr
        .parse()
        .map_err(|_| format!("invalid num_args `{expr}`"))?;
    Ok(arg.num_args(to_usize(exact)?))
}

fn parse_bound(text: &str) -> Result<Option<usize>, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    text.parse::<u64>()
        .map_err(|_| format!("invalid num_args bound `{text}`"))
        .and_then(to_usize)
        .map(Some)
}

fn to_usize(value: u64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("num_args {value} does not fit usize"))
}

fn lossy_os_string(value: OsString) -> String {
    value.to_string_lossy().into_owned()
}

fn bool_as_bit(value: bool) -> String {
    u8::from(value).to_string()
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ActionSpec {
    Set,
    Append,
    SetTrue,
    SetFalse,
    Count,
    Help,
    Version,
    StoreValue,
    IncOccurrence,
}

impl From<ActionSpec> for ArgAction {
    fn from(action: ActionSpec) -> Self {
        match action {
            ActionSpec::Set | ActionSpec::StoreValue => ArgAction::Set,
            ActionSpec::Append => ArgAction::Append,
            ActionSpec::SetTrue => ArgAction::SetTrue,
            ActionSpec::SetFalse => ArgAction::SetFalse,
            ActionSpec::Count | ActionSpec::IncOccurrence => ArgAction::Count,
            ActionSpec::Help => ArgAction::Help,
            ActionSpec::Version => ArgAction::Version,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ValueHintSpec {
    Unknown,
    Other,
    AnyPath,
    FilePath,
    DirPath,
    ExecutablePath,
    CommandName,
    CommandString,
    CommandWithArguments,
    Username,
    Hostname,
    Url,
    EmailAddress,
}

impl From<ValueHintSpec> for ValueHint {
    fn from(hint: ValueHintSpec) -> Self {
        match hint {
            ValueHintSpec::Unknown => ValueHint::Unknown,
            ValueHintSpec::Other => ValueHint::Other,
            ValueHintSpec::AnyPath => ValueHint::AnyPath,
            ValueHintSpec::FilePath => ValueHint::FilePath,
            ValueHintSpec::DirPath => ValueHint::DirPath,
            ValueHintSpec::ExecutablePath => ValueHint::ExecutablePath,
            ValueHintSpec::CommandName => ValueHint::CommandName,
            ValueHintSpec::CommandString => ValueHint::CommandString,
            ValueHintSpec::CommandWithArguments => ValueHint::CommandWithArguments,
            ValueHintSpec::Username => ValueHint::Username,
            ValueHintSpec::Hostname => ValueHint::Hostname,
            ValueHintSpec::Url => ValueHint::Url,
            ValueHintSpec::EmailAddress => ValueHint::EmailAddress,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ValueParserKind {
    String,
    OsString,
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl ValueParserKind {
    fn parser(self) -> ValueParser {
        match self {
            ValueParserKind::String => ValueParser::string(),
            ValueParserKind::OsString => OsStringValueParser::new().map(lossy_os_string).into(),
            ValueParserKind::Bool => BoolishValueParser::new().map(bool_as_bit).into(),
            ValueParserKind::U8 => from_str_parser!(u8),
            ValueParserKind::U16 => from_str_parser!(u16),
            ValueParserKind::U32 => from_str_parser!(u32),
            ValueParserKind::U64 => from_str_parser!(u64),
            ValueParserKind::I8 => from_str_parser!(i8),
            ValueParserKind::I16 => from_str_parser!(i16),
            ValueParserKind::I32 => from_str_parser!(i32),
            ValueParserKind::I64 => from_str_parser!(i64),
            ValueParserKind::F32 => from_str_parser!(f32),
            ValueParserKind::F64 => from_str_parser!(f64),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ColorSpec {
    Auto,
    Always,
    Never,
}

impl From<ColorSpec> for ColorChoice {
    fn from(color: ColorSpec) -> Self {
        match color {
            ColorSpec::Auto => ColorChoice::Auto,
            ColorSpec::Always => ColorChoice::Always,
            ColorSpec::Never => ColorChoice::Never,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum NumArgs {
    Exact(u64),
    Expr(String),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct CommandSpec {
    name: Option<String>,
    bin_name: Option<String>,
    display_name: Option<String>,
    version: Option<String>,
    long_version: Option<String>,
    about: Option<String>,
    long_about: Option<String>,
    author: Option<String>,
    before_help: Option<String>,
    before_long_help: Option<String>,
    after_help: Option<String>,
    after_long_help: Option<String>,
    help_template: Option<String>,
    override_usage: Option<String>,
    subcommand_help_heading: Option<String>,
    next_help_heading: Option<String>,
    alias: Option<String>,
    aliases: Vec<String>,
    visible_alias: Option<String>,
    visible_aliases: Vec<String>,
    #[serde(deserialize_with = "de_opt_char")]
    short_flag: Option<char>,
    #[serde(deserialize_with = "de_opt_char")]
    short_flag_alias: Option<char>,
    #[serde(deserialize_with = "de_chars")]
    short_flag_aliases: Vec<char>,
    #[serde(deserialize_with = "de_opt_char")]
    visible_short_flag_alias: Option<char>,
    #[serde(deserialize_with = "de_chars")]
    visible_short_flag_aliases: Vec<char>,
    long_flag: Option<String>,
    long_flag_alias: Option<String>,
    long_flag_aliases: Vec<String>,
    visible_long_flag_alias: Option<String>,
    visible_long_flag_aliases: Vec<String>,
    arg_required_else_help: bool,
    subcommand_required: bool,
    subcommand_negates_reqs: bool,
    args_conflicts_with_subcommands: bool,
    args_override_self: bool,
    propagate_version: bool,
    infer_subcommands: bool,
    infer_long_args: bool,
    allow_external_subcommands: bool,
    allow_negative_numbers: bool,
    trailing_var_arg: bool,
    dont_delimit_trailing_values: bool,
    disable_help_flag: bool,
    disable_help_subcommand: bool,
    disable_version_flag: bool,
    disable_colored_help: bool,
    next_line_help: bool,
    #[serde(alias = "hidden")]
    hide: bool,
    display_order: Option<usize>,
    term_width: Option<usize>,
    max_term_width: Option<usize>,
    color: Option<ColorSpec>,
    #[serde(deserialize_with = "de_named_vec")]
    args: Vec<(String, ArgSpec)>,
    #[serde(deserialize_with = "de_named_vec")]
    args_map: Vec<(String, ArgSpec)>,
    #[serde(deserialize_with = "de_named_vec")]
    subcommands: Vec<(String, CommandSpec)>,
    #[serde(deserialize_with = "de_named_vec")]
    subcommands_map: Vec<(String, CommandSpec)>,
    #[serde(deserialize_with = "de_named_vec")]
    groups: Vec<(String, GroupSpec)>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ArgSpec {
    help: Option<String>,
    long_help: Option<String>,
    help_heading: Option<String>,
    #[serde(deserialize_with = "de_opt_char")]
    short: Option<char>,
    long: Option<String>,
    alias: Option<String>,
    aliases: Vec<String>,
    visible_alias: Option<String>,
    visible_aliases: Vec<String>,
    #[serde(deserialize_with = "de_opt_char")]
    short_alias: Option<char>,
    #[serde(deserialize_with = "de_chars")]
    short_aliases: Vec<char>,
    #[serde(deserialize_with = "de_opt_char")]
    visible_short_alias: Option<char>,
    #[serde(deserialize_with = "de_chars")]
    visible_short_aliases: Vec<char>,
    #[serde(alias = "arg_action")]
    action: Option<ActionSpec>,
    takes_value: bool,
    multiple_values: bool,
    multiple_occurrences: Option<bool>,
    num_args: Option<NumArgs>,
    number_of_values: Option<u64>,
    min_values: Option<u64>,
    max_values: Option<u64>,
    possible_values: Option<Vec<String>>,
    #[serde(alias = "case_insensitive")]
    ignore_case: bool,
    hide_possible_values: bool,
    value_name: Option<String>,
    value_names: Vec<String>,
    #[serde(deserialize_with = "de_opt_char")]
    value_delimiter: Option<char>,
    value_terminator: Option<String>,
    require_equals: bool,
    default_value: Option<String>,
    default_values: Vec<String>,
    default_missing_value: Option<String>,
    env: Option<String>,
    hide_env: bool,
    hide_env_values: bool,
    required: bool,
    required_unless_present: Option<String>,
    required_unless_present_any: Vec<String>,
    required_unless_present_all: Vec<String>,
    global: bool,
    #[serde(alias = "hidden")]
    hide: bool,
    #[serde(alias = "hidden_long_help")]
    hide_long_help: bool,
    #[serde(alias = "hidden_short_help")]
    hide_short_help: bool,
    exclusive: bool,
    last: bool,
    index: Option<usize>,
    conflicts_with: Option<String>,
    conflicts_with_all: Vec<String>,
    requires: Option<String>,
    requires_all: Vec<String>,
    overrides_with: Option<String>,
    overrides_with_all: Vec<String>,
    allow_hyphen_values: bool,
    group: Option<String>,
    value_hint: Option<ValueHintSpec>,
    value_parser: Option<ValueParserKind>,
    display_order: Option<usize>,
    next_line_help: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct GroupSpec {
    arg: Option<String>,
    args: Vec<String>,
    multiple: Option<bool>,
    required: Option<bool>,
    conflicts_with: Option<String>,
    conflicts_with_all: Vec<String>,
    requires: Option<String>,
    requires_all: Vec<String>,
}

fn de_named_vec<'de, D, T>(deserializer: D) -> Result<Vec<(String, T)>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = yaml_serde::Value::deserialize(deserializer)?;
    named_from_value(value).map_err(de::Error::custom)
}

fn named_from_value<T>(value: yaml_serde::Value) -> Result<Vec<(String, T)>, String>
where
    T: DeserializeOwned + Default,
{
    match value {
        yaml_serde::Value::Null => Ok(Vec::new()),
        yaml_serde::Value::Sequence(items) => items.into_iter().map(one_entry).collect(),
        yaml_serde::Value::Mapping(map) => map
            .into_iter()
            .map(|(key, value)| Ok((value_string(key)?, spec_from_value(value)?)))
            .collect(),
        other => Err(format!(
            "expected a list or a map, got {}",
            value_kind(&other)
        )),
    }
}

fn one_entry<T>(value: yaml_serde::Value) -> Result<(String, T), String>
where
    T: DeserializeOwned + Default,
{
    let yaml_serde::Value::Mapping(map) = value else {
        return Err("each entry must be a single-key map".into());
    };
    if map.len() != 1 {
        return Err("each entry must be a single-key map".into());
    }
    let (key, value) = map.into_iter().next().expect("map length is 1");
    Ok((value_string(key)?, spec_from_value(value)?))
}

fn spec_from_value<T>(value: yaml_serde::Value) -> Result<T, String>
where
    T: DeserializeOwned + Default,
{
    match value {
        yaml_serde::Value::Null => Ok(T::default()),
        other => yaml_serde::from_value(other).map_err(|err| err.to_string()),
    }
}

fn value_string(value: yaml_serde::Value) -> Result<String, String> {
    match value {
        yaml_serde::Value::String(text) => Ok(text),
        other => Err(format!("expected a string, got {}", value_kind(&other))),
    }
}

fn value_kind(value: &yaml_serde::Value) -> &'static str {
    match value {
        yaml_serde::Value::Null => "null",
        yaml_serde::Value::Bool(_) => "a boolean",
        yaml_serde::Value::Number(_) => "a number",
        yaml_serde::Value::String(_) => "a string",
        yaml_serde::Value::Sequence(_) => "a list",
        yaml_serde::Value::Mapping(_) => "a map",
        yaml_serde::Value::Tagged(_) => "a tagged value",
    }
}

fn de_opt_char<'de, D>(deserializer: D) -> Result<Option<char>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = yaml_serde::Value::deserialize(deserializer)?;
    match value {
        yaml_serde::Value::Null => Ok(None),
        yaml_serde::Value::String(text) => one_char(&text).map(Some).map_err(de::Error::custom),
        _ => Err(de::Error::custom(
            "expected a single character; quote YAML 1.1 booleans such as 'y', 'n', 'yes', and 'no'",
        )),
    }
}

fn de_chars<'de, D>(deserializer: D) -> Result<Vec<char>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<yaml_serde::Value>::deserialize(deserializer)?;
    values
        .into_iter()
        .map(|value| match value {
            yaml_serde::Value::String(text) => one_char(&text).map_err(de::Error::custom),
            _ => Err(de::Error::custom(
                "expected a single character; quote YAML 1.1 booleans such as 'y', 'n', 'yes', and 'no'",
            )),
        })
        .collect()
}

fn one_char(text: &str) -> Result<char, String> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) => Ok(ch),
        _ => Err(format!("expected a single character, got {text:?}")),
    }
}
