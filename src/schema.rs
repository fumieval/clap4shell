use std::collections::HashSet;
use std::ffi::OsString;

use clap::builder::{
    BoolishValueParser, OsStringValueParser, PossibleValuesParser, TypedValueParser, ValueParser,
};

macro_rules! apply_str {
    ($target:ident, $value:expr, $method:ident) => {
        if let Some(value) = &$value {
            $target = $target.$method(value);
        }
    };
}

macro_rules! apply_char {
    ($target:ident, $value:expr, $method:ident) => {
        if let Some(value) = $value {
            $target = $target.$method(value);
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
use clap::{Arg, ArgAction, ArgGroup, ColorChoice, Command, ValueHint};
use serde::Deserialize;
use serde::de::{self, DeserializeOwned, Deserializer};

pub(crate) fn command_from_yaml(yaml: &str) -> Result<Command, String> {
    let spec: CommandSpec = serde_yaml::from_str(yaml).map_err(|err| err.to_string())?;
    let name = spec
        .name
        .clone()
        .ok_or_else(|| "missing field `name`".to_string())?;
    build_command(&name, &spec)
}

fn build_command(name: &str, spec: &CommandSpec) -> Result<Command, String> {
    let mut cmd = Command::new(name.to_owned());
    apply_str!(cmd, spec.bin_name, bin_name);
    apply_str!(cmd, spec.display_name, display_name);
    apply_str!(cmd, spec.version, version);
    apply_str!(cmd, spec.long_version, long_version);
    apply_str!(cmd, spec.about, about);
    apply_str!(cmd, spec.long_about, long_about);
    apply_str!(cmd, spec.author, author);
    apply_str!(cmd, spec.before_help, before_help);
    apply_str!(cmd, spec.before_long_help, before_long_help);
    apply_str!(cmd, spec.after_help, after_help);
    apply_str!(cmd, spec.after_long_help, after_long_help);
    apply_str!(cmd, spec.help_template, help_template);
    apply_str!(cmd, spec.override_usage, override_usage);
    apply_str!(cmd, spec.subcommand_help_heading, subcommand_help_heading);
    apply_str!(cmd, spec.next_help_heading, next_help_heading);
    apply_str!(cmd, spec.alias, alias);
    apply_str!(cmd, spec.visible_alias, visible_alias);
    apply_str!(cmd, spec.long_flag, long_flag);
    apply_str!(cmd, spec.long_flag_alias, long_flag_alias);
    apply_str!(cmd, spec.visible_long_flag_alias, visible_long_flag_alias);
    apply_char!(cmd, spec.short_flag, short_flag);
    apply_char!(cmd, spec.short_flag_alias, short_flag_alias);
    apply_char!(cmd, spec.visible_short_flag_alias, visible_short_flag_alias);
    if !spec.aliases.is_empty() {
        cmd = cmd.aliases(spec.aliases.iter());
    }
    if !spec.visible_aliases.is_empty() {
        cmd = cmd.visible_aliases(spec.visible_aliases.iter());
    }
    if !spec.long_flag_aliases.is_empty() {
        cmd = cmd.long_flag_aliases(spec.long_flag_aliases.iter());
    }
    if !spec.visible_long_flag_aliases.is_empty() {
        cmd = cmd.visible_long_flag_aliases(spec.visible_long_flag_aliases.iter());
    }
    if !spec.short_flag_aliases.is_empty() {
        cmd = cmd.short_flag_aliases(spec.short_flag_aliases.iter().copied());
    }
    if !spec.visible_short_flag_aliases.is_empty() {
        cmd = cmd.visible_short_flag_aliases(spec.visible_short_flag_aliases.iter().copied());
    }
    cmd = when(cmd, spec.arg_required_else_help, |cmd| {
        cmd.arg_required_else_help(true)
    });
    cmd = when(cmd, spec.subcommand_required, |cmd| {
        cmd.subcommand_required(true)
    });
    cmd = when(cmd, spec.subcommand_negates_reqs, |cmd| {
        cmd.subcommand_negates_reqs(true)
    });
    cmd = when(cmd, spec.args_conflicts_with_subcommands, |cmd| {
        cmd.args_conflicts_with_subcommands(true)
    });
    cmd = when(cmd, spec.args_override_self, |cmd| {
        cmd.args_override_self(true)
    });
    cmd = when(cmd, spec.propagate_version, |cmd| {
        cmd.propagate_version(true)
    });
    cmd = when(cmd, spec.infer_subcommands, |cmd| {
        cmd.infer_subcommands(true)
    });
    cmd = when(cmd, spec.infer_long_args, |cmd| cmd.infer_long_args(true));
    cmd = when(cmd, spec.allow_external_subcommands, |cmd| {
        cmd.allow_external_subcommands(true)
    });
    cmd = when(cmd, spec.allow_negative_numbers, |cmd| {
        cmd.allow_negative_numbers(true)
    });
    cmd = when(cmd, spec.trailing_var_arg, |cmd| cmd.trailing_var_arg(true));
    cmd = when(cmd, spec.dont_delimit_trailing_values, |cmd| {
        cmd.dont_delimit_trailing_values(true)
    });
    cmd = when(cmd, spec.disable_help_flag, |cmd| {
        cmd.disable_help_flag(true)
    });
    cmd = when(cmd, spec.disable_help_subcommand, |cmd| {
        cmd.disable_help_subcommand(true)
    });
    cmd = when(cmd, spec.disable_version_flag, |cmd| {
        cmd.disable_version_flag(true)
    });
    cmd = when(cmd, spec.disable_colored_help, |cmd| {
        cmd.disable_colored_help(true)
    });
    cmd = when(cmd, spec.next_line_help, |cmd| cmd.next_line_help(true));
    cmd = when(cmd, spec.hide, |cmd| cmd.hide(true));
    if let Some(order) = spec.display_order {
        cmd = cmd.display_order(order);
    }
    if let Some(width) = spec.term_width {
        cmd = cmd.term_width(width);
    }
    if let Some(width) = spec.max_term_width {
        cmd = cmd.max_term_width(width);
    }
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
    let mut group = ArgGroup::new(name.to_owned());
    if let Some(arg) = &spec.arg {
        group = group.arg(arg);
    }
    if !spec.args.is_empty() {
        group = group.args(spec.args.iter());
    }
    if let Some(multiple) = spec.multiple {
        group = group.multiple(multiple);
    }
    if let Some(required) = spec.required {
        group = group.required(required);
    }
    if let Some(conflict) = &spec.conflicts_with {
        group = group.conflicts_with(conflict);
    }
    if !spec.conflicts_with_all.is_empty() {
        group = group.conflicts_with_all(spec.conflicts_with_all.iter());
    }
    if let Some(requires) = &spec.requires {
        group = group.requires(requires);
    }
    if !spec.requires_all.is_empty() {
        group = group.requires_all(spec.requires_all.iter());
    }
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

    let mut arg = Arg::new(name.to_owned());
    apply_str!(arg, spec.help, help);
    apply_str!(arg, spec.long_help, long_help);
    apply_str!(arg, spec.help_heading, help_heading);
    apply_str!(arg, spec.long, long);
    apply_str!(arg, spec.alias, alias);
    apply_str!(arg, spec.visible_alias, visible_alias);
    apply_str!(arg, spec.value_name, value_name);
    apply_str!(arg, spec.env, env);
    apply_str!(arg, spec.default_value, default_value);
    apply_str!(arg, spec.default_missing_value, default_missing_value);
    apply_str!(arg, spec.value_terminator, value_terminator);
    apply_str!(arg, spec.conflicts_with, conflicts_with);
    apply_str!(arg, spec.requires, requires);
    apply_str!(arg, spec.overrides_with, overrides_with);
    apply_str!(arg, spec.group, group);
    apply_str!(arg, spec.required_unless_present, required_unless_present);
    apply_char!(arg, spec.short, short);
    apply_char!(arg, spec.short_alias, short_alias);
    apply_char!(arg, spec.visible_short_alias, visible_short_alias);
    apply_char!(arg, spec.value_delimiter, value_delimiter);
    if !spec.aliases.is_empty() {
        arg = arg.aliases(spec.aliases.iter());
    }
    if !spec.visible_aliases.is_empty() {
        arg = arg.visible_aliases(spec.visible_aliases.iter());
    }
    if !spec.short_aliases.is_empty() {
        arg = arg.short_aliases(spec.short_aliases.iter().copied());
    }
    if !spec.visible_short_aliases.is_empty() {
        arg = arg.visible_short_aliases(spec.visible_short_aliases.iter().copied());
    }
    if !spec.value_names.is_empty() {
        arg = arg.value_names(spec.value_names.iter());
    }
    if !spec.default_values.is_empty() {
        arg = arg.default_values(spec.default_values.iter());
    }
    if !spec.conflicts_with_all.is_empty() {
        arg = arg.conflicts_with_all(spec.conflicts_with_all.iter());
    }
    if !spec.requires_all.is_empty() {
        for required in &spec.requires_all {
            arg = arg.requires(required);
        }
    }
    if !spec.overrides_with_all.is_empty() {
        arg = arg.overrides_with_all(spec.overrides_with_all.iter());
    }
    if !spec.required_unless_present_any.is_empty() {
        arg = arg.required_unless_present_any(spec.required_unless_present_any.iter());
    }
    if !spec.required_unless_present_all.is_empty() {
        arg = arg.required_unless_present_all(spec.required_unless_present_all.iter());
    }
    if let Some(values) = &spec.possible_values {
        arg = arg.value_parser(PossibleValuesParser::new(values.clone()));
    } else if let Some(kind) = spec.value_parser {
        arg = arg.value_parser(kind.parser());
    }
    if let Some(hint) = spec.value_hint {
        arg = arg.value_hint(ValueHint::from(hint));
    }
    if let Some(index) = spec.index {
        arg = arg.index(index);
    }
    if let Some(order) = spec.display_order {
        arg = arg.display_order(order);
    }
    arg = when(arg, spec.ignore_case, |arg| arg.ignore_case(true));
    arg = when(arg, spec.hide_possible_values, |arg| {
        arg.hide_possible_values(true)
    });
    arg = when(arg, spec.hide_env, |arg| arg.hide_env(true));
    arg = when(arg, spec.hide_env_values, |arg| arg.hide_env_values(true));
    arg = when(arg, spec.global, |arg| arg.global(true));
    arg = when(arg, spec.hide, |arg| arg.hide(true));
    arg = when(arg, spec.hide_long_help, |arg| arg.hide_long_help(true));
    arg = when(arg, spec.hide_short_help, |arg| arg.hide_short_help(true));
    arg = when(arg, spec.exclusive, |arg| arg.exclusive(true));
    arg = when(arg, spec.last, |arg| arg.last(true));
    arg = when(arg, spec.allow_hyphen_values, |arg| {
        arg.allow_hyphen_values(true)
    });
    arg = when(arg, spec.require_equals, |arg| arg.require_equals(true));
    arg = when(arg, spec.next_line_help, |arg| arg.next_line_help(true));
    arg = arg.required(spec.required);

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

fn when<T>(value: T, yes: bool, f: impl FnOnce(T) -> T) -> T {
    if yes { f(value) } else { value }
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
    let value = serde_yaml::Value::deserialize(deserializer)?;
    named_from_value(value).map_err(de::Error::custom)
}

fn named_from_value<T>(value: serde_yaml::Value) -> Result<Vec<(String, T)>, String>
where
    T: DeserializeOwned + Default,
{
    match value {
        serde_yaml::Value::Null => Ok(Vec::new()),
        serde_yaml::Value::Sequence(items) => items.into_iter().map(one_entry).collect(),
        serde_yaml::Value::Mapping(map) => map
            .into_iter()
            .map(|(key, value)| Ok((value_string(key)?, spec_from_value(value)?)))
            .collect(),
        other => Err(format!(
            "expected a list or a map, got {}",
            value_kind(&other)
        )),
    }
}

fn one_entry<T>(value: serde_yaml::Value) -> Result<(String, T), String>
where
    T: DeserializeOwned + Default,
{
    let serde_yaml::Value::Mapping(map) = value else {
        return Err("each entry must be a single-key map".into());
    };
    if map.len() != 1 {
        return Err("each entry must be a single-key map".into());
    }
    let (key, value) = map.into_iter().next().expect("map length is 1");
    Ok((value_string(key)?, spec_from_value(value)?))
}

fn spec_from_value<T>(value: serde_yaml::Value) -> Result<T, String>
where
    T: DeserializeOwned + Default,
{
    match value {
        serde_yaml::Value::Null => Ok(T::default()),
        other => serde_yaml::from_value(other).map_err(|err| err.to_string()),
    }
}

fn value_string(value: serde_yaml::Value) -> Result<String, String> {
    match value {
        serde_yaml::Value::String(text) => Ok(text),
        other => Err(format!("expected a string, got {}", value_kind(&other))),
    }
}

fn value_kind(value: &serde_yaml::Value) -> &'static str {
    match value {
        serde_yaml::Value::Null => "null",
        serde_yaml::Value::Bool(_) => "a boolean",
        serde_yaml::Value::Number(_) => "a number",
        serde_yaml::Value::String(_) => "a string",
        serde_yaml::Value::Sequence(_) => "a list",
        serde_yaml::Value::Mapping(_) => "a map",
        serde_yaml::Value::Tagged(_) => "a tagged value",
    }
}

fn de_opt_char<'de, D>(deserializer: D) -> Result<Option<char>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_yaml::Value::deserialize(deserializer)?;
    match value {
        serde_yaml::Value::Null => Ok(None),
        serde_yaml::Value::String(text) => one_char(&text).map(Some).map_err(de::Error::custom),
        _ => Err(de::Error::custom(
            "expected a single character; quote YAML 1.1 booleans such as 'y', 'n', 'yes', and 'no'",
        )),
    }
}

fn de_chars<'de, D>(deserializer: D) -> Result<Vec<char>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<serde_yaml::Value>::deserialize(deserializer)?;
    values
        .into_iter()
        .map(|value| match value {
            serde_yaml::Value::String(text) => one_char(&text).map_err(de::Error::custom),
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
