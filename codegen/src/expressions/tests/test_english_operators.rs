use crate::expressions::parse_yarn_expr;
use crate::expressions::tests::*;
use pretty_assertions::assert_eq;

const ENGLISH_PATTERNS: &[(&[&[&str]], &str, &str, &str)] = &[
	(&[
		&["is", "not", "greater", "than", "or", "equal", "to"],
		&["is_not_greater_than_or_equal_to"],
	], "<", "$condition", "{5 + 3}"),
	(&[
		&["is", "not", "less", "than", "or", "equal", "to"],
		&["is_not_less_than_or_equal_to"],
	], ">", "$condition", "{5 + 3}"),
	(&[
		&["is", "not", "greater", "than"],
		&["is_not_greater_than"],
	], "<=", "$condition", "{5 + 3}"),
	(&[
		&["is", "not", "less", "than"],
		&["is_not_less_than"],
	], ">=", "$condition", "{5 + 3}"),
	(&[
		&["greater", "than", "or", "equal", "to"],
		&["greater_than_or_equal_to"],
		&["is", "greater", "than", "or", "equal", "to"],
		&["is_greater_than_or_equal_to"],
		&["gte"],
	], ">=", "$condition", "{5 + 3}"),
	(&[
		&["less", "than", "or", "equal", "to"],
		&["less_than_or_equal_to"],
		&["is", "less", "than", "or", "equal", "to"],
		&["is_less_than_or_equal_to"],
		&["lte"],
	], "<=", "$condition", "{5 + 3}"),
	(&[
		&["greater", "than"],
		&["greater_than"],
		&["is", "greater", "than"],
		&["is_greater_than"],
		&["gt"],
	], ">", "$condition", "{5 + 3}"),
	(&[
		&["less", "than"],
		&["less_than"],
		&["is", "less", "than"],
		&["is_less_than"],
		&["lt"],
	], "<", "$condition", "{5 + 3}"),
	(&[
		&["not", "equal", "to"],
		&["not_equal_to"],
		&["is", "not", "equal", "to"],
		&["is_not_equal_to"],
		&["is", "not"],
		&["is_not"],
		&["neq"],
	], "!=", "$condition", "{5 + 3}"),
	(&[
		&["equal", "to"],
		&["equal_to"],
		&["is", "equal", "to"],
		&["is_equal_to"],
		&["eq"],
		&["is"],
	], "==", "$condition", "{5 + 3}"),
	(&[
		&["bit", "xor"],
		&["bit_xor"],
		&["xor"],
	], "^", "$condition", "false"),
	(&[
		&["bit", "and"],
		&["bit_and"],
	], "&", "$condition", "false"),
	(&[
		&["bit", "or"],
		&["bit_or"],
	], "|", "$condition", "false"),
	(&[
		&["or"],
	], "||", "$condition", "false"),
	(&[
		&["and"],
	], "&&", "$condition", "false"),
	(&[
		&["not"],
	], "!", "", "false")
];

#[test]
fn test_simple() {
	ENGLISH_PATTERNS.iter().for_each(|(operator, punct, comp_left, comp_right)|
		operator.iter().for_each(|sequence| {
			let pattern = sequence.join(" ");

			let left_input = format!("{comp_left} {pattern} {comp_right}");
			let left_expr =
				match parse_yarn_expr(left_input.as_str()) {
					Ok(ok) => { ok }
					Err(err) => {
						panic!(
							"Could not parse: {left_input}.\n\
											 Error: {err}")
					}
				};

			let right_input = format!("{comp_left} {punct} {comp_right}");
			let right_expr =
				match parse_yarn_expr(right_input.as_str()) {
					Ok(ok) => { ok }
					Err(err) => {
						panic!(
							"Could not parse: {right_input}.\n\
											 Error: {err}")
					}
				};

			assert_eq!(left_expr, right_expr);
		}));

	/*
	{
		let pattern = sequence.join(" ");
		parse_both_expect_eq!(
			format!("{comp_left} {pattern} {comp_right}").as_str(), 
			format!("{comp_left} {punct} {comp_right}").as_str())
	}
	
));*/
}

#[test]
fn test_complex() {
	parse_both_expect_eq!(
		"($my_var) is not true and (5 + 3) is greater than 2", 
		"($my_var) != true && (5 + 3) > 2");

	parse_both_expect_eq!(
		"($my_var) is not true or (5 + 3) is greater than ! false", 
		"($my_var) != true || (5 + 3) > !false");

	parse_both_expect_eq!(
        "($my_var) is not true and (5 + 3) is greater than 2 or (4 < 3) is not false", 
        "($my_var) != true && (5 + 3) > 2 || (4 < 3) != false");

	parse_both_expect_eq!(
        "($my_var) is not true or (5 + 3) is greater than ! false and (2 + 2) is equal to 4", 
        "($my_var) != true || (5 + 3) > !false && (2 + 2) == 4");

	parse_both_expect_eq!(
        "($my_var) is not true and (5 + 3) is greater than 2 or (4 < 3) is not false and (2 + 2) is equal to 4", 
        "($my_var) != true && (5 + 3) > 2 || (4 < 3) != false && (2 + 2) == 4");

	parse_both_expect_eq!(
        "($my_var) is not true or (5 + 3) is greater than ! false and (2 + 2) is equal to 4 or (3 * 3) is not equal to 9", 
        "($my_var) != true || (5 + 3) > !false && (2 + 2) == 4 || (3 * 3) != 9");

	parse_both_expect_eq!(
        "($my_var) is not true and (5 + 3) is greater than 2 or (4 < 3) is not false and (2 + 2) is equal to 4 or (3 * 3) is not equal to 9", 
        "($my_var) != true && (5 + 3) > 2 || (4 < 3) != false && (2 + 2) == 4 || (3 * 3) != 9");
}