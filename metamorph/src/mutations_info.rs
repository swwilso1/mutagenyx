//! The `mutations_info` module provides code to output the list of supported mutations and
//! descriptions of the supported mutations.

use crate::AlgorithmsCLArgs;
use metamorph_lib::mutation::{all_algorithm_descriptions, MutationType};
use metamorph_lib::pretty_printer::{
    write_flowable_text, write_indent, write_newline, write_space, write_spaces, write_token,
    write_tokens, PrettyPrinter,
};
use std::io::{stdout, Write};
use termsize;

/// Simple struct for keeping track of the width of the mutation algorithm name
/// and the width of the category heading text.
struct ColumnSizes {
    /// Length of the longest algorithm name.
    alg_name_width: usize,

    /// Length of the longest category name.
    category_width: usize,
}

impl ColumnSizes {
    /// Create a new column sizes object.
    fn new() -> ColumnSizes {
        ColumnSizes {
            alg_name_width: 0,
            category_width: 0,
        }
    }
}

/// Function that displays either/or the mutation algorithm documentation.
pub fn display_mutations_info(params: AlgorithmsCLArgs) {
    if params.list {
        display_simple_algorithm_list();

        // If we list the short version of the algorithms do not display the longer algorithm
        // descriptions, even if the params.describe is true.
        return;
    }

    if params.describe {
        display_algorithm_descriptions();
    }
}

fn get_terminal_size() -> termsize::Size {
    // Get the console dimensions.
    if let Some(size) = termsize::get() {
        size
    } else {
        // We get to this case if the tool runs in a non-terminal window (ie as part of a script
        // running somewhere in a workflow.
        termsize::Size {
            // In this use case, rows doesn't matter, we only need a semi-valid cols value.
            rows: 50,
            cols: 80,
        }
    }
}

/// Function to write the short list of the algorithm names and the summary text for each algorithm
/// to stdout.
fn display_simple_algorithm_list() {
    let terminal_size = get_terminal_size();

    // Use a pretty printer for formatting the output.  The pretty printer gives us one advantage
    // over just writing basic formatted strings to the console.  The pretty printer can flow text
    // around an indentation length.  This feature allows us to not have to worry about whether or
    // not the formatting specifications will correctly handle long text descriptions in the
    // MutationAlgorithmDefinition structures.
    let mut printer = PrettyPrinter::new(1, terminal_size.cols as usize);

    let descriptions = all_algorithm_descriptions();

    let mut arg_name_width: usize = 0;

    // Get the sorted keys for the map.  We want them sorted so we can write out the algorithms
    // in alphabetical order(for now).  First, we determine the longest algorithm name.
    let mut key_list: Vec<&MutationType> = vec![];
    for key in descriptions.keys() {
        key_list.push(key);
        let key_length = key.to_string().len();
        if key_length > arg_name_width {
            arg_name_width = key_length;
        }
    }

    // Now sort the algorithms by string contents. This sort should produce an alphabetical sort
    // of the names.
    key_list.sort_unstable_by(|a, b| {
        let a_str = a.to_string();
        let b_str = b.to_string();
        a_str.cmp(&b_str)
    });

    let mut stream = stdout();

    let mut i: usize = 0;

    while i < key_list.len() {
        let algorithm_name = key_list[i].to_string();
        let description = &descriptions[key_list[i]];

        // Write the algorithm name.
        write_token(&mut printer, &mut stream, &algorithm_name);

        // Write remaining spaces to pad the algorithm name to max_algorithm_name_length.
        write_spaces(
            &mut printer,
            &mut stream,
            arg_name_width - algorithm_name.len(),
        );

        write_space(&mut printer, &mut stream);

        printer.increase_indent_by(arg_name_width + 1);
        write_flowable_text(&mut printer, &mut stream, description.summary, "");
        printer.decrease_indent_by(arg_name_width + 1);

        if i < (key_list.len() - 1) {
            write_newline(&mut printer, &mut stream);
            write_newline(&mut printer, &mut stream);
        }
        i += 1;
    }

    write_newline(&mut printer, &mut stream);
}

/// Function to display the detailed (ie more verbose) description of the mutation algorithms.
fn display_algorithm_descriptions() {
    // Get the console dimensions.
    let terminal_size = get_terminal_size();

    // Use a pretty printer for formatting the output.  The pretty printer gives us one advantage
    // over just writing basic formatted strings to the console.  The pretty printer can flow text
    // around an indentation length.  This feature allows us to not have to worry about whether or
    // not the formatting specifications will correctly handle long text descriptions in the
    // MutationAlgorithmDefinition structures.
    let mut printer = PrettyPrinter::new(1, terminal_size.cols as usize);

    let descriptions = all_algorithm_descriptions();

    let operators_text = "Operators:";
    let examples_text = "Examples:";

    let mut column_sizes = ColumnSizes::new();
    column_sizes.category_width = operators_text.len();

    // Get the sorted keys for the map.  We want them sorted so we can write out the algorithms
    // in alphabetical order(for now).
    let mut key_list: Vec<&MutationType> = vec![];
    for key in descriptions.keys() {
        key_list.push(key);
        let key_length = key.to_string().len();
        if key_length > column_sizes.alg_name_width {
            column_sizes.alg_name_width = key_length;
        }
    }

    // Alphabetically sort the algorithm names.
    key_list.sort_unstable_by(|a, b| {
        let a_str = a.to_string();
        let b_str = b.to_string();
        a_str.cmp(&b_str)
    });

    let mut stream = stdout();

    let mut i: usize = 0;

    while i < key_list.len() {
        let algorithm_name = key_list[i].to_string();
        let description = &descriptions[key_list[i]];

        // Write the algorithm name.
        write_token(&mut printer, &mut stream, &algorithm_name);

        // Write remaining spaces to pad the algorithm name to max_algorithm_name_length.
        write_spaces(
            &mut printer,
            &mut stream,
            column_sizes.alg_name_width - algorithm_name.len(),
        );

        write_space(&mut printer, &mut stream);
        write_lines(
            &mut printer,
            &mut stream,
            "",
            description.summary,
            &column_sizes,
            false,
        );

        if !description.extra_details.is_empty() {
            // Start the next line.
            write_newline(&mut printer, &mut stream);
            write_newline(&mut printer, &mut stream);

            write_lines(
                &mut printer,
                &mut stream,
                "",
                description.extra_details,
                &column_sizes,
                true,
            );
        }

        if !description.operators.is_empty() {
            // Start the next line.
            write_newline(&mut printer, &mut stream);
            write_newline(&mut printer, &mut stream);

            let operators_string = format!("{:?}", description.operators);
            write_lines(
                &mut printer,
                &mut stream,
                operators_text,
                &operators_string,
                &column_sizes,
                true,
            );
        }

        if !description.examples.is_empty() {
            // Start the next line.
            write_newline(&mut printer, &mut stream);
            write_newline(&mut printer, &mut stream);

            write_lines(
                &mut printer,
                &mut stream,
                examples_text,
                description.examples,
                &column_sizes,
                true,
            );
        }

        if i < (key_list.len() - 1) {
            write_newline(&mut printer, &mut stream);
            write_newline(&mut printer, &mut stream);
            write_newline(&mut printer, &mut stream);
        }
        i += 1;
    }
    write_newline(&mut printer, &mut stream);
}

/// Helper function to write out a sub-portion of the long algorithm description.
///
/// # Arguments
///
/// * `printer` - The [`PrettyPrinter`] object that will write formatted text to `stream`.
/// * `stream` - The [`Write`] object that will receive the formatted text.
/// * `category_text` - The next for the category heading. This may be an empty string.
/// * `text` - The text containing the content of the category.
/// * `column_sizes` - The size object containing various important widths to use for formatting
/// the category.
/// * `do_indent` - True if the function should write out an indent and a space.  The function
/// should write the indent if it is only writing a line without a mutation algorithm name.
fn write_lines<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    category_text: &str,
    text: &str,
    column_sizes: &ColumnSizes,
    do_indent: bool,
) {
    printer.increase_indent_by(column_sizes.alg_name_width);
    if do_indent {
        write_indent(printer, stream);
        write_space(printer, stream);
    }

    if !category_text.is_empty() {
        write_token(printer, stream, category_text);

        write_newline(printer, stream);
        write_indent(printer, stream);
        write_space(printer, stream);

        write_tokens(printer, stream, "-", category_text.len());

        write_newline(printer, stream);
        write_newline(printer, stream);
        write_indent(printer, stream);
        write_space(printer, stream);
    }

    printer.increase_indent();
    write_flowable_text(printer, stream, text, "");
    printer.decrease_indent_by(column_sizes.alg_name_width + 1);
}
