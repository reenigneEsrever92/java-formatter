//! Per-option formatting tests — one module per option in the
//! [`config::OPTIONS`](java_formatter_core::config::OPTIONS) registry, named
//! after the XML option it exercises. Each module's fixtures live under
//! `tests/java/<option>/`.

mod common;

#[path = "options/align_consecutive_assignments.rs"]
mod align_consecutive_assignments;
#[path = "options/align_consecutive_variable_declarations.rs"]
mod align_consecutive_variable_declarations;
#[path = "options/align_group_field_declarations.rs"]
mod align_group_field_declarations;
#[path = "options/align_multiline_annotation_parameters.rs"]
mod align_multiline_annotation_parameters;
#[path = "options/align_multiline_array_initializer_expression.rs"]
mod align_multiline_array_initializer_expression;
#[path = "options/align_multiline_assignment.rs"]
mod align_multiline_assignment;
#[path = "options/align_multiline_binary_operation.rs"]
mod align_multiline_binary_operation;
#[path = "options/align_multiline_chained_methods.rs"]
mod align_multiline_chained_methods;
#[path = "options/align_multiline_extends_list.rs"]
mod align_multiline_extends_list;
#[path = "options/align_multiline_for.rs"]
mod align_multiline_for;
#[path = "options/align_multiline_method_brackets.rs"]
mod align_multiline_method_brackets;
#[path = "options/align_multiline_parameters.rs"]
mod align_multiline_parameters;
#[path = "options/align_multiline_parameters_in_calls.rs"]
mod align_multiline_parameters_in_calls;
#[path = "options/align_multiline_parenthesized_expression.rs"]
mod align_multiline_parenthesized_expression;
#[path = "options/align_multiline_records.rs"]
mod align_multiline_records;
#[path = "options/align_multiline_resources.rs"]
mod align_multiline_resources;
#[path = "options/align_multiline_ternary_operation.rs"]
mod align_multiline_ternary_operation;
#[path = "options/align_multiline_throws_list.rs"]
mod align_multiline_throws_list;
#[path = "options/align_subsequent_simple_methods.rs"]
mod align_subsequent_simple_methods;
#[path = "options/align_throws_keyword.rs"]
mod align_throws_keyword;
#[path = "options/annotation_new_line_in_record_component.rs"]
mod annotation_new_line_in_record_component;
#[path = "options/annotation_parameter_wrap.rs"]
mod annotation_parameter_wrap;
#[path = "options/array_element_indent.rs"]
mod array_element_indent;
#[path = "options/array_initializer_lbrace_on_next_line.rs"]
mod array_initializer_lbrace_on_next_line;
#[path = "options/array_initializer_rbrace_on_next_line.rs"]
mod array_initializer_rbrace_on_next_line;
#[path = "options/array_initializer_wrap.rs"]
mod array_initializer_wrap;
#[path = "options/assert_statement_colon_on_next_line.rs"]
mod assert_statement_colon_on_next_line;
#[path = "options/assert_statement_wrap.rs"]
mod assert_statement_wrap;
#[path = "options/assignment_wrap.rs"]
mod assignment_wrap;
#[path = "options/binary_operation_sign_on_next_line.rs"]
mod binary_operation_sign_on_next_line;
#[path = "options/binary_operation_wrap.rs"]
mod binary_operation_wrap;
#[path = "options/blank_lines_after_anonymous_class_header.rs"]
mod blank_lines_after_anonymous_class_header;
#[path = "options/blank_lines_after_class_header.rs"]
mod blank_lines_after_class_header;
#[path = "options/blank_lines_after_imports.rs"]
mod blank_lines_after_imports;
#[path = "options/blank_lines_after_package.rs"]
mod blank_lines_after_package;
#[path = "options/blank_lines_around_class.rs"]
mod blank_lines_around_class;
#[path = "options/blank_lines_around_field.rs"]
mod blank_lines_around_field;
#[path = "options/blank_lines_around_field_in_interface.rs"]
mod blank_lines_around_field_in_interface;
#[path = "options/blank_lines_around_field_with_annotations.rs"]
mod blank_lines_around_field_with_annotations;
#[path = "options/blank_lines_around_initializer.rs"]
mod blank_lines_around_initializer;
#[path = "options/blank_lines_around_method.rs"]
mod blank_lines_around_method;
#[path = "options/blank_lines_around_method_in_interface.rs"]
mod blank_lines_around_method_in_interface;
#[path = "options/blank_lines_before_class_end.rs"]
mod blank_lines_before_class_end;
#[path = "options/blank_lines_before_imports.rs"]
mod blank_lines_before_imports;
#[path = "options/blank_lines_before_method_body.rs"]
mod blank_lines_before_method_body;
#[path = "options/blank_lines_before_package.rs"]
mod blank_lines_before_package;
#[path = "options/blank_lines_between_record_components.rs"]
mod blank_lines_between_record_components;
#[path = "options/block_comment_at_first_column.rs"]
mod block_comment_at_first_column;
#[path = "options/brace_style.rs"]
mod brace_style;
#[path = "options/call_parameter_indent.rs"]
mod call_parameter_indent;
#[path = "options/call_parameters_lparen_on_next_line.rs"]
mod call_parameters_lparen_on_next_line;
#[path = "options/call_parameters_rparen_on_next_line.rs"]
mod call_parameters_rparen_on_next_line;
#[path = "options/call_parameters_wrap.rs"]
mod call_parameters_wrap;
#[path = "options/case_statement_on_new_line.rs"]
mod case_statement_on_new_line;
#[path = "options/catch_on_new_line.rs"]
mod catch_on_new_line;
#[path = "options/chained_call_indent.rs"]
mod chained_call_indent;
#[path = "options/class_annotation_wrap.rs"]
mod class_annotation_wrap;
#[path = "options/class_brace_style.rs"]
mod class_brace_style;
#[path = "options/class_count_to_use_import_on_demand.rs"]
mod class_count_to_use_import_on_demand;
#[path = "options/continuation_indent_size.rs"]
mod continuation_indent_size;
#[path = "options/declaration_parameter_indent.rs"]
mod declaration_parameter_indent;
#[path = "options/delete_unused_module_imports.rs"]
mod delete_unused_module_imports;
#[path = "options/do_not_indent_top_level_class_members.rs"]
mod do_not_indent_top_level_class_members;
#[path = "options/do_not_wrap_after_single_annotation.rs"]
mod do_not_wrap_after_single_annotation;
#[path = "options/do_not_wrap_after_single_annotation_in_parameter.rs"]
mod do_not_wrap_after_single_annotation_in_parameter;
#[path = "options/dowhile_brace_force.rs"]
mod dowhile_brace_force;
#[path = "options/else_on_new_line.rs"]
mod else_on_new_line;
#[path = "options/enum_field_annotation_wrap.rs"]
mod enum_field_annotation_wrap;
#[path = "options/extends_keyword_wrap.rs"]
mod extends_keyword_wrap;
#[path = "options/extends_list_wrap.rs"]
mod extends_list_wrap;
#[path = "options/field_annotation_wrap.rs"]
mod field_annotation_wrap;
#[path = "options/finally_on_new_line.rs"]
mod finally_on_new_line;
#[path = "options/for_brace_force.rs"]
mod for_brace_force;
#[path = "options/for_statement_lparen_on_next_line.rs"]
mod for_statement_lparen_on_next_line;
#[path = "options/for_statement_rparen_on_next_line.rs"]
mod for_statement_rparen_on_next_line;
#[path = "options/for_statement_wrap.rs"]
mod for_statement_wrap;
#[path = "options/generic_type_parameter_indent.rs"]
mod generic_type_parameter_indent;
#[path = "options/if_brace_force.rs"]
mod if_brace_force;
#[path = "options/import_layout_table.rs"]
mod import_layout_table;
#[path = "options/indent_break_from_case.rs"]
mod indent_break_from_case;
#[path = "options/indent_case_from_switch.rs"]
mod indent_case_from_switch;
#[path = "options/indent_size.rs"]
mod indent_size;
#[path = "options/keep_blank_lines_before_rbrace.rs"]
mod keep_blank_lines_before_rbrace;
#[path = "options/keep_blank_lines_between_imports.rs"]
mod keep_blank_lines_between_imports;
#[path = "options/keep_blank_lines_between_package_declaration_and_header.rs"]
mod keep_blank_lines_between_package_declaration_and_header;
#[path = "options/keep_blank_lines_in_code.rs"]
mod keep_blank_lines_in_code;
#[path = "options/keep_blank_lines_in_declarations.rs"]
mod keep_blank_lines_in_declarations;
#[path = "options/keep_control_statement_in_one_line.rs"]
mod keep_control_statement_in_one_line;
#[path = "options/keep_first_column_comment.rs"]
mod keep_first_column_comment;
#[path = "options/keep_indents_on_empty_lines.rs"]
mod keep_indents_on_empty_lines;
#[path = "options/keep_line_breaks.rs"]
mod keep_line_breaks;
#[path = "options/keep_multiple_expressions_in_one_line.rs"]
mod keep_multiple_expressions_in_one_line;
#[path = "options/keep_simple_blocks_in_one_line.rs"]
mod keep_simple_blocks_in_one_line;
#[path = "options/keep_simple_classes_in_one_line.rs"]
mod keep_simple_classes_in_one_line;
#[path = "options/keep_simple_lambdas_in_one_line.rs"]
mod keep_simple_lambdas_in_one_line;
#[path = "options/keep_simple_methods_in_one_line.rs"]
mod keep_simple_methods_in_one_line;
#[path = "options/label_indent_absolute.rs"]
mod label_indent_absolute;
#[path = "options/label_indent_size.rs"]
mod label_indent_size;
#[path = "options/lambda_brace_style.rs"]
mod lambda_brace_style;
#[path = "options/layout_on_demand_import_from_same_package_first.rs"]
mod layout_on_demand_import_from_same_package_first;
#[path = "options/layout_static_imports_separately.rs"]
mod layout_static_imports_separately;
#[path = "options/line_comment_add_space_in_suppression.rs"]
mod line_comment_add_space_in_suppression;
#[path = "options/line_comment_add_space_on_reformat.rs"]
mod line_comment_add_space_on_reformat;
#[path = "options/line_comment_at_first_column.rs"]
mod line_comment_at_first_column;
#[path = "options/line_separator.rs"]
mod line_separator;
#[path = "options/method_annotation_wrap.rs"]
mod method_annotation_wrap;
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
#[path = "options/modifier_list_wrap.rs"]
mod modifier_list_wrap;
#[path = "options/new_line_after_lparen_in_annotation.rs"]
mod new_line_after_lparen_in_annotation;
#[path = "options/new_line_after_lparen_in_record_header.rs"]
mod new_line_after_lparen_in_record_header;
#[path = "options/new_line_when_body_is_presented.rs"]
mod new_line_when_body_is_presented;
#[path = "options/parameter_annotation_wrap.rs"]
mod parameter_annotation_wrap;
#[path = "options/parentheses_expression_lparen_wrap.rs"]
mod parentheses_expression_lparen_wrap;
#[path = "options/parentheses_expression_rparen_wrap.rs"]
mod parentheses_expression_rparen_wrap;
#[path = "options/place_assignment_sign_on_next_line.rs"]
mod place_assignment_sign_on_next_line;
#[path = "options/prefer_parameters_wrap.rs"]
mod prefer_parameters_wrap;
#[path = "options/preserve_module_imports.rs"]
mod preserve_module_imports;
#[path = "options/record_components_wrap.rs"]
mod record_components_wrap;
#[path = "options/resource_list_lparen_on_next_line.rs"]
mod resource_list_lparen_on_next_line;
#[path = "options/resource_list_rparen_on_next_line.rs"]
mod resource_list_rparen_on_next_line;
#[path = "options/resource_list_wrap.rs"]
mod resource_list_wrap;
#[path = "options/right_margin.rs"]
mod right_margin;
#[path = "options/rparen_on_new_line_in_annotation.rs"]
mod rparen_on_new_line_in_annotation;
#[path = "options/rparen_on_new_line_in_record_header.rs"]
mod rparen_on_new_line_in_record_header;
#[path = "options/smart_tabs.rs"]
mod smart_tabs;
#[path = "options/soft_margins.rs"]
mod soft_margins;
#[path = "options/space_after_colon.rs"]
mod space_after_colon;
#[path = "options/space_after_comma.rs"]
mod space_after_comma;
#[path = "options/space_after_comma_in_type_arguments.rs"]
mod space_after_comma_in_type_arguments;
#[path = "options/space_after_quest.rs"]
mod space_after_quest;
#[path = "options/space_after_semicolon.rs"]
mod space_after_semicolon;
#[path = "options/space_after_type_cast.rs"]
mod space_after_type_cast;
#[path = "options/space_around_additive_operators.rs"]
mod space_around_additive_operators;
#[path = "options/space_around_annotation_eq.rs"]
mod space_around_annotation_eq;
#[path = "options/space_around_assignment_operators.rs"]
mod space_around_assignment_operators;
#[path = "options/space_around_bitwise_operators.rs"]
mod space_around_bitwise_operators;
#[path = "options/space_around_equality_operators.rs"]
mod space_around_equality_operators;
#[path = "options/space_around_lambda_arrow.rs"]
mod space_around_lambda_arrow;
#[path = "options/space_around_logical_operators.rs"]
mod space_around_logical_operators;
#[path = "options/space_around_method_ref_dbl_colon.rs"]
mod space_around_method_ref_dbl_colon;
#[path = "options/space_around_multiplicative_operators.rs"]
mod space_around_multiplicative_operators;
#[path = "options/space_around_relational_operators.rs"]
mod space_around_relational_operators;
#[path = "options/space_around_shift_operators.rs"]
mod space_around_shift_operators;
#[path = "options/space_around_unary_operator.rs"]
mod space_around_unary_operator;
#[path = "options/space_before_annotation_array_initializer_lbrace.rs"]
mod space_before_annotation_array_initializer_lbrace;
#[path = "options/space_before_anotation_parameter_list.rs"]
mod space_before_anotation_parameter_list;
#[path = "options/space_before_array_initializer_lbrace.rs"]
mod space_before_array_initializer_lbrace;
#[path = "options/space_before_catch_keyword.rs"]
mod space_before_catch_keyword;
#[path = "options/space_before_catch_lbrace.rs"]
mod space_before_catch_lbrace;
#[path = "options/space_before_catch_parentheses.rs"]
mod space_before_catch_parentheses;
#[path = "options/space_before_class_lbrace.rs"]
mod space_before_class_lbrace;
#[path = "options/space_before_colon.rs"]
mod space_before_colon;
#[path = "options/space_before_colon_in_foreach.rs"]
mod space_before_colon_in_foreach;
#[path = "options/space_before_comma.rs"]
mod space_before_comma;
#[path = "options/space_before_do_lbrace.rs"]
mod space_before_do_lbrace;
#[path = "options/space_before_else_keyword.rs"]
mod space_before_else_keyword;
#[path = "options/space_before_else_lbrace.rs"]
mod space_before_else_lbrace;
#[path = "options/space_before_finally_keyword.rs"]
mod space_before_finally_keyword;
#[path = "options/space_before_finally_lbrace.rs"]
mod space_before_finally_lbrace;
#[path = "options/space_before_for_lbrace.rs"]
mod space_before_for_lbrace;
#[path = "options/space_before_for_parentheses.rs"]
mod space_before_for_parentheses;
#[path = "options/space_before_if_lbrace.rs"]
mod space_before_if_lbrace;
#[path = "options/space_before_if_parentheses.rs"]
mod space_before_if_parentheses;
#[path = "options/space_before_method_call_parentheses.rs"]
mod space_before_method_call_parentheses;
#[path = "options/space_before_method_lbrace.rs"]
mod space_before_method_lbrace;
#[path = "options/space_before_method_parentheses.rs"]
mod space_before_method_parentheses;
#[path = "options/space_before_quest.rs"]
mod space_before_quest;
#[path = "options/space_before_semicolon.rs"]
mod space_before_semicolon;
#[path = "options/space_before_switch_lbrace.rs"]
mod space_before_switch_lbrace;
#[path = "options/space_before_switch_parentheses.rs"]
mod space_before_switch_parentheses;
#[path = "options/space_before_synchronized_lbrace.rs"]
mod space_before_synchronized_lbrace;
#[path = "options/space_before_synchronized_parentheses.rs"]
mod space_before_synchronized_parentheses;
#[path = "options/space_before_try_lbrace.rs"]
mod space_before_try_lbrace;
#[path = "options/space_before_try_parentheses.rs"]
mod space_before_try_parentheses;
#[path = "options/space_before_type_parameter_list.rs"]
mod space_before_type_parameter_list;
#[path = "options/space_before_while_keyword.rs"]
mod space_before_while_keyword;
#[path = "options/space_before_while_lbrace.rs"]
mod space_before_while_lbrace;
#[path = "options/space_before_while_parentheses.rs"]
mod space_before_while_parentheses;
#[path = "options/space_within_annotation_parentheses.rs"]
mod space_within_annotation_parentheses;
#[path = "options/space_within_array_initializer_braces.rs"]
mod space_within_array_initializer_braces;
#[path = "options/space_within_braces.rs"]
mod space_within_braces;
#[path = "options/space_within_brackets.rs"]
mod space_within_brackets;
#[path = "options/space_within_cast_parentheses.rs"]
mod space_within_cast_parentheses;
#[path = "options/space_within_catch_parentheses.rs"]
mod space_within_catch_parentheses;
#[path = "options/space_within_empty_array_initializer_braces.rs"]
mod space_within_empty_array_initializer_braces;
#[path = "options/space_within_empty_method_call_parentheses.rs"]
mod space_within_empty_method_call_parentheses;
#[path = "options/space_within_empty_method_parentheses.rs"]
mod space_within_empty_method_parentheses;
#[path = "options/space_within_for_parentheses.rs"]
mod space_within_for_parentheses;
#[path = "options/space_within_if_parentheses.rs"]
mod space_within_if_parentheses;
#[path = "options/space_within_method_call_parentheses.rs"]
mod space_within_method_call_parentheses;
#[path = "options/space_within_method_parentheses.rs"]
mod space_within_method_parentheses;
#[path = "options/space_within_parentheses.rs"]
mod space_within_parentheses;
#[path = "options/space_within_record_header.rs"]
mod space_within_record_header;
#[path = "options/space_within_switch_parentheses.rs"]
mod space_within_switch_parentheses;
#[path = "options/space_within_synchronized_parentheses.rs"]
mod space_within_synchronized_parentheses;
#[path = "options/space_within_try_parentheses.rs"]
mod space_within_try_parentheses;
#[path = "options/space_within_while_parentheses.rs"]
mod space_within_while_parentheses;
#[path = "options/spaces_inside_block_braces_when_body_is_present.rs"]
mod spaces_inside_block_braces_when_body_is_present;
#[path = "options/special_else_if_treatment.rs"]
mod special_else_if_treatment;
#[path = "options/switch_expressions_wrap.rs"]
mod switch_expressions_wrap;
#[path = "options/tab_size.rs"]
mod tab_size;
#[path = "options/ternary_operation_signs_on_next_line.rs"]
mod ternary_operation_signs_on_next_line;
#[path = "options/ternary_operation_wrap.rs"]
mod ternary_operation_wrap;
#[path = "options/throws_keyword_wrap.rs"]
mod throws_keyword_wrap;
#[path = "options/throws_list_wrap.rs"]
mod throws_list_wrap;
#[path = "options/use_relative_indents.rs"]
mod use_relative_indents;
#[path = "options/use_tab_character.rs"]
mod use_tab_character;
#[path = "options/variable_annotation_wrap.rs"]
mod variable_annotation_wrap;
#[path = "options/while_brace_force.rs"]
mod while_brace_force;
#[path = "options/while_on_new_line.rs"]
mod while_on_new_line;
#[path = "options/wrap_comments.rs"]
mod wrap_comments;
#[path = "options/wrap_first_method_in_call_chain.rs"]
mod wrap_first_method_in_call_chain;
#[path = "options/wrap_long_lines.rs"]
mod wrap_long_lines;
#[path = "options/wrap_semicolon_after_call_chain.rs"]
mod wrap_semicolon_after_call_chain;
