//! The `mutations_info` module provides code to output the list of supported mutations and
//! descriptions of the supported mutations.

use crate::AlgorithmsCLArgs;
use metamorph_lib::mutation::{
    all_algorithm_descriptions, get_all_mutation_algorithms, MutationType,
};
use metamorph_lib::pretty_printer::{
    write_flowable_text, write_indent, write_newline, write_space, write_spaces, write_token,
    PrettyPrinter,
};
use std::io::{stdout, Write};
use termsize;

struct ColumnSizes {
    arg_name_width: usize,
    category_width: usize,
}

impl ColumnSizes {
    fn new() -> ColumnSizes {
        ColumnSizes {
            arg_name_width: 0,
            category_width: 0,
        }
    }

    fn total_column_length(&self) -> usize {
        self.arg_name_width + self.category_width
    }
}

pub fn display_mutations_info(params: AlgorithmsCLArgs) {
    if params.list {
        for mutation_type in get_all_mutation_algorithms() {
            println!("{}", mutation_type);
        }
    }

    if params.describe {
        display_algorithm_descriptions();
    }
}

fn display_algorithm_descriptions() {
    // Get the console dimensions.
    let terminal_size = if let Some(size) = termsize::get() {
        size
    } else {
        termsize::Size { rows: 50, cols: 80 }
    };

    // Use a pretty printer for formatting the output.
    let mut printer = PrettyPrinter::new(1, terminal_size.cols as usize);

    let descriptions = all_algorithm_descriptions();

    let operators_text = "-operators";
    let examples_text = "-examples";
    let short_description_text = "-short-description";
    let description_text = "-description";

    let mut column_sizes = ColumnSizes::new();
    column_sizes.category_width = short_description_text.len();

    // Get the sorted keys for the map.  We want them sorted so we can write out the algorithms
    // in alphabetical order(for now).
    let mut key_list: Vec<&MutationType> = vec![];
    for key in descriptions.keys() {
        key_list.push(key);
        let key_length = key.to_string().len();
        if key_length > column_sizes.arg_name_width {
            column_sizes.arg_name_width = key_length;
        }
    }

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
            column_sizes.arg_name_width - algorithm_name.len(),
        );

        write_space(&mut printer, &mut stream);
        write_lines(
            &mut printer,
            &mut stream,
            short_description_text,
            description.short_description,
            &column_sizes,
            false,
        );

        if description.long_description.len() > 0 {
            // Start the next line.
            write_newline(&mut printer, &mut stream);
            write_newline(&mut printer, &mut stream);

            write_lines(
                &mut printer,
                &mut stream,
                description_text,
                description.long_description,
                &column_sizes,
                true,
            );
        }

        if description.operators.len() > 0 {
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

        if description.examples.len() > 0 {
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

fn write_lines<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    category_text: &str,
    text: &str,
    column_sizes: &ColumnSizes,
    do_indent: bool,
) {
    printer.increase_indent_by(column_sizes.arg_name_width + 1);

    if do_indent {
        write_indent(printer, stream);
    }

    write_token(printer, stream, category_text);
    write_spaces(
        printer,
        stream,
        column_sizes.category_width - category_text.len(),
    );
    write_space(printer, stream);

    printer.increase_indent_by(column_sizes.category_width + 1);
    write_flowable_text(printer, stream, text, "");
    printer.decrease_indent_by(column_sizes.total_column_length() + 2);
}
