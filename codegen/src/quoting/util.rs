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
		let mut iter = self.0.into_iter().peekable();

		while let Some(line) = iter.next() {
			let item = line.into();

			if item.is_empty() {
				tokens.push();
				tokens.append("///");
			} else {
				for line in item.lines() {
					tokens.push();
					tokens.append("///");
					tokens.space();
					tokens.append(line);
				}
			}
			
			if iter.peek().is_some() {
				tokens.push();
				tokens.append("///");
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
		let mut iter =
			self.0.into_iter().peekable();

		while let Some(item) = iter.next() {
			tokens.append(item);

			if iter.peek().is_some() {
				let separator = self.1;
				if let Some((current_line, next_line)) = separator.split_once('\n') {
					tokens.append(current_line);
					tokens.push();
					tokens.append(next_line);
				} else {
					tokens.append(separator);
				}
			}
		}
	}
}
