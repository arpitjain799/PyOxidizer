// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/*! Functionality related to running Python interpreters. */

use {
    crate::resource::BytecodeOptimizationLevel,
    std::{convert::TryFrom, ffi::OsString, os::raw::c_ulong, path::PathBuf, str::FromStr},
};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Defines the profile to use to configure a Python interpreter.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum PythonInterpreterProfile {
    /// Python is isolated from the system.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#isolated-configuration>.
    Isolated,

    /// Python interpreter behaves like `python`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#python-configuration>.
    Python,
}

impl Default for PythonInterpreterProfile {
    fn default() -> Self {
        PythonInterpreterProfile::Isolated
    }
}

impl ToString for PythonInterpreterProfile {
    fn to_string(&self) -> String {
        match self {
            Self::Isolated => "isolated",
            Self::Python => "python",
        }
        .to_string()
    }
}

impl From<PythonInterpreterProfile> for String {
    fn from(v: PythonInterpreterProfile) -> Self {
        v.to_string()
    }
}

impl TryFrom<&str> for PythonInterpreterProfile {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "isolated" => Ok(Self::Isolated),
            "python" => Ok(Self::Python),
            _ => Err(format!(
                "{} is not a valid profile; use 'isolated' or 'python'",
                value
            )),
        }
    }
}

impl TryFrom<String> for PythonInterpreterProfile {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Defines `terminfo`` database resolution semantics.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum TerminfoResolution {
    /// Resolve `terminfo` database using appropriate behavior for current OS.
    Dynamic,
    /// Do not attempt to resolve the `terminfo` database. Basically a no-op.
    None,
    /// Use a specified string as the `TERMINFO_DIRS` value.
    Static(String),
}

impl ToString for TerminfoResolution {
    fn to_string(&self) -> String {
        match self {
            Self::Dynamic => "dynamic".to_string(),
            Self::None => "none".to_string(),
            Self::Static(value) => format!("static:{}", value),
        }
    }
}

impl From<TerminfoResolution> for String {
    fn from(t: TerminfoResolution) -> Self {
        t.to_string()
    }
}

impl TryFrom<&str> for TerminfoResolution {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "dynamic" {
            Ok(Self::Dynamic)
        } else if value == "none" {
            Ok(Self::None)
        } else if let Some(suffix) = value.strip_prefix("static:") {
            Ok(Self::Static(suffix.to_string()))
        } else {
            Err(format!(
                "{} is not a valid terminfo resolution value",
                value
            ))
        }
    }
}

impl TryFrom<String> for TerminfoResolution {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Defines a backend for a memory allocator.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum MemoryAllocatorBackend {
    /// The default allocator as configured by Python.
    Default,
    /// Use jemalloc.
    Jemalloc,
    /// Use Mimalloc.
    Mimalloc,
    /// Use Snmalloc.
    Snmalloc,
    /// Use Rust's global allocator.
    Rust,
}

impl Default for MemoryAllocatorBackend {
    fn default() -> Self {
        if cfg!(windows) {
            Self::Default
        } else {
            Self::Jemalloc
        }
    }
}

impl ToString for MemoryAllocatorBackend {
    fn to_string(&self) -> String {
        match self {
            Self::Default => "default",
            Self::Jemalloc => "jemalloc",
            Self::Mimalloc => "mimalloc",
            Self::Snmalloc => "snmalloc",
            Self::Rust => "rust",
        }
        .to_string()
    }
}

impl From<MemoryAllocatorBackend> for String {
    fn from(v: MemoryAllocatorBackend) -> Self {
        v.to_string()
    }
}

impl TryFrom<&str> for MemoryAllocatorBackend {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "default" => Ok(Self::Default),
            "jemalloc" => Ok(Self::Jemalloc),
            "mimalloc" => Ok(Self::Mimalloc),
            "snmalloc" => Ok(Self::Snmalloc),
            "rust" => Ok(Self::Rust),
            _ => Err(format!("{} is not a valid memory allocator backend", value)),
        }
    }
}

impl TryFrom<String> for MemoryAllocatorBackend {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Holds values for coerce_c_locale.
///
/// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.coerce_c_locale>.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum CoerceCLocale {
    #[allow(clippy::upper_case_acronyms)]
    LCCtype = 1,
    C = 2,
}

impl ToString for CoerceCLocale {
    fn to_string(&self) -> String {
        match self {
            Self::LCCtype => "LC_CTYPE",
            Self::C => "C",
        }
        .to_string()
    }
}

impl From<CoerceCLocale> for String {
    fn from(v: CoerceCLocale) -> Self {
        v.to_string()
    }
}

impl TryFrom<&str> for CoerceCLocale {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "LC_CTYPE" => Ok(Self::LCCtype),
            "C" => Ok(Self::C),
            _ => Err(format!("{} is not a valid C locale coercion value", value)),
        }
    }
}

impl TryFrom<String> for CoerceCLocale {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Defines what to do when comparing bytes with str.
///
/// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.bytes_warning>.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum BytesWarning {
    None = 0,
    Warn = 1,
    Raise = 2,
}

impl ToString for BytesWarning {
    fn to_string(&self) -> String {
        match self {
            Self::None => "none",
            Self::Warn => "warn",
            Self::Raise => "raise",
        }
        .to_string()
    }
}

impl From<BytesWarning> for String {
    fn from(v: BytesWarning) -> Self {
        v.to_string()
    }
}

impl TryFrom<&str> for BytesWarning {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "warn" => Ok(Self::Warn),
            "raise" => Ok(Self::Raise),
            _ => Err(format!("{} is not a valid bytes warning value", value)),
        }
    }
}

impl TryFrom<String> for BytesWarning {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<i32> for BytesWarning {
    fn from(value: i32) -> BytesWarning {
        match value {
            0 => Self::None,
            1 => Self::Warn,
            _ => Self::Raise,
        }
    }
}

/// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.check_hash_pycs_mode>.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum CheckHashPycsMode {
    Always,
    Never,
    Default,
}

impl ToString for CheckHashPycsMode {
    fn to_string(&self) -> String {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::Default => "default",
        }
        .to_string()
    }
}

impl From<CheckHashPycsMode> for String {
    fn from(v: CheckHashPycsMode) -> Self {
        v.to_string()
    }
}

impl TryFrom<&str> for CheckHashPycsMode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            "default" => Ok(Self::Default),
            _ => Err(format!(
                "{} is not a valid check hash pycs mode value",
                value
            )),
        }
    }
}

impl TryFrom<String> for CheckHashPycsMode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.allocator>.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum Allocator {
    NotSet = 0,
    Default = 1,
    Debug = 2,
    Malloc = 3,
    MallocDebug = 4,
    PyMalloc = 5,
    PyMallocDebug = 6,
}

impl ToString for Allocator {
    fn to_string(&self) -> String {
        match self {
            Self::NotSet => "not-set",
            Self::Default => "default",
            Self::Debug => "debug",
            Self::Malloc => "malloc",
            Self::MallocDebug => "malloc-debug",
            Self::PyMalloc => "py-malloc",
            Self::PyMallocDebug => "py-malloc-debug",
        }
        .to_string()
    }
}

impl From<Allocator> for String {
    fn from(v: Allocator) -> Self {
        v.to_string()
    }
}

impl TryFrom<&str> for Allocator {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not-set" => Ok(Self::NotSet),
            "default" => Ok(Self::Default),
            "debug" => Ok(Self::Debug),
            "malloc" => Ok(Self::Malloc),
            "malloc-debug" => Ok(Self::MallocDebug),
            "py-malloc" => Ok(Self::PyMalloc),
            "py-malloc-debug" => Ok(Self::PyMallocDebug),
            _ => Err(format!("{} is not a valid allocator value", value)),
        }
    }
}

impl TryFrom<String> for Allocator {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Defines how to call `multiprocessing.set_start_method()` when `multiprocessing` is imported.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(try_from = "String", into = "String"))]
pub enum MultiprocessingStartMethod {
    /// Do not call `multiprocessing.set_start_method()`.
    None,
    /// Call with value `fork`.
    Fork,
    /// Call with value `forkserver`
    ForkServer,
    /// Call with value `spawn`
    Spawn,
    /// Call with a valid appropriate for the given environment.
    Auto,
}

impl ToString for MultiprocessingStartMethod {
    fn to_string(&self) -> String {
        match self {
            Self::None => "none",
            Self::Fork => "fork",
            Self::ForkServer => "forkserver",
            Self::Spawn => "spawn",
            Self::Auto => "auto",
        }
        .to_string()
    }
}

impl From<MultiprocessingStartMethod> for String {
    fn from(v: MultiprocessingStartMethod) -> Self {
        v.to_string()
    }
}

impl FromStr for MultiprocessingStartMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "fork" => Ok(Self::Fork),
            "forkserver" => Ok(Self::ForkServer),
            "spawn" => Ok(Self::Spawn),
            "auto" => Ok(Self::Auto),
            _ => Err(format!("{} is not a valid multiprocessing start method", s)),
        }
    }
}

impl TryFrom<&str> for MultiprocessingStartMethod {
    type Error = String;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        Self::from_str(v)
    }
}

impl TryFrom<String> for MultiprocessingStartMethod {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Holds configuration of a Python interpreter.
///
/// This struct holds fields that are exposed by `PyPreConfig` and
/// `PyConfig` in the CPython API.
///
/// Other than the profile (which is used to initialize instances of
/// `PyPreConfig` and `PyConfig`), all fields are optional. Only fields
/// with `Some(T)` will be updated from the defaults.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(default))]
pub struct PythonInterpreterConfig {
    /// Profile to use to initialize pre-config and config state of interpreter.
    pub profile: PythonInterpreterProfile,

    // The following fields are from PyPreConfig or are shared with PyConfig.
    /// Name of the memory allocator.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.allocator>.
    pub allocator: Option<Allocator>,

    /// Whether to set the LC_CTYPE locale to the user preferred locale.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.configure_locale>.
    pub configure_locale: Option<bool>,

    /// How to coerce the locale settings.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.coerce_c_locale>.
    pub coerce_c_locale: Option<CoerceCLocale>,

    /// Whether to emit a warning if the C locale is coerced.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.coerce_c_locale_warn>.
    pub coerce_c_locale_warn: Option<bool>,

    /// Whether to enable Python development mode.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.dev_mode>.
    pub development_mode: Option<bool>,

    /// Isolated mode.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.isolated>.
    pub isolated: Option<bool>,

    /// Whether to use legacy filesystem encodings on Windows.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.legacy_windows_fs_encoding>.
    pub legacy_windows_fs_encoding: Option<bool>,

    /// Whether argv should be parsed the way `python` parses them.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.parse_argv>.
    pub parse_argv: Option<bool>,

    /// Whether environment variables are read to control the interpreter configuration.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.use_environment>.
    pub use_environment: Option<bool>,

    /// Controls Python UTF-8 mode.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyPreConfig.utf8_mode>.
    pub utf8_mode: Option<bool>,
    // The following fields are from PyConfig.
    /// Command line arguments.
    ///
    /// These will become `sys.argv`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.argv>.
    pub argv: Option<Vec<OsString>>,

    /// Controls `sys.base_exec_prefix`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.base_exec_prefix>.
    pub base_exec_prefix: Option<PathBuf>,

    /// Controls `sys._base_executable`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.base_executable>.
    pub base_executable: Option<PathBuf>,

    /// Controls `sys.base_prefix`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.base_prefix>.
    pub base_prefix: Option<PathBuf>,

    /// Controls buffering on `stdout` and `stderr`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.buffered_stdio>.
    pub buffered_stdio: Option<bool>,

    /// Controls warnings/errors for some bytes type coercions.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.bytes_warning>.
    pub bytes_warning: Option<BytesWarning>,

    /// Validation mode for `.pyc` files.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.check_hash_pycs_mode>.
    pub check_hash_pycs_mode: Option<CheckHashPycsMode>,

    /// Controls binary mode and buffering on C standard streams.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.configure_c_stdio>.
    pub configure_c_stdio: Option<bool>,

    /// Dump Python references.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.dump_refs>.
    pub dump_refs: Option<bool>,

    /// Controls `sys.exec_prefix`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.exec_prefix>.
    pub exec_prefix: Option<PathBuf>,

    /// Controls `sys.executable`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.executable>.
    pub executable: Option<PathBuf>,

    /// Enable `faulthandler`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.faulthandler>.
    pub fault_handler: Option<bool>,

    /// Controls the encoding to use for filesystems/paths.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.filesystem_encoding>.
    pub filesystem_encoding: Option<String>,

    /// Filesystem encoding error handler.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.filesystem_errors>.
    pub filesystem_errors: Option<String>,

    /// Randomized hash function seed.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.hash_seed>.
    pub hash_seed: Option<c_ulong>,

    /// Python home directory.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.home>.
    pub home: Option<PathBuf>,

    /// Whether to profile `import` time.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.import_time>.
    pub import_time: Option<bool>,

    /// Enter interactive mode after executing a script or a command.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.inspect>.
    pub inspect: Option<bool>,

    /// Whether to install Python signal handlers.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.install_signal_handlers>.
    pub install_signal_handlers: Option<bool>,

    /// Whether to enable the interactive REPL mode.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.interactive>.
    pub interactive: Option<bool>,

    /// Controls legacy stdio behavior on Windows.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.legacy_windows_stdio>.
    pub legacy_windows_stdio: Option<bool>,

    /// Whether to dump statistics from the `pymalloc` allocator on exit.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.malloc_stats>.
    pub malloc_stats: Option<bool>,

    /// Defines `sys.path`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.module_search_paths>.
    pub module_search_paths: Option<Vec<PathBuf>>,

    /// Bytecode optimization level.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.optimization_level>.
    pub optimization_level: Option<BytecodeOptimizationLevel>,

    /// Parser debug mode.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.parser_debug>.
    pub parser_debug: Option<bool>,

    /// Whether calculating the Python path configuration can emit warnings.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.pathconfig_warnings>.
    pub pathconfig_warnings: Option<bool>,

    /// Defines `sys.prefix`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.prefix>.
    pub prefix: Option<PathBuf>,

    /// Program named used to initialize state during path configuration.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.program_name>.
    pub program_name: Option<PathBuf>,

    /// Directory where `.pyc` files are written.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.pycache_prefix>.
    pub pycache_prefix: Option<PathBuf>,

    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.pythonpath_env>.
    pub python_path_env: Option<String>,

    /// Quiet mode.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.quiet>.
    pub quiet: Option<bool>,

    /// Value of the `-c` command line option.
    ///
    /// Effectively defines Python code to evaluate in `Py_RunMain()`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.run_command>.
    pub run_command: Option<String>,

    /// Filename passed on the command line.
    ///
    /// Effectively defines the Python file to run in `Py_RunMain()`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.run_filename>.
    pub run_filename: Option<PathBuf>,

    /// Value of the `-m` command line option.
    ///
    /// Effectively defines the Python module to run as `__main__` in `Py_RunMain()`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.run_module>.
    pub run_module: Option<String>,

    /// Whether to show the total reference count at exit.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.show_ref_count>.
    pub show_ref_count: Option<bool>,

    /// Whether to import the `site` module at startup.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.site_import>.
    pub site_import: Option<bool>,

    /// Whether to skip the first line of [Self::run_filename].
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.skip_source_first_line>.
    pub skip_first_source_line: Option<bool>,

    /// Encoding of `sys.stdout`, `sys.stderr`, and `sys.stdin`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.stdio_encoding>.
    pub stdio_encoding: Option<String>,

    /// Encoding error handler for `sys.stdout` and `sys.stdin`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.stdio_errors>.
    pub stdio_errors: Option<String>,

    /// Whether to enable `tracemalloc`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.tracemalloc>.
    pub tracemalloc: Option<bool>,

    /// Whether to add the user site directory to `sys.path`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.user_site_directory>.
    pub user_site_directory: Option<bool>,

    /// Verbose mode.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.verbose>.
    pub verbose: Option<bool>,

    /// Options of the `warning` module to control behavior.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.warnoptions>.
    pub warn_options: Option<Vec<String>>,

    /// Controls `sys.dont_write_bytecode`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.write_bytecode>.
    pub write_bytecode: Option<bool>,

    /// Values of the `-X` command line options / `sys._xoptions`.
    ///
    /// See <https://docs.python.org/3/c-api/init_config.html#c.PyConfig.xoptions>.
    pub x_options: Option<Vec<String>>,
}
