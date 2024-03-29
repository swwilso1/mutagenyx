//! The `vyper::language_interface` module provides the implementation for the [`JSONLanguageDelegate d`]
//! trait and the function `get_vyper_sub_language_interface`.

use crate::compiler_details::*;
use crate::error::MutagenyxError;
use crate::id::Id;
use crate::json::*;
use crate::json_ast_id_maker::JSONIDMaker;
use crate::json_ast_permitter::JSONPermitter;
use crate::json_comment_inserter::JSONCommentInserter;
use crate::json_language_delegate::JSONLanguageDelegate;
use crate::json_namer::JSONNamer;
use crate::language::Language;
use crate::mutation_visitor::NodePath;
use crate::mutator::*;
use crate::namer::Namer;
use crate::node_printer::NodePrinterFactory;
use crate::permissions::Permissions;
use crate::permit::Permit;
use crate::preferences::Preferences;
use crate::pretty_print_visitor::PrettyPrintVisitor;
use crate::pretty_printer::PrettyPrinter;
use crate::super_ast::SuperAST;
use crate::utility::shell_execute;
use crate::visitor::Visitor;
use crate::vyper::ast::VyperAST;
use crate::vyper::commenter::VyperCommenterFactory;
use crate::vyper::compiler_details::ROOT_PATH_KEY;
use crate::vyper::mutators::VyperMutatorFactory;
use crate::vyper::node_finder::VyperNodeFinderFactory;
use crate::vyper::pretty_printer::VyperNodePrinterFactory;
use serde_json::Value;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use versions::{Mess, Versioning};

/// The type that implements [`JSONLanguageDelegate`]
pub struct VyperLanguageDelegate {
    node_printer_factory: Box<dyn NodePrinterFactory<VyperAST>>,
}

impl VyperLanguageDelegate {
    /// Create a new Vyper language delegate
    pub fn new() -> VyperLanguageDelegate {
        VyperLanguageDelegate {
            node_printer_factory: Box::new(VyperNodePrinterFactory::default()),
        }
    }
}

impl JSONLanguageDelegate for VyperLanguageDelegate {
    fn recover_ast<'a>(&self, super_ast: &'a SuperAST) -> Result<&'a Value, MutagenyxError> {
        let vyper_ast = match super_ast {
            SuperAST::Vyper(sast) => sast,
            _ => return Err(MutagenyxError::ASTTypeNotSupported),
        };
        Ok(vyper_ast)
    }

    fn get_value_as_super_ast(&self, value: Value) -> Result<SuperAST, MutagenyxError> {
        if <VyperLanguageDelegate as JSONLanguageDelegate>::json_is_language_ast_json(self, &value)
        {
            return Ok(SuperAST::Vyper(value));
        }
        Err(MutagenyxError::LanguageNotRecognized)
    }

    fn get_mutator_factory(&self) -> Box<dyn MutatorFactory<Value>> {
        Box::new(VyperMutatorFactory {})
    }

    fn get_pretty_print_visitor<'a>(
        &'a self,
        stream: &'a mut dyn Write,
        printer: &'a mut PrettyPrinter,
    ) -> Box<dyn Visitor<Value> + 'a> {
        return Box::new(PrettyPrintVisitor::new(
            stream,
            printer,
            self.node_printer_factory.as_ref(),
        ));
    }

    fn json_is_language_ast_json(&self, value: &Value) -> bool {
        if value.has_value_for_key("contract_name") && value.has_value_for_key("ast") {
            return true;
        }
        false
    }

    fn convert_source_file_to_ast(
        &self,
        file_name: &str,
        prefs: &Preferences,
    ) -> Result<SuperAST, MutagenyxError> {
        if let Ok(s) = file_is_source_file_with_vyper_from_pip(file_name, prefs) {
            let value = load_json_from_file_with_name(&s)?;
            return <VyperLanguageDelegate as JSONLanguageDelegate>::get_value_as_super_ast(
                self, value,
            );
        }

        if let Ok(s) = file_is_source_file_with_docker(file_name, prefs) {
            let value = load_json_from_file_with_name(&s)?;
            return <VyperLanguageDelegate as JSONLanguageDelegate>::get_value_as_super_ast(
                self, value,
            );
        }

        Err(MutagenyxError::SourceDoesNotCompile(String::from(
            file_name,
        )))
    }

    fn file_is_language_source_file(&self, file_name: &str, prefs: &Preferences) -> bool {
        file_is_source_file_with_vyper_from_pip(file_name, prefs).is_ok()
            || file_is_source_file_with_docker(file_name, prefs).is_ok()
    }

    fn implements(&self) -> Language {
        Language::Vyper
    }

    fn get_file_extension(&self) -> &str {
        "vy"
    }

    fn default_compiler_settings(&self) -> Preferences {
        let mut preferences = Preferences::new();
        preferences.set_string_for_key(PATH_KEY, "vyper");
        preferences
    }

    fn get_node_permitter<'a>(
        &'a self,
        permissions: &'a Permissions,
    ) -> Box<dyn Permit<Value> + '_> {
        Box::new(JSONPermitter::new(permissions))
    }

    fn mutant_compiles(&self, file_name: &str, prefs: &Preferences) -> bool {
        file_compiles(file_name, prefs)
    }

    fn get_node_id_maker(&self) -> Box<dyn Id<Value>> {
        Box::new(JSONIDMaker::new(|n| {
            n.get_int_for_key("node_id").map(|node_id| node_id as u64)
        }))
    }

    fn insert_comment_by_path(&self, ast: &mut Value, comment_node: Value, node_path: &NodePath) {
        let id_maker = self.get_node_id_maker();
        let finder_factory = Box::new(VyperNodeFinderFactory {});
        let commenter_factory = Box::new(VyperCommenterFactory {});
        let json_comment_inserter =
            JSONCommentInserter::new(finder_factory, commenter_factory, id_maker);
        json_comment_inserter.insert_comment_by_path(ast, comment_node, node_path)
    }

    fn get_namer(&self) -> Box<dyn Namer<Value>> {
        Box::new(JSONNamer::new(|v| {
            v.get_str_for_key("name").map(String::from)
        }))
    }
}

/// Retrieve Vyper compiler flags from a [`Preferences`] object.
///
/// # Arguments
///
/// * `prefs` - The [`Preferences`] object.
fn get_vyper_compiler_flags_from_preferences(prefs: &Preferences) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    let language_key = format!["{}", Language::Vyper];
    if let Some(language_prefs) = prefs.get_preferences_for_key(&language_key) {
        if let Some(compiler_prefs) = language_prefs.get_preferences_for_key(COMPILER_KEY) {
            if let Some(root_path) = compiler_prefs.get_string_for_key(ROOT_PATH_KEY) {
                args.push(String::from("-p"));
                args.push(root_path);
            }
        }
    }

    args
}

/// Retrieve the Vyper compiler from a [`Preferences`] object.
///
/// # Arguments
///
/// * `prefs` - The [`Preferences`] containing the compiler path.
fn get_vyper_compiler_from_preferences(prefs: &Preferences) -> String {
    let mut vyper_compiler = String::from("vyper");

    let language_key = format!["{}", Language::Vyper];
    if let Some(language_prefs) = prefs.get_preferences_for_key(&language_key) {
        if let Some(compiler_prefs) = language_prefs.get_preferences_for_key(COMPILER_KEY) {
            if let Some(path) = compiler_prefs.get_string_for_key(PATH_KEY) {
                vyper_compiler = path;
            }
        }
    }

    vyper_compiler
}

/// Invoke the compiler and get it's version information.
///
/// # Arguments
///
/// * `compiler` - The compiler to query.
fn get_vyper_compiler_version(compiler: &str) -> Result<Versioning, MutagenyxError> {
    let discovered_compiler_version = match shell_execute(compiler, vec![String::from("--version")])
    {
        Ok(output) => {
            if output.status.success() {
                let output_version = core::str::from_utf8(output.stdout.as_slice()).unwrap();

                // Clip off the newline character.  The Mess code dislikes the newline.
                let output_version = &output_version[..output_version.len() - 1];

                // We use Mess here because the compiler may output some non-semantic versioned string
                // like `0.2.11+commit.5db35ef` for a compiler version.  Mess gives us a shot at recovering
                // the major/minor/release numbers even when the version string has extra, hard-to-parse
                // contents.
                match Mess::new(output_version) {
                    Some(v) => {
                        let mut vstr = String::new();
                        if let Some(value) = v.nth(0) {
                            vstr += &(value).to_string();
                        }
                        vstr += ".";
                        if let Some(value) = v.nth(1) {
                            vstr += &(value).to_string();
                        }
                        vstr += ".";
                        if let Some(value) = v.nth(2) {
                            vstr += &(value).to_string();
                        }
                        Versioning::new(&vstr).unwrap()
                    }
                    None => return Err(MutagenyxError::CompilerNoVersion(String::from(compiler))),
                }
            } else {
                let command_error = core::str::from_utf8(output.stderr.as_slice()).unwrap();
                log::error!("Unable to retrieve compiler version: {}", command_error);
                return Err(MutagenyxError::CompilerNoVersion(String::from(compiler)));
            }
        }
        Err(e) => {
            log::error!("shell_execute error: {}", e);
            return Err(e);
        }
    };
    Ok(discovered_compiler_version)
}

/// Try to execute the vyper compiler on the command line assuming that the user
/// has installed the pip version of the vyper compiler. Return the path to the
/// file that contains the Vyper AST.
///
/// # Arguments
///
/// * `file_name` - The path to the source file to compile.
/// * `preferences` - The [`Preferences`] object that might contain a value for `vyper_compiler`.
fn file_is_source_file_with_vyper_from_pip(
    file_name: &str,
    preferences: &Preferences,
) -> Result<String, MutagenyxError> {
    // This code does the following:
    // * Tries to determine the Vyper compiler version.
    // * Checks the discovered version against the first know version that supports -o output_file
    // * Tries to generate the source file AST.
    // * If successful, returns the file name of the AST.

    let file_path = PathBuf::from_str(file_name).unwrap();
    let base_name = file_path.file_name().unwrap().to_str().unwrap();
    let tmp_dir = env::temp_dir();
    let mut full_path_to_tmp_file = PathBuf::from(&tmp_dir);

    // The Vyper compiler did not start to support '-o outfile_name' as a command line option until
    // Vyper compiler version 3.0.0.
    let first_version_to_support_dash_o = Versioning::new("3.0.0").unwrap();

    full_path_to_tmp_file.push(String::from(base_name) + ".json");

    let vyper_compiler = get_vyper_compiler_from_preferences(preferences);
    let mut full_compiler_args = get_vyper_compiler_flags_from_preferences(preferences);

    let discovered_compiler_version = get_vyper_compiler_version(&vyper_compiler)?;

    // Now that we know the compiler version, set the appropriate command-line arguments and
    // mark whether or not we should post-process the output.  If the compiler does not support
    // `-o outfile_name`, then we need to take the contents of stdout and write them to the
    // temporary file.
    let post_process_compiler_output_to_file: bool;
    let args = if discovered_compiler_version >= first_version_to_support_dash_o {
        post_process_compiler_output_to_file = false;
        vec![
            "-f",
            "ast",
            "-o",
            full_path_to_tmp_file.to_str().unwrap(),
            file_name,
        ]
    } else {
        post_process_compiler_output_to_file = true;
        vec!["-f", "ast", file_name]
    };

    for arg in args {
        full_compiler_args.push(String::from(arg));
    }

    log::debug!(
        "Invoking Vyper compiler {} with {:?}",
        vyper_compiler,
        full_compiler_args
    );
    match shell_execute(&vyper_compiler, full_compiler_args) {
        Ok(output) => {
            if output.status.success() {
                if post_process_compiler_output_to_file {
                    // The compiler did not support the -o flag to output the AST to a file.  So,
                    // we get the output from stdout and write that to the output file.
                    let ast_contents = core::str::from_utf8(output.stdout.as_slice()).unwrap();
                    let mut file = std::fs::File::create(full_path_to_tmp_file.to_str().unwrap())?;
                    write!(file, "{ast_contents}")?;
                }
                Ok(String::from(full_path_to_tmp_file.to_str().unwrap()))
            } else {
                Err(MutagenyxError::SourceDoesNotCompile(String::from(
                    file_name,
                )))
            }
        }
        Err(_e) => Err(MutagenyxError::SourceDoesNotCompile(String::from(
            file_name,
        ))),
    }
}

/// Try to execute the vyper compiler in a docker container. On success, return the path to
/// the output file that contains the Vyper AST.
///
/// # Arguments
///
/// * `file_name` - The path to the source file to compile.
fn file_is_source_file_with_docker(
    file_name: &str,
    preferences: &Preferences,
) -> Result<String, MutagenyxError> {
    // The docker command to invoke the Vyper compiler requires the path where the source file
    // is located to map into the container /code directory.
    let full_file_path = PathBuf::from_str(file_name).unwrap();
    let base_name = full_file_path.file_name().unwrap().to_str().unwrap();

    let tmp_dir = env::temp_dir();

    let separator = String::from(std::path::MAIN_SEPARATOR);

    let out_path =
        String::from(tmp_dir.to_str().unwrap()) + separator.as_str() + base_name + ".json";

    let mut directory_name = full_file_path.clone();

    // Now get just the directory:
    directory_name.pop();

    let dir_name = directory_name.to_str().unwrap();

    let mut args: Vec<String> = Vec::new();
    args.push(String::from("run"));
    args.push(String::from("-v"));
    args.push(format!("{dir_name}:/code"));
    args.push(String::from("vyperlang/vyper"));
    args.push(String::from("-f"));
    args.push(String::from("ast"));

    let other_compiler_args = get_vyper_compiler_flags_from_preferences(preferences);

    for other_arg in other_compiler_args {
        args.push(other_arg);
    }

    args.push(String::from(file_name));
    args.push(String::from("-o"));
    // Clone here because we need out_path later.
    args.push(out_path.clone());

    log::debug!("Invoking docker Vyper compiler {:?}", args);
    match shell_execute("docker", args) {
        Ok(output) => {
            if output.status.success() {
                Ok(out_path)
            } else {
                Err(MutagenyxError::SourceDoesNotCompile(String::from(
                    file_name,
                )))
            }
        }
        Err(_e) => Err(MutagenyxError::SourceDoesNotCompile(String::from(
            file_name,
        ))),
    }
}

fn file_compiles_with_pip(file_name: &str, prefs: &Preferences) -> bool {
    let vyper_compiler = get_vyper_compiler_from_preferences(prefs);
    let mut full_compiler_args = get_vyper_compiler_flags_from_preferences(prefs);

    full_compiler_args.push(String::from(file_name));

    log::debug!(
        "Attempting to compile {} with Vyper compiler '{}' and args: {:?}",
        file_name,
        vyper_compiler,
        full_compiler_args
    );

    match shell_execute(&vyper_compiler, full_compiler_args) {
        Ok(output) => {
            if !output.status.success() {
                let stdout_contents = core::str::from_utf8(output.stdout.as_slice()).unwrap();
                let stderr_contents = core::str::from_utf8(output.stderr.as_slice()).unwrap();
                log::debug!(
                    "Compilation failed:\n\tstdout: {}\n\tstderr: {}",
                    stdout_contents,
                    stderr_contents
                );
            }
            output.status.success()
        }
        Err(_e) => false,
    }
}

fn file_compiles_with_docker(file_name: &str, prefs: &Preferences) -> bool {
    // The docker command to invoke the Vyper compiler requires the path where the source file
    // is located to map into the container /code directory.
    let mut directory_name = PathBuf::from_str(file_name).unwrap();

    // Now get just the directory:
    directory_name.pop();

    let dir_name = directory_name.to_str().unwrap();

    let mut args: Vec<String> = Vec::new();
    args.push(String::from("run"));
    args.push(String::from("-v"));
    args.push(format!("{dir_name}:/code"));
    args.push(String::from("vyperlang/vyper"));

    let other_compiler_args = get_vyper_compiler_flags_from_preferences(prefs);

    for other_arg in other_compiler_args {
        args.push(other_arg);
    }

    args.push(String::from(file_name));

    log::debug!(
        "Attempting to compile {} with docker Vyper compiler and args: {:?}",
        file_name,
        args
    );

    match shell_execute("docker", args) {
        Ok(output) => {
            if !output.status.success() {
                let stdout_contents = core::str::from_utf8(output.stdout.as_slice()).unwrap();
                let stderr_contents = core::str::from_utf8(output.stderr.as_slice()).unwrap();
                log::debug!(
                    "Compilation failed:\n\tstdout: {}\n\tstderr: {}",
                    stdout_contents,
                    stderr_contents
                );
            }
            output.status.success()
        }
        Err(_e) => false,
    }
}

fn file_compiles(file_name: &str, prefs: &Preferences) -> bool {
    if file_compiles_with_pip(file_name, prefs) {
        return true;
    }

    file_compiles_with_docker(file_name, prefs)
}
