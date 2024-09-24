use genco::lang::rust::Tokens;
use genco::quote;

pub fn all_tokens() -> Tokens {
	quote! {
		use rand::Rng;
		
		pub fn random(rng: &mut impl Rng) -> f64 {
			rng.gen_range(0.0..1.0)
		}
		
		pub fn random_range(rng: &mut impl Rng, lower: f64, upper: f64) -> f64 {
			rng.gen_range(lower..upper)
		}
		
		pub fn dice(rng: &mut impl Rng, sides: usize) -> usize {
			rng.gen_range(1..=sides)
		}
		
		pub fn round(num: f64) -> isize {
			num.round() as isize
		}
		
		pub fn round_places(num: f64, places: i32) -> f64 {
			let multiplier = 10f64.powi(places);
			(num * multiplier).round() / multiplier
		}
		
		pub fn floor(num: f64) -> isize {
			num.floor() as isize
		}
		
		pub fn ceil(num: f64) -> isize {
			num.ceil() as isize
		}
		
		fn is_integer(num: f64) -> bool {
			num == num.round()
		}
		
		pub fn inc(num: f64) -> isize {
			if is_integer(num) {
				num as isize + 1
			} else {
				num.ceil() as isize
			}
		}
		
		pub fn dec(num: f64) -> isize {
			if is_integer(num) {
				num as isize - 1
			} else {
				num.floor() as isize
			}
		}
		
		pub fn int(num: f64) -> isize {
			if num > 0.0 {
				num.floor() as isize
			} else {
				num.ceil() as isize
			}
		}
	}
}