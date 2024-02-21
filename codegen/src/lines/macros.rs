#![allow(unused_macros)]
#![allow(unused_imports)]

macro_rules! trim {
    ($input: ident) => {
	    #[allow(unused_imports)] { 
		    use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
		    $input = $input.trim(); 
	    }
    };
}

macro_rules! trim_start {
	($input: ident) => {{
	    $input = $input.trim_start();
	}};
}

macro_rules! trim_end {
	($input: ident) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			$input = $input.trim_end(); 
		}
	};
}

macro_rules! strip_then_trim {
	($input: ident, $pattern: expr) => {
		#[allow(unused_imports)] { 
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			use $crate::lines::macros::trim;
			
			let mut any: bool = false;
			if let Some(stripped) = $input.strip_prefix($pattern) {
				$input = stripped;
				any = true;
			}
			
			if let Some(stripped) = $input.strip_suffix($pattern) {
				$input = stripped;
				any = true;
			}
			
			trim!($input);
			any 
		}
	};
}

macro_rules! strip_start {
	($input: ident, $expression: literal) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
		    
			let mut any: bool = false;
			if let Some(stripped) = $input.strip_prefix($expression) {
				$input = stripped;
				any = true;
			}

			any
		}
	};
	($input: ident, $($pattern: literal) | +) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
		    
			let mut any: bool = false;
			$(if let Some(stripped) = $input.strip_prefix($pattern) {
				$input = stripped;
				any = true;
			}) else +
			
			any
		}
	};
}

macro_rules! strip_start_then_trim {
	($input: ident, $expression: literal) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			use $crate::lines::macros::trim_start;
		    
			let mut any: bool = false;
			if let Some(stripped) = $input.strip_prefix($expression) {
				$input = stripped;
				any = true;
			}
			
			trim_start!($input);
			any
		}
	};
	($input: ident, $($pattern: literal) | +) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			use $crate::lines::macros::trim_start;
		    
			let mut any: bool = false;
			$(if let Some(stripped) = $input.strip_prefix($pattern) {
				$input = stripped;
				any = true;
			}) else +
			
			trim_start!($input);
			any
		}
	};
}

macro_rules! strip_end {
	($input: ident, $pattern: literal) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			
			let mut any: bool = false;
			if let Some(stripped) = $input.strip_suffix($pattern) {
				$input = stripped;
				any = true;
			}
			
			any
		}
	};
	($input: ident, $($pattern: literal) | +) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			
			let mut any: bool = false;
			$(if let Some(stripped) = $input.strip_suffix($pattern) {
				$input = stripped;
				any = true;
			}) else +
			
			any
		}
	};
}

macro_rules! strip_end_then_trim {
	($input: ident, $pattern: literal) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			use $crate::lines::macros::trim_end;
			
			let mut any: bool = false;
			if let Some(stripped) = $input.strip_suffix($pattern) {
				$input = stripped;
				any = true;
			}
			
			trim_end!($input);
			any
		}
	};
	($input: ident, $($pattern: literal) | +) => {
		#[allow(unused_imports)] {
			use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, Searcher};
			use $crate::lines::macros::trim_end;
			
			let mut any: bool = false;
			$(if let Some(stripped) = $input.strip_suffix($pattern) {
				$input = stripped;
				any = true;
			}) else +
			
			trim_end!($input);
			any
		}
	};
}

macro_rules! starts_with_any {
	($input: ident, $($pattern: literal) | +) => {{
		$($input.starts_with($pattern)) || +
	}};
}

macro_rules! return_if_err {
    ($expr: expr) => {
	    match $expr {
		    Ok(ok) => ok,
		    Err(err) => return Some(Err(err)),
	    }
    };
}

pub(crate) use {
	trim, 
	trim_start, 
	trim_end, 
	strip_then_trim, 
	strip_start,
	strip_start_then_trim,
	strip_end,
	strip_end_then_trim,
	starts_with_any,
	return_if_err,
};