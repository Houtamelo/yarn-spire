use genco::lang::Rust;
use genco::tokens::{FormatInto, ItemStr, Tokens};

/// Documentation comments.
pub struct Comments<I>(pub I);

impl<I> FormatInto<Rust> for Comments<I>
	where
		I: IntoIterator,
		I::Item: Into<ItemStr>,
{
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		let mut it = self.0.into_iter().peekable();

		while let Some(line) = it.next() {
			let line = line.into();

			if line.is_empty() {
				tokens.append("///");
			} else {
				tokens.append("///");
				tokens.space();
				tokens.append(line);
			}

			tokens.line();
			tokens.append("/// \n");

			if it.peek().is_some() {
				tokens.push();
			}
		}
	}
}

pub struct SeparatedItems<T, TIter: IntoIterator<Item = T>>(pub TIter, pub &'static str);

impl<T, TIter> FormatInto<Rust> for SeparatedItems<T, TIter>
	where
		T: FormatInto<Rust>,
		TIter: IntoIterator<Item = T>,
{
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		let mut it = self.0.into_iter().peekable();

		while let Some(item) = it.next() {
			tokens.append(item);

			if it.peek().is_some() {
				tokens.append(self.1);
			}
		}
	}
}
