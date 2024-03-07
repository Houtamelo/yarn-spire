use pretty_assertions::{assert_eq, assert_matches};
use houtamelo_utils::own;
use crate::parsing::raw::{Content, ParseRawYarn};
use crate::parsing::raw::speech::{Speech, Speaker};

macro_rules! expr {
    ($lit: literal) => {
	    crate::expressions::parse_yarn_expr($lit).unwrap()
    };
}

macro_rules! speaker {
	() => {
		None
	};
	($lit: literal) => {
		Some(Speaker::Literal(own!($lit)))
	};
	({$speaker: ident}) => {
		Some(Speaker::Variable(stringify!($speaker).to_string()))
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

macro_rules! speech {
	($text: literal $(, tags[$($tags: literal),*])? $(, args[$($args: literal),*])? $(, id:$id: literal)?) => {
	    Speech {
			line_number: 0,
			line_id: line_id!($($id)?),
			speaker: None,
			text: (own!($text), vec![$($(expr!($args)),*)?]),
			tags: vec![$($(own!($tags)),*)?],
		}
    };
	($speaker: literal: $text: literal $(, tags[$($tags: literal),*])? $(, args[$($args: literal),*])? $(, id:$id: literal)?) => {
	    Speech {
			line_number: 0,
			line_id: line_id!($($id)?),
			speaker: speaker!($speaker),
			text: (own!($text), vec![$($(expr!($args)),*)?]),
			tags: vec![$($(own!($tags)),*)?],
		}
    };
	({$speaker: ident}: $text: literal $(, tags[$($tags: literal),*])? $(, args[$($args: literal),*])? $(, id:$id: literal)?) => {
	    Speech {
			line_number: 0,
			line_id: line_id!($($id)?),
			speaker: speaker!({$speaker}),
			text: (own!($text), vec![$($(expr!($args)),*)?]),
			tags: vec![$($(own!($tags)),*)?],
		}
    };
}

macro_rules! parse_unwrap {
    ($lit: literal) => {{
	    let Content::Speech(speech) =
	        Speech::parse_raw_yarn($lit, 0).unwrap().unwrap() 
	        else {
		        panic!();
	        };
	    
	    speech
    }};
}

#[test]
fn test() {
	assert_eq!(
		parse_unwrap!("Speaker: This is the dialogue line"),
		speech!("Speaker": "This is the dialogue line")
	);

	assert_eq!(
		parse_unwrap!("{$player_name}: This is the dialogue line"),
		speech!({player_name}: "This is the dialogue line")
	);
	
	assert_eq!(
		parse_unwrap!("{$player_name}: This is the {5 + 3} dialogue line, the player name is {$player_name}"),
		speech!({player_name}: "This is the {} dialogue line, the player name is {}", args["5 + 3", "$player_name"])
	);
	
	assert_eq!(
		parse_unwrap!("Speaker: This is { 3 / 10 } the dialogue line"),
		speech!("Speaker": "This is {} the dialogue line", args["3/10"])
	);

	assert_eq!(
		parse_unwrap!("Speaker: This is the dialogue line #metadata"),
		speech!("Speaker": "This is the dialogue line", tags["metadata"])
	);

	assert_eq!(
		parse_unwrap!("This is the dialogue line"),
		speech!("This is the dialogue line")
	);

	assert_eq!(
		parse_unwrap!("Speaker: \\\\\"This is the \"dialogue\" line\""),
		speech!("Speaker": r#"\"This is the "dialogue" line""#)
	);

	assert_eq!(
		parse_unwrap!(": This is the dialogue line"),
		speech!(": This is the dialogue line")
	);

	assert_eq!(
		parse_unwrap!("This is the dialogue line with a colon: in it"),
		speech!("This is the dialogue line with a colon: in it")
	);

	assert_eq!(
		parse_unwrap!("Speaker: This is the dialogue line with a colon: in it"),
		speech!("Speaker": "This is the dialogue line with a colon: in it")
	);

	assert_eq!(
		parse_unwrap!("\tYou wake up. Something you shouldn't have done."),
		speech!("You wake up. Something you shouldn't have done.")
	);
	
	assert_eq!(
		parse_unwrap!("Ethel: hey there, {$player_name} after var"),
		speech!("Ethel": "hey there, {} after var", args["$player_name"])
	);
	
	assert_eq!(
		parse_unwrap!("Ethel: hey there, I played this game {(5 + 7) * 10} times!"),
		speech!("Ethel": "hey there, I played this game {} times!", args["(5+7)*10"])
	);

	assert_eq!(
		parse_unwrap!("Speaker: This is the dialogue line #line:this_id"),
		speech!("Speaker": "This is the dialogue line", id:"this_id")
	);

	assert_eq!(
		parse_unwrap!("{$player_name}: This is the dialogue line#line:ad_2434"),
		speech!({player_name}: "This is the dialogue line", id:"ad_2434")
	);

	assert_eq!(
		parse_unwrap!("{$player_name}: This is the {5 + 3} dialogue line, the player name is {$player_name}  #line:ad_2434"),
		speech!({player_name}: "This is the {} dialogue line, the player name is {}", args["5 + 3", "$player_name"], id:"ad_2434")
	);

	assert_eq!(
		parse_unwrap!("Speaker: This is { 3 / 10 } the dialogue line  #line:232f0"),
		speech!("Speaker": "This is {} the dialogue line", args["3/10"], id:"232f0")
	);

	assert_eq!(
		parse_unwrap!("Speaker: This is the dialogue line #metadata #line:232f0"),
		speech!("Speaker": "This is the dialogue line", tags["metadata"], id:"232f0")
	);
}

macro_rules! parse {
	($lit: literal) => {{
	    Speech::parse_raw_yarn($lit, 0)
	}};
}

#[test]
fn test_invalid() {
	assert_matches!(parse!("<< This is not a speech line"), None);
	assert_matches!(parse!("-> This is not a speech line"), None);
	assert_matches!(parse!("<- This is not a speech line"), None);
	assert_matches!(parse!("#metadata"), Some(Err(_)));
	
	assert_matches!(parse!("<<fade_in 1>>"), None);
	assert_matches!(parse!("<<cg \"CG_ch01_Not-yet-awake\">>"), None);
	assert_matches!(parse!("<<fade_out 1>>"), None);
	assert_matches!(parse!("-> Option A Do that"), None);
	assert_matches!(parse!("   \t-> Option B Do this # with tags"), None);
	assert_matches!(parse!("<<if $condition_true>>"), None);
	assert_matches!(parse!("<<elseif false>>"), None);
	assert_matches!(parse!("   \t<<else>>"), None);
	assert_matches!(parse!("<<endif>>"), None);
}
