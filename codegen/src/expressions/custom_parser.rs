use proc_macro2::{Ident, Punct, Spacing, Span};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use syn::Expr;
use proc_macro2::{TokenStream, TokenTree};
use quote::{TokenStreamExt, ToTokens};
use syn::parse::Parse;

fn tokens_iter_recursive(mut tokens: impl Iterator<Item = TokenTree>) -> impl Iterator<Item = TokenTree> {
	std::iter::from_coroutine(
	move || {
		while let Some(token) = tokens.next() {
			match token {
				TokenTree::Group(group) => {
					yield TokenTree::Punct(Punct::new('(', Spacing::Alone));
					
					let mut inner_iter =
						Box::pin(group.stream().into_iter());
					while let Some(_inner_next) = inner_iter.next() {
						yield _inner_next;
					}
					
					yield TokenTree::Punct(Punct::new(')', Spacing::Alone));
				},
				_other => {
					yield _other;
				}
			}
		}
	})
}

fn replace_sequences(
	tokens: &mut Vec<TokenTree>, 
	sequence: &[&[&str]], 
	replace_with: &[TokenTree]
) {
	
	return next(tokens, &sequence, &replace_with, 0);
	
	fn matches_ident(tuple: (&&str, &TokenTree)) -> bool {
		let (pattern, token) = tuple;
		if let TokenTree::Ident(ident) = token
			&& &ident.to_string() == pattern {
			return true;
		} else {
			return false;
		};
	}
	
	fn next(
		tokens: &mut Vec<TokenTree>,
		sequences: &[&[&str]],
		replace_with: &[TokenTree],
		index: usize,
	) {
		if index >= tokens.len() {
			return;
		}
		
		let remove_len_option =
			sequences
				.iter()
				.find_map(|seq| {
					let Some(slice) = tokens.get(index..(index + seq.len()))
						else {
							return None;
						};
					
					seq.iter()
					   .zip(slice)
					   .all(matches_ident)
					   .then_some(seq.len())
				});
		
		let next_index =
			if let Some(remove_len) = remove_len_option {
				let _ =
					tokens.splice(index..(index + remove_len), replace_with.iter().cloned())
					      .skip(usize::MAX);
				
				index + replace_with.len()
			} else {
				index + 1
			};
		
		return next(tokens, sequences, replace_with, next_index);
	}
}

macro_rules! punct {
	($ch: literal) => {
		&[TokenTree::Punct(Punct::new($ch, Spacing::Alone))]
	};
	($ch_1: literal $ch_2: literal) => {
		&[TokenTree::Punct(Punct::new($ch_1, Spacing::Joint)),
		  TokenTree::Punct(Punct::new($ch_2, Spacing::Alone))]
	};
}

pub fn replace_english_operators(tokens: &mut Vec<TokenTree>) {
	let patterns: &[(&[&[&str]], &[TokenTree])] = &[
		(&[
			&["is", "not", "greater", "than", "or", "equal", "to"],
			&["is_not_greater_than_or_equal_to"],
		], punct!{'<'}),

		(&[
			&["is", "not", "less", "than", "or", "equal", "to"],
			&["is_not_less_than_or_equal_to"],
		], punct!{'>'}),
		
		(&[
			&["greater", "than", "or", "equal", "to"],
			&["greater_than_or_equal_to"],
			&["is", "greater", "than", "or", "equal", "to"],
			&["is_greater_than_or_equal_to"],
			&["gte"],
		], punct!{'>''='}),

		(&[
			&["less", "than", "or", "equal", "to"],
			&["less_than_or_equal_to"],
			&["is", "less", "than", "or", "equal", "to"],
			&["is_less_than_or_equal_to"],
			&["lte"],
		], punct!{'<''='}),

		(&[
			&["is", "not", "greater", "than"],
			&["is_not_greater_than"],
		], punct!{'<''='}),

		(&[
			&["is", "not", "less", "than"],
			&["is_not_less_than"],
		], punct!{'>''='}),

		(&[
			&["greater", "than"],
			&["greater_than"],
			&["is", "greater", "than"],
			&["is_greater_than"],
			&["gt"],
		], punct!{'>'}),

		(&[
			&["less", "than"],
			&["less_than"],
			&["is", "less", "than"],
			&["is_less_than"],
			&["lt"],
		], punct!{'<'}),

		(&[
			&["not", "equal", "to"],
			&["not_equal_to"],
			&["is", "not", "equal", "to"],
			&["is_not_equal_to"],
			&["is", "not"],
			&["is_not"],
			&["neq"],
		], punct!{'!''='}),

		(&[
			&["equal", "to"],
			&["equal_to"],
			&["is", "equal", "to"],
			&["is_equal_to"],
			&["eq"],
			&["is"],
		], punct!{'=''='}),

		(&[
			&["bit", "xor"],
			&["bit_xor"],
			&["xor"],
		], punct!{'^'}),

		(&[
			&["bit", "and"],
			&["bit_and"],
		], punct!{'&'}),

		(&[
			&["bit", "or"],
			&["bit_or"],
		], punct!{'|'}),

		(&[
			&["or"],
		], punct!{'|''|'}),

		(&[
			&["and"],
		], punct!{'&''&'}),

		(&[
			&["not"],
		], punct!{'!'}),
	];

	patterns
		.iter()
		.for_each(|(sequences, replace_with)| 
			replace_sequences(tokens, sequences, replace_with));
}

pub struct CustomExpr(pub Expr);

impl CustomExpr {
	pub fn parse_str(input: &str) -> Result<CustomExpr> {
		let mut tokens =
			proc_macro2::TokenStream::from_str(input)
				.map(|stream|
					stream.into_iter()
					      .collect::<Vec<_>>())
				.map_err(|err| anyhow!(
					 "Could not parse input into token stream.\n\
					  Input: `{input}`\n\
					  Error: {err}")
				)?;
		
		replace_english_operators(&mut tokens);

		let mut iterator =
			tokens_iter_recursive(tokens.into_iter());
		let mut result = TokenStream::new();

		while let Some(token) = iterator.next() {
			match token {
				TokenTree::Punct(punct) if punct.as_char() == '$' => {
					match iterator.next() {
						Some(TokenTree::Ident(ident)) => {
							result.append(TokenTree::Ident(Ident::new("get_var", Span::call_site())));
							result.append(TokenTree::Punct(Punct::new('(', Spacing::Alone)));
							result.append(TokenTree::Ident(ident));
							result.append(TokenTree::Punct(Punct::new(')', Spacing::Alone)));
						}
						invalid => {
							return Err(anyhow!(
								"Expected identifier after $\n\
								 Got: `{invalid:?}`"));
						}
					}
				},
				_ => result.append(token),
			}
		}
		
		let final_tokens = 
			TokenStream::from_str(result.to_string().as_str())
				.map_err(|err| anyhow!(
					"Could not convert modified expression input into `TokenStream`\n\
					 TokenStream(as str): `{result:?}`\n\
					 Error: {err}")
				)?;
		
		syn::parse::Parser::parse2(Expr::parse, final_tokens.clone())
			.or_else(|_| syn::parse::Parser::parse2(Expr::parse_without_eager_brace, final_tokens.clone()))
			.map(CustomExpr)
			.map_err(|err| anyhow!(
				"Could not parse token stream into `syn::Expr`.\n\
				 Initial TokenStream: `{result:?}`\n\
				 Final TokenStream: `{final_tokens:?}`\n\
				 Error: {err}"))
	}
}

impl Debug for CustomExpr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0.to_token_stream())
	}
}

impl Deref for CustomExpr {
	type Target = Expr;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}