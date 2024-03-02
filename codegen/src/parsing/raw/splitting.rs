use crate::{LineNumber, UnparsedLine};
use anyhow::{Result, anyhow};
use State::{Outside, Inside};

pub struct UnparsedNode<'a> {
	pub outer_lines: Vec<&'a UnparsedLine>, // metadata
	pub inner_lines: Vec<&'a UnparsedLine>,
}

enum NodeDelimiter<'a> {
	NodeStart(LineNumber),
	NodeEnd(LineNumber),
	Not(&'a UnparsedLine),
}

fn as_delimiter(line: &UnparsedLine) -> NodeDelimiter {
	let trimmed_line = line.text.trim();
	
	return match trimmed_line {
		"---" => NodeDelimiter::NodeStart(line.line_number),
		"===" => NodeDelimiter::NodeEnd(line.line_number),
		_ => NodeDelimiter::Not(line),
	};
}

enum State<'a> {
	Outside {
		lines: Vec<&'a UnparsedLine>,
	},
	Inside {
		outer_lines: Vec<&'a UnparsedLine>,
		inner_lines: Vec<&'a UnparsedLine>,
	},
}

pub fn split_into_unparsed_nodes(lines: &[UnparsedLine])
                                 -> Result<Vec<UnparsedNode>> {
	let mut as_delimiters =
		lines.iter()
		     .map(|line| as_delimiter(line));
	
	let mut nodes = Vec::new();
	let mut state = Outside { lines: vec![] };
	
	while let Some(maybe_delimiter) = as_delimiters.next() {
		match &mut state {
			Outside { lines } =>
				match maybe_delimiter {
					NodeDelimiter::NodeStart(_) => {
						state = Inside { 
							outer_lines: std::mem::take(lines), 
							inner_lines: vec![],
						};
					},
					NodeDelimiter::NodeEnd(line_number) => {
						return Err(anyhow!(
							"Node end delimiter found before start delimiter.\n\
							 At line nº{line_number}\n\n\
							 Help: Maybe you meant to start a node? \n\
							 Help: A node is started by writing a line with three hyphens (`---`), \
							 and ended by writing a line with three equals signs (`===`)."
						));
					},
					NodeDelimiter::Not(unparsed_line) => {
						lines.push(unparsed_line);
					},
				}
			Inside { outer_lines, inner_lines } =>
				match maybe_delimiter {
					NodeDelimiter::NodeStart(line_number) => {
						return Err(anyhow!(
							"Node start delimiter found before end delimiter.\n\
							 At line nº{line_number}\n\n\
							 Help: Maybe you meant to end a node? \n\
							 Help: A node is started by writing a line with three hyphens (`---`), \
							 and ended by writing a line with three equals signs (`===`)."
						));
					},
					NodeDelimiter::NodeEnd(_) => {
						nodes.push(UnparsedNode {
							outer_lines: std::mem::take(outer_lines),
							inner_lines: std::mem::take(inner_lines),
						});
						
						state = Outside {
							lines: vec![],
						};
					},
					NodeDelimiter::Not(unparsed_line) => {
						inner_lines.push(unparsed_line);
					},
				}
		}
	} 
	
	return match state {
		Outside { lines } =>
			if lines.is_empty() {
				Ok(nodes)
			} else {
				Err(anyhow!(
					"File ended with orphan lines. (Orphan lines are not allowed)\n\
					 Lines: \n\t{}\n\
					 Help: If you want to ignore a line, you can comment it out by starting it with `//`.\n\
					 Help: Lines above a node are considered metadata.\n\
					 Lines bellow a node are only allowed if there's another node bellow them.\n\
					 ", lines.into_iter().map(|u| format!("nº{}: `{}`", u.line_number, u.text)).collect::<Vec<_>>().join("\n\t")
				))
			},
		Inside { .. } =>
			Err(anyhow!(
				"File ended without ending node.\n\
				 Help: The last line of a file needs to be made of three `equals signs` (`===`) \n\
				 Help: A node is started by writing a line with three `hyphens` (`---`), \
				 and ended by writing a line with three `equals signs` (`===`)."
			)),
	};
}

/*
#[test]
fn test() {
	macro_rules! unpln {
	    ($number: literal, $text: literal) => {
		    UnparsedLine { line_number: $number, text: $text.to_string()}
	    };
	}
	
	assert_eq!(as_delimiter(&unpln!(0, "---")), NodeDelimiter::NodeStart(0));
	assert_eq!(as_delimiter(&unpln!(1, "===")), NodeDelimiter::NodeEnd(1));
	
	assert_eq!(as_delimiter(&unpln!(2, "Celina: hey there!")), NodeDelimiter::Not(&unpln!(2, "Celina: hey there!")));
	assert_eq!(as_delimiter(&unpln!(3, "Celina: hey there!")), NodeDelimiter::Not(&unpln!(3, "Celina: hey there!")));
	
	assert_eq!(as_delimiter(&unpln!(4, "  ---\t")), NodeDelimiter::NodeStart(4));
	assert_eq!(as_delimiter(&unpln!(5, "\t  ===")), NodeDelimiter::NodeEnd(5));
	assert_eq!(as_delimiter(&unpln!(6, "  ===     \t")), NodeDelimiter::NodeEnd(6));
	assert_eq!(as_delimiter(&unpln!(7, "\t  ---")), NodeDelimiter::NodeStart(7));
	
	assert_eq!(as_delimiter(&unpln!(8, "  ---\tCelina: hey there!")), NodeDelimiter::Not(&unpln!(8, "  ---\tCelina: hey there!")));
	assert_eq!(as_delimiter(&unpln!(9, "  ===\tCelina: hey there!")), NodeDelimiter::Not(&unpln!(9, "  ===\tCelina: hey there!")));

	assert_eq!(as_delimiter(&unpln!(1, "<<set $player_var 50>>")), NodeDelimiter::Not(&unpln!(1, "<<set $player_var 50>>")));
	assert_eq!(as_delimiter(&unpln!(2, "<<set $player_var 50>>")), NodeDelimiter::Not(&unpln!(2, "<<set $player_var 50>>")));

	assert_eq!(as_delimiter(&unpln!(3, "-> Option A \t #metadata")), NodeDelimiter::Not(&unpln!(3, "-> Option A \t #metadata")));
	assert_eq!(as_delimiter(&unpln!(4, "-> Option A \t #metadata")), NodeDelimiter::Not(&unpln!(4, "-> Option A \t #metadata")));
}
*/