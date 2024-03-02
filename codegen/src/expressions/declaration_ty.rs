use genco::prelude::FormatInto;
use genco::lang::Rust;
use genco::Tokens;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeclarationTy {
	String,
	bool,
	number,
	i8,
	i16,
	i32,
	i64,
	i128,
	isize,
	u8,
	u16,
	u32,
	u64,
	u128,
	usize,
	f32,
	f64,
}

impl DeclarationTy {
	pub fn from_syn(syn_ty: syn::Type) -> Option<Self> {
		let ty_str =
			match syn_ty {
				syn::Type::Group(group) => {
					return Self::from_syn(*group.elem);
				},
				syn::Type::Paren(paren) => {
					return Self::from_syn(*paren.elem);
				},
				syn::Type::Path(path) => {
					if let Some(ident) = path.path.get_ident() {
						ident.to_string()
					} else {
						return None;
					}
				},
				syn::Type::Verbatim(verbatim) => {
					verbatim.to_string()
				},
				_ => {
					return None;
				}
			};
		
		return Self::from_str(&ty_str);
	}
	
	pub fn from_str(str: &str) -> Option<Self> {
		return Some(match str.to_ascii_lowercase().as_str() {
			"string" => DeclarationTy::String,
			"bool" => DeclarationTy::bool,
			"number" => DeclarationTy::number,
			"i8" => DeclarationTy::i8,
			"i16" => DeclarationTy::i16,
			"i32" => DeclarationTy::i32,
			"i64" => DeclarationTy::i64,
			"i128" => DeclarationTy::i128,
			"isize" => DeclarationTy::isize,
			"u8" => DeclarationTy::u8,
			"u16" => DeclarationTy::u16,
			"u32" => DeclarationTy::u32,
			"u64" => DeclarationTy::u64,
			"u128" => DeclarationTy::u128,
			"usize" => DeclarationTy::usize,
			"f32" => DeclarationTy::f32,
			"f64" => DeclarationTy::f64,
			_ => {
				return None;
			}
		});
	}
}

impl FormatInto<Rust> for &DeclarationTy {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		match self {
			DeclarationTy::String => tokens.append("String"),
			DeclarationTy::bool => tokens.append("bool"),
			DeclarationTy::number => tokens.append("number"),
			DeclarationTy::i8 => tokens.append("i8"),
			DeclarationTy::i16 => tokens.append("i16"),
			DeclarationTy::i32 => tokens.append("i32"),
			DeclarationTy::i64 => tokens.append("i64"),
			DeclarationTy::i128 => tokens.append("i128"),
			DeclarationTy::isize => tokens.append("isize"),
			DeclarationTy::u8 => tokens.append("u8"),
			DeclarationTy::u16 => tokens.append("u16"),
			DeclarationTy::u32 => tokens.append("u32"),
			DeclarationTy::u64 => tokens.append("u64"),
			DeclarationTy::u128 => tokens.append("u128"),
			DeclarationTy::usize => tokens.append("usize"),
			DeclarationTy::f32 => tokens.append("f32"),
			DeclarationTy::f64 => tokens.append("f64"),
		}
	}
}
