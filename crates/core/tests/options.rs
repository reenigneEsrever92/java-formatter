//! Per-option formatting tests — one module per option in the
//! [`config::OPTIONS`](java_formatter_core::config::OPTIONS) registry, named
//! after the XML option it exercises. Each module's fixtures live under
//! `tests/java/<option>/`.

mod common;

#[path = "options/align_multiline_records.rs"]
mod align_multiline_records;
#[path = "options/annotation_parameter_wrap.rs"]
mod annotation_parameter_wrap;
#[path = "options/assignment_wrap.rs"]
mod assignment_wrap;
#[path = "options/binary_operation_wrap.rs"]
mod binary_operation_wrap;
#[path = "options/brace_style.rs"]
mod brace_style;
#[path = "options/call_parameters_lparen_on_next_line.rs"]
mod call_parameters_lparen_on_next_line;
#[path = "options/call_parameters_rparen_on_next_line.rs"]
mod call_parameters_rparen_on_next_line;
#[path = "options/call_parameters_wrap.rs"]
mod call_parameters_wrap;
#[path = "options/class_brace_style.rs"]
mod class_brace_style;
#[path = "options/class_count_to_use_import_on_demand.rs"]
mod class_count_to_use_import_on_demand;
#[path = "options/continuation_indent_size.rs"]
mod continuation_indent_size;
#[path = "options/indent_size.rs"]
mod indent_size;
#[path = "options/keep_simple_blocks_in_one_line.rs"]
mod keep_simple_blocks_in_one_line;
#[path = "options/keep_simple_lambdas_in_one_line.rs"]
mod keep_simple_lambdas_in_one_line;
#[path = "options/keep_simple_methods_in_one_line.rs"]
mod keep_simple_methods_in_one_line;
#[path = "options/method_brace_style.rs"]
mod method_brace_style;
#[path = "options/method_call_chain_wrap.rs"]
mod method_call_chain_wrap;
#[path = "options/method_parameters_lparen_on_next_line.rs"]
mod method_parameters_lparen_on_next_line;
#[path = "options/method_parameters_rparen_on_next_line.rs"]
mod method_parameters_rparen_on_next_line;
#[path = "options/method_parameters_wrap.rs"]
mod method_parameters_wrap;
#[path = "options/new_line_after_lparen_in_record_header.rs"]
mod new_line_after_lparen_in_record_header;
#[path = "options/record_components_wrap.rs"]
mod record_components_wrap;
#[path = "options/right_margin.rs"]
mod right_margin;
#[path = "options/tab_size.rs"]
mod tab_size;
#[path = "options/use_tab_character.rs"]
mod use_tab_character;
