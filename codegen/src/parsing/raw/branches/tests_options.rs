use pretty_assertions::{assert_eq, assert_matches};
use houtamelo_utils::own;
use crate::parsing::raw::{Content, ParseRawYarn};
use crate::parsing::raw::branches::options::OptionLine;

macro_rules! parse {
    ($lit: literal) => {
	    OptionLine::parse_raw_yarn($lit, 0)
    };
}

macro_rules! expr {
    ($lit: literal) => {
	    crate::expressions::parse_yarn_expr($lit).unwrap()
    };
}

macro_rules! parse_unwrap {
    ($lit: literal) => {{
	    let Content::OptionLine(line) =
	        OptionLine::parse_raw_yarn($lit, 0).unwrap().unwrap() 
	        else {
		        panic!();
	        };
	    
	    line
    }};
}

macro_rules! if_cd {
	() => {
		None
	};
	($if_cd: expr) => {
		Some($if_cd)
	};
}

macro_rules! line_id {
	() => {
		None
	};
	($lit: literal) => {
		Some(own!($lit))
	};
}

macro_rules! option_line {
	($text: literal $(, tags[$($tags: literal),*])? $(, args[$($args: literal),*])? $(, id:$id: literal)? $(, if $if_cd: expr)?) => {
	    OptionLine {
			line_number: 0,
			line_id: line_id!($($id)?),
			text: (own!($text), vec![$($(expr!($args)),*)?]),
			tags: vec![$($(own!($tags)),*)?],
		    if_condition: if_cd!($($if_cd)?),
		}
    };
}

#[test]
fn test_no_if() {
	assert_eq!(
		parse_unwrap!("-> hello"),
		option_line!("hello")
	);

	assert_eq!(
		parse_unwrap!("-> hello # metadata"),
		option_line!("hello", tags["metadata"])
	);

	assert_eq!(
		parse_unwrap!("-> hello # test data"),
		option_line!("hello", tags["test data"])
	);

	assert_eq!(
		parse_unwrap!("->Very big choice my dude \"not metadata or \\#\\# tags\""),
		option_line!("Very big choice my dude \"not metadata or ## tags\"")
	);
}

#[test]
fn test_with_if() {
	assert_eq!(
		parse_unwrap!("-> hello <<if $condition>>"),
		option_line!("hello", if expr!("$condition"))
	);

	assert_eq!(
		parse_unwrap!("-> hello <<if $condition>>  # metadata"),
		option_line!("hello", tags["metadata"], if expr!("$condition"))
	);

	assert_eq!(
		parse_unwrap!("-> hello <<if $condition>>  # metadata # more metadata"),
		option_line!("hello", tags["metadata", "more metadata"], if expr!("$condition"))
	);

	assert_eq!(
		parse_unwrap!("-> hello <<if {(5 - 3) / 3 + 2} > 0>>  # metadata # more metadata"),
		option_line!("hello", tags["metadata", "more metadata"], if expr!("{(5 - 3) / 3 + 2} > 0"))
	);
}

#[test]
fn test_invalid() {
	assert_matches!(
		parse!("-> <<if $condition>>"), 
		Some(Err(_))
	);

	assert_matches!(
		parse!("-> <<if $condition>> # metadata"),
		Some(Err(_))
	);

	assert_matches!(
		parse!("-> Hey there <<if $condition # metadata"), 
		Some(Err(_))
	);

	assert_matches!(
		parse!("-> Hey there <<if $condition>> metadata"),
		Some(Err(_))
	);

	assert_matches!(
		parse!("-> Hey there <<if >> #metadata"),
		Some(Err(_))
	);

	assert_matches!(
		parse!("-> Hey there <<$condition>> metadata"),
		Some(Err(_))
	);

	assert_matches!(
		parse!("-> Hey there <<not_if $condition>> #metadata"),
		Some(Err(_))
	);
}



