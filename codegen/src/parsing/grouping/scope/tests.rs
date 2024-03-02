/*
use houtamelo_utils::{own, own_vec};
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::parsing::raw::branches::choices::OptionLine;
use crate::parsing::raw::branches::if_statement::{ElseIfStruct, EndIfStruct, IfStruct};
use crate::parsing::raw::command::YarnCommand;
use crate::parsing::raw::speech::{Speaker, Speech};
use pretty_assertions::assert_eq;
use crate::expressions::yarn_ops::YarnBinaryOp;
use crate::parsing::grouping::options::OptionsFork;
use crate::parsing::grouping::if_branch::IfBranch;
use crate::parsing::grouping::scope::{FlatLine, Flow, YarnScope};
use crate::parsing::raw::node_metadata::{NodeMetadata, TrackingSetting};
use crate::parsing::raw::{Content, RawLine, RawNode};

/*
	"title: MyFirstScene",
	"tags: night, outside",
	"tracking: never",
	"---",
	"<<fade_in 1>>",
	"<<cg \"CG_ch01_Not-yet-awake\">>",
	"You wake up. Something you shouldn't have done.",
	"<<fade_out 1>>",
	"-> Continue Sleeping",
	"-> Get up",
	"    <<if $stamina > 50 >>",
	"        You managed to get up.",
	"    <<elseif   get_random(2, 4)  >>",
	"        You managed to get up, but it was hard.",
	"    <<elseif $stamina >= 10>>",
	"        You failed to get up."
	"    <<endif>>",
	""
	"Narrator: The end. Here is your score: {$score}.",
	"===",         
*/

pub fn node_1_raw() -> RawNode {
	RawNode {
		metadata: NodeMetadata {
			title: own!("MyFirstScene"),
			tags: own_vec!["night", "outside"],
			tracking: Some(TrackingSetting::Never),
			customs: vec![],
		},
		lines: vec![
			RawLine { 
				indent: 0, 
				content: Content::Command(
					YarnCommand::Other {
						line_number: 3,
						variant: own!("fade_in"), 
						args: vec![YarnExpr::Lit(YarnLit::Int(1))]
					}
				),
			},
			RawLine { indent: 0, 
				content: Content::Command(
					YarnCommand::Other {
						line_number: 4,
						variant: own!("cg"), 
						args: vec![YarnExpr::Lit(YarnLit::Str(own!("CG_ch01_Not-yet-awake")))]
					}
				)
			},
			RawLine { indent: 0, 
				content: Content::Speech(
					Speech {
						line_number: 5,
						speaker: None,
						text: (own!("You wake up. Something you shouldn't have done."), vec![]),
						metadata: None,
					}) 
			},
			RawLine { indent: 0, 
				content: Content::Command(
					YarnCommand::Other {
						line_number: 6,
						variant: own!("fade_out"),
						args: vec![YarnExpr::Lit(YarnLit::Int(1))]
					})
			},
			RawLine { indent: 0, 
				content: Content::OptionLine(
					OptionLine {
						line_number: 7,
						text: (own!("Continue Sleeping"), vec![]),
						if_condition: None,
						metadata: None
					})
			},
			RawLine { indent: 0, 
				content: Content::OptionLine(
					OptionLine {
						line_number: 8,
						text: (own!("Get up"), vec![]),
						if_condition: None,
						metadata: None,
					})
			},
			RawLine { indent: 4, 
				content: Content::If(
					IfStruct {
						line_number: 9,
						condition: YarnExpr::BinaryOp {
							yarn_op: YarnBinaryOp::Gt,
							left: Box::new(YarnExpr::VarGet(own!("stamina"))),
							right: Box::new(YarnExpr::Lit(YarnLit::Int(50))),
						}
					}) 
			},
			RawLine { indent: 8, 
				content: Content::Speech(
					Speech {
						line_number: 10,
						speaker: None,
						text: (own!("You managed to get up."), vec![]),
						metadata: None,
					})
			},
			RawLine { indent: 4,
				content: Content::ElseIf(
					ElseIfStruct {
						line_number: 11,
						condition: YarnExpr::CustomFunctionCall {
							func_name: own!("get_random"),
							args: vec![YarnExpr::Lit(YarnLit::Int(2)), YarnExpr::Lit(YarnLit::Int(4))],
						}
					})
			},
			RawLine { indent: 8, 
				content: Content::Speech(
					Speech {
						line_number: 12,
						speaker: None,
						text: (own!("You managed to get up, but it was hard."), vec![]),
						metadata: None,
					})
			},
			RawLine { indent: 4, 
				content: Content::ElseIf(
					ElseIfStruct {
						line_number: 13,
						condition: YarnExpr::BinaryOp {
							yarn_op: YarnBinaryOp::Ge,
							left: Box::new(YarnExpr::VarGet(own!("stamina"))),
							right: Box::new(YarnExpr::Lit(YarnLit::Int(10))),
						}
					})
			},
			RawLine { indent: 8, 
				content: Content::Speech(
					Speech {
						line_number: 14,
						speaker: None,
						text: (own!("You failed to get up."), vec![]),
						metadata: None,
					})
			},
			RawLine { indent: 4, 
				content: Content::EndIf(
					EndIfStruct {
						line_number: 15,
					}
				)
			},
			RawLine { indent: 0, 
				content: Content::Speech(
					Speech {
						line_number: 16,
						speaker: Some(Speaker::Literal(own!("Narrator"))),
						text: (own!("The end. Here is your score: {$score}."), vec![]),
						metadata: None,
					})
			},
		],
	}
}

#[test]
fn test() {
	let input = node_1_raw();
	
	let expect =
		YarnScope {
			indent: 0,
			flows: vec![
				Flow::Flat(vec![
					FlatLine::Command(YarnCommand::Other {
						line_number: 0,
						variant: own!("fade_in"),
						args: vec![YarnExpr::Lit(YarnLit::Int(1))]
					}),
					FlatLine::Command(YarnCommand::Other {
						line_number: 1,
						variant: own!("cg"),
						args: vec![YarnExpr::Lit(YarnLit::Str(own!("CG_ch01_Not-yet-awake")))]
					}),
					FlatLine::Speech(
						Speech {
							line_number: 2,
							speaker: None,
							text: (own!("You wake up. Something you shouldn't have done."), vec![]),
							metadata: None,
						}
					),
					FlatLine::Command(YarnCommand::Other {
						line_number: 3,
						variant: own!("fade_out"),
						args: vec![YarnExpr::Lit(YarnLit::Int(1))]
					}), 
				]),
				Flow::OptionsFork(OptionsFork {
					first_option: (OptionLine {
						line_number: 7,
						text: (own!("Continue Sleeping"), vec![]),
						if_condition: None,
						metadata: None, 
					}, None),
					other_options: vec![
						(OptionLine {
							line_number: 8,
							text: (own!("Get up"), vec![]),
							if_condition: None,
							metadata: None,
						}, Some(Box::from(
							YarnScope {
								indent: 4,
								flows: vec![
									Flow::IfBranch(
										IfBranch {
											if_: (IfStruct {
												line_number: 9,
												condition: YarnExpr::BinaryOp {
													yarn_op: YarnBinaryOp::Gt,
													left: Box::new(YarnExpr::VarGet(own!("stamina"))),
													right: Box::new(YarnExpr::Lit(YarnLit::Int(50))),
												},
											}, Some(Box::from(
												YarnScope {
													indent: 8,
													flows: vec![
														Flow::Flat(vec![
															FlatLine::Speech(
																Speech {
																	line_number: 10,
																	speaker: None,
																	text: (own!("You managed to get up."), vec![]),
																	metadata: None,
																}
															),
														]),
													],
												}
											))),
											else_ifs: vec![
												(ElseIfStruct {
													line_number: 11,
													condition: YarnExpr::CustomFunctionCall {
														func_name: own!("get_random"),
														args: vec![YarnExpr::Lit(YarnLit::Int(2)), YarnExpr::Lit(YarnLit::Int(4))],
													},
												}, Some(Box::from(
													YarnScope {
														indent: 8,
														flows: vec![
															Flow::Flat(vec![
																FlatLine::Speech(
																	Speech {
																		line_number: 10,
																		speaker: None,
																		text: (own!("You managed to get up, but it was hard."), vec![]),
																		metadata: None,
																	}
																),
															]),
														],
													}
												))),
												(ElseIfStruct {
													line_number: 13,
													condition: YarnExpr::BinaryOp {
														yarn_op: YarnBinaryOp::Ge,
														left: Box::new(YarnExpr::VarGet(own!("stamina"))),
														right: Box::new(YarnExpr::Lit(YarnLit::Int(10))),
													},
												}, Some(Box::from(
													YarnScope {
														indent: 8,
														flows: vec![
															Flow::Flat(vec![
																FlatLine::Speech(
																	Speech {
																		line_number: 12,
																		speaker: None,
																		text: (own!("You failed to get up."), vec![]),
																		metadata: None,
																	}
																),
															]),
														],
													}
												))),
											],
											else_: None,
										}
									)
								],
							}
						)))
					],
				}),
				Flow::Flat(vec![
					FlatLine::Speech(
						Speech {
							line_number: 10,
							speaker: Some(Speaker::Literal(own!("Narrator"))),
							text: (own!("The end. Here is your score: {}."), vec![YarnExpr::VarGet(own!("score"))]),
							metadata: None,
						}
					),
				])
			],
		};
	
	let mut input_iter = input.lines.into_iter().peekable();
	let result = super::read_next_scope(-1, &mut input_iter).unwrap().unwrap();
	
	assert_eq!(result.flows[0], expect.flows[0]);
	assert_eq!(result.flows[1], expect.flows[1]);
	assert_eq!(result.flows[2], expect.flows[2]);
}
*/