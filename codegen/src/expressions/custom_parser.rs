use proc_macro2::{Delimiter, Group};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use syn::{parse::{Parse, ParseStream, Result}, Expr};
use proc_macro2::{TokenStream, TokenTree};
use proc_macro2::token_stream::IntoIter;
use quote::{TokenStreamExt, ToTokens};

struct TokenIterator {
	stream: IntoIter,
	inner: Option<Box<TokenIterator>>,
}

impl TokenIterator {
	fn new(into_iter: IntoIter) -> Self {
		TokenIterator {
			stream: into_iter,
			inner: None,
		}
	}
	
	fn next(&mut self) -> Option<TokenTree> {
		if let Some(inner_iterator) = &mut self.inner {
			if let Some(inner_next) = inner_iterator.next() {
				return Some(inner_next);
			} else {
				self.inner = None;
				return Some(TokenTree::Punct(proc_macro2::Punct::new(')', proc_macro2::Spacing::Alone)));
			}
		}
		
		return self.stream
			.next()
			.map(|token| match token {
				TokenTree::Group(group) => {
					
					
					let inner_iter = Box::from(TokenIterator::new(group.stream().into_iter()));
					self.inner = Some(inner_iter);
					Some(TokenTree::Punct(proc_macro2::Punct::new('(', proc_macro2::Spacing::Alone)))
				}
				others => Some(others),
			}).flatten();
	}
}

pub struct CustomExpr(pub Expr);

impl Parse for CustomExpr {
	fn parse(input: ParseStream) -> Result<Self> {
		let tokens: IntoIter = 
			input.parse::<TokenStream>()?
				 .into_iter();
		
		let mut iterator = TokenIterator::new(tokens);
		let mut new_tokens = TokenStream::new();
		
		while let Some(token) = iterator.next() {
			match token {
				TokenTree::Punct(punct) if punct.as_char() == '$' => {
					if let Some(TokenTree::Ident(ident)) = iterator.next() {
						let group = Group::new(Delimiter::Bracket, ident.into_token_stream());
						new_tokens.append(TokenTree::Group(group));
					} else {
						return Err(syn::Error::new(punct.span(), "Expected identifier after $"));
					}
				},
				_ => new_tokens.extend(std::iter::once(token)),
			}
		}
		
		let final_tokens = 
			new_tokens.to_string()
					  .parse()?;
		
		let expr = syn::parse2(final_tokens)?;
		Ok(CustomExpr(expr))
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