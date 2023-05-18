use std::fmt::Display;

use rand::{seq::SliceRandom};
use enum_derived::Rand;



#[derive(Clone, Copy, Rand, Debug, PartialEq)]
enum State {
	#[weight(5)] Empty,
	#[weight(5)] BL,
	#[weight(5)] BLT,
	#[weight(2)] BLR,
	#[weight(3)] BT,
	#[weight(2)] BTR,
	#[weight(3)] BR,
	#[weight(3)] LT,
	#[weight(2)] LTR,
	#[weight(3)] LR,
	#[weight(3)] TR,
	#[weight(1)] BLTR,
}
impl State {
	fn all() -> Vec<Self> {
		use State::*;

		vec![Empty, BL, BLT, BLR, BT, BTR, BR, LT, LTR, LR, TR, BLTR]
	}
	fn count() -> usize {
		Self::all().len()
	}

	fn connects_left(&self) -> bool {
		use State::*;

		matches!(self, BL | BLT | BLR | LT | LTR | LR | BLTR)
	}
	fn connects_right(&self) -> bool {
		use State::*;

		matches!(self, BLR | BTR | BR | LTR | LR | TR | BLTR)
	}
	fn connects_top(&self) -> bool {
		use State::*;

		matches!(self, BLT | BT | BTR | LT | LTR | TR | BLTR)
	}
	fn connects_bottom(&self) -> bool {
		use State::*;

		matches!(self, BL | BLT | BLR | BT | BTR | BR | BLTR)
	}

	fn fits_left(&self, other: &State) -> bool {
		self.connects_left() == other.connects_right()
	}
	fn fits_bottom(&self, other: &State) -> bool {
		self.connects_bottom() == other.connects_top()
	}

	fn fits_right(&self, other: &State) -> bool {
		other.fits_left(self)
	}
	fn fits_top(&self, other: &State) -> bool {
		other.fits_bottom(self)
	}
	
}
impl ToString for State {
	fn to_string(&self) -> String {
		use State::*;

		match self {
			Empty => "   ",
			BL =>    "━┓ ",
			BLT =>   "━┫ ",
			BLR =>   "━┳━",
			BT =>    " ┃ ",
			BTR =>   " ┣━",
			BR =>    " ┏━",
			LT =>    "━┛ ",
			LTR =>   "━┻━",
			LR =>    "━━━",
			TR =>    " ┗━",
			BLTR =>  "━╋━",
		}.to_string()
	}
}



#[derive(Clone, Debug)]
struct InvalidDomainError;
impl Display for InvalidDomainError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Domain has an invalid state")
	}
}

#[derive(Debug)]
enum Domain {
	Collapsed(State),
	Superposition(Vec<State>),
	Invalid,
}
impl Domain {
	fn enthropy(&self) -> Result<usize, InvalidDomainError> {
		use Domain::*;

		match self {
			Invalid => Err(InvalidDomainError),
			Collapsed(_) => Ok(0),
			Superposition(v) => Ok(v.len()),
		}
	}
	fn collapse(&mut self) {
		use Domain::*;

		match self {
			Collapsed(_) => {},
			Superposition(v) => {
				*self = match v.choose_mut(&mut rand::thread_rng()) {
					Some(s) => Collapsed(*s),
					None => Invalid,
				};
			},
			Invalid => {},
		}

	}
}
impl Default for Domain {
	fn default() -> Self {
		Domain::Superposition(State::all())
	}
}
impl ToString for Domain {
	fn to_string(&self) -> String {
		use Domain::*;

		match self {
			Collapsed(s) => s.to_string(),
			Superposition(v) => v.len().to_string(),
			Invalid => "!".to_string(),
		}
	}
}



#[derive(Debug, Clone)]
struct OutOfBoundsError;
impl Display for OutOfBoundsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Index out of bounds")
	}
}

struct Field {
	size: usize,
	domains: Vec<Domain>,
}
impl Field {
	fn new(size: usize) -> Self {
		Self {
			size,
			domains: (0..size*size).map(|_| Domain::default()).collect(),
		}
	}
	fn get(&self, x: usize, y: usize) -> Result<&Domain, OutOfBoundsError> {
		if x < self.size && y < self.size {
			self.domains.get(y * self.size + x).ok_or(OutOfBoundsError)
		} else {
			Err(OutOfBoundsError)
		}
	}
	fn get_mut(&mut self, x: usize, y: usize) -> Result<&mut Domain, OutOfBoundsError> {
		if x < self.size && y < self.size {
			self.domains.get_mut(y * self.size + x).ok_or(OutOfBoundsError)
		} else {
			Err(OutOfBoundsError)
		}
	}
	fn collapse_random(&mut self) -> Result<bool, InvalidDomainError> {

		let mut minimum_enthropy = State::count() + 1;
		let mut minimum_enthropy_domains: Vec<&mut Domain> = vec![];

		for domain in self.domains.iter_mut() {
			let enthropy = domain.enthropy()?;
			if enthropy == 0 {
				continue;
			}

			if enthropy < minimum_enthropy {
				minimum_enthropy_domains.clear();
				minimum_enthropy = enthropy
			}
			if enthropy == minimum_enthropy {
				minimum_enthropy_domains.push(domain);
			}

		}

		match minimum_enthropy_domains.choose_mut(&mut rand::thread_rng()) {
			Some(d) => {
				d.collapse();
				Ok(true)
			},
			None => Ok(false),
		}

	}
	fn propagate(&mut self) -> Result<(), InvalidDomainError> {
		use Domain::*;

		fn is_allowed_state(state: &State, left: Option<&Domain>, right: Option<&Domain>, top: Option<&Domain>, bottom: Option<&Domain>) -> Result<bool, InvalidDomainError> {
			Ok(
				match left {
					Some(Invalid) => return Err(InvalidDomainError),
					Some(Collapsed(other)) => state.fits_left(other),
					Some(Superposition(v)) => v.iter().any(|other| state.fits_left(other)),
					None => true,
				} &&
				match right {
					Some(Invalid) => return Err(InvalidDomainError),
					Some(Collapsed(other)) => state.fits_right(other),
					Some(Superposition(v)) => v.iter().any(|other| state.fits_right(other)),
					None => true,
				} &&
				match bottom {
					Some(Invalid) => return Err(InvalidDomainError),
					Some(Collapsed(other)) => state.fits_bottom(other),
					Some(Superposition(v)) => v.iter().any(|other| state.fits_bottom(other)),
					None => true,
				} &&
				match top {
					Some(Invalid) => return Err(InvalidDomainError),
					Some(Collapsed(other)) => state.fits_top(other),
					Some(Superposition(v)) => v.iter().any(|other| state.fits_top(other)),
					None => true,
				}
			)
		}

		for y in 0..self.size {
			for x in 0..self.size {
				let current = self.get(x, y).unwrap();
				let left = match x {
					0 => None,
					_ => self.get(x - 1, y).ok(),
				};
				let right = self.get(x + 1, y).ok();
				let top = match y {
					0 => None,
					_ => self.get(x, y - 1).ok(),
				};
				let bottom = self.get(x, y + 1).ok();

				match current {
					Invalid => return Err(InvalidDomainError),
					Collapsed(state) => {
						if !is_allowed_state(state, left, right, top, bottom)? {
							*self.get_mut(x, y).unwrap() = Invalid;
						}
					},
					Superposition(v) => {
						let mut allowed_states: Vec<State> = vec![];

						for state in v {
							if is_allowed_state(state, left, right, top, bottom)? {
								allowed_states.push(*state);
							}
						}
						
						*self.get_mut(x, y).unwrap() = match allowed_states.len() {
							0 => Invalid,
							1 => Collapsed(allowed_states[0]),
							_ => Superposition(allowed_states),
						}
					}
				}
			}
		}

		Ok(())
	}
}
impl Display for Field {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut out = String::new();
		for y in 0..self.size {
			for x in 0..self.size {
				out = format!("{}{}", out, self.get(x, y).unwrap().to_string());
			}
			out += "\n";
		}

		write!(f, "{}", out)
	}
}

fn main() -> Result<(), InvalidDomainError> {

	let mut f = Field::new(15);
	
	while f.domains.iter().any(|d| matches!(d, Domain::Superposition(_))) && !f.domains.iter().any(|d| matches!(d, Domain::Invalid)) {

		if f.collapse_random().is_err() {
			println!("{}", f);
			break;
		}

		let mut old_enthropy = 0;
		let mut new_enthropy = 1;
		while old_enthropy != new_enthropy {
			if f.propagate().is_err() {
				break;
			};
			old_enthropy = new_enthropy;
			new_enthropy = f.domains
				.iter()
				.map(|d| d.enthropy())
				.collect::<Result<Vec<usize>, InvalidDomainError>>()?
				.iter()
				.sum();
		}
		
	}

	println!("{}", f);

	Ok(())

}
