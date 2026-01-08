// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

// Based on: https://github.com/google/audio-to-tactile/blob/main/src/dsp/number_util.c

use std::u32;

const MAX_TERM_COUNT:        usize = 42;
const CONVERGENCE_TOLERANCE: f64   = 1e-9;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct uR64
{
    pub nominator:   u32,
	pub denominator: u32
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct iR64
{
    pub nominator:   i32,
	pub denominator: i32
}



fn greatest_common_divisor
(
	mut a: u32,
	mut b: u32
)
-> u32
{
	while b != 0
	{
		let remainder = a % b;
		a = b;
		b = remainder;
	}
	return a;
}

fn add_next_fraction_term
(
	term:                &u32,
	convergent:          &uR64,
	previous_convergent: &uR64,
)
-> uR64
{
	return uR64 {
		nominator:   term * convergent.nominator   + previous_convergent.nominator,
		denominator: term * convergent.denominator + previous_convergent.denominator
	};
}



fn
rational64s_to_f64
(
	fraction: &iR64
)
-> f64
{
	fraction.nominator as f64 / fraction.denominator as f64
}

fn
rational64u_to_f64
(
	fraction: &uR64
)
-> f64
{
	fraction.nominator as f64 / fraction.denominator as f64
}

fn
f64_to_rational64s
(
	real_number:     f64,
)
-> iR64
{
	let best_approximation = f64_to_rational64u(real_number);
	return iR64 {
		nominator: if real_number < 0.0 
			{ 0-best_approximation.nominator as i32 } 
		else 
			{ best_approximation.nominator as i32 },
		denominator: best_approximation.denominator as i32
	};
}

fn
f64_to_rational64u
(
	real_number:     f64,
)
-> uR64
{
	// Make sure that we are dealing with positive real numbers
	let real_number = real_number.abs();

	// Check if we are given a NaN value
	if real_number.is_nan()
	{
		return uR64 { nominator: 0, denominator: 0};
	}
	
	// Check if real number is too large for us to handle
	if real_number > u32::MAX as f64 - 0.5
	{
		return uR64 { nominator: i32::MAX as u32, denominator: 1};
	}

	let mut reciprocal_residual     = real_number;
	let mut continued_fraction_term = real_number.floor();

	let mut previous_convergent = uR64 { nominator: 1u32,                           denominator: 0u32 };
	let mut convergent          = uR64 { nominator: continued_fraction_term as u32, denominator: 1u32 };


	let mut n = 0;
	for _ in 2..MAX_TERM_COUNT
	{
		// Basically the value after the decimal point
		let next_residual = reciprocal_residual - continued_fraction_term;

		// If the difference is smaller than our tolerance we can return the 
		// current representation
		if next_residual.abs() <= CONVERGENCE_TOLERANCE
		{
			return convergent;
		}

		reciprocal_residual     = 1.0 / next_residual;
		continued_fraction_term = reciprocal_residual.floor();

		
		n = (i32::MAX as u32 - previous_convergent.denominator) / convergent.denominator;
		if convergent.nominator > 0
		{
			n = std::cmp::min(
				(u32::MAX - previous_convergent.nominator) / convergent.nominator, 
				n
			);	
		}

		if continued_fraction_term >= n as f64 { break; }

		let next_convergent = add_next_fraction_term(&(continued_fraction_term as u32), &convergent, &previous_convergent);
		previous_convergent = convergent;
		convergent          = next_convergent;
	}

	let mut best_approximation = convergent.clone();

	// Add a final term if a semiconvergent further improves the approximation
	let lower_bound = continued_fraction_term / 2.0;

	if n as f64 >= lower_bound
	{
		if n as f64 > continued_fraction_term 
		{ 
			n = continued_fraction_term as u32; 
		}

		let semiconvergent = add_next_fraction_term(&n, &convergent, &previous_convergent);

		if 
			(n as f64 > lower_bound)
			|| 
			(
				(real_number - rational64u_to_f64(&semiconvergent)).abs()
				< (real_number - rational64u_to_f64(&convergent)).abs()
			)
		{
			best_approximation = semiconvergent;
		}
	}

	let gcd = greatest_common_divisor(
		best_approximation.nominator, 
		best_approximation.denominator
	);

	return uR64 {
		nominator:   best_approximation.nominator   / gcd,
		denominator: best_approximation.denominator / gcd
	};
}

impl From<f64> for uR64 { fn from (val: f64) -> Self { f64_to_rational64u(val) } }
impl From<f64> for iR64 { fn from (val: f64) -> Self { f64_to_rational64s(val) } }

impl From<u32> for uR64 { fn from (val: u32) -> Self { f64_to_rational64u(val as f64) } }
impl From<u32> for iR64 { fn from (val: u32) -> Self { f64_to_rational64s(val as f64) } }
impl From<i32> for iR64 { fn from (val: i32) -> Self { f64_to_rational64s(val as f64) } }


impl From<uR64> for f64 { fn from (val: uR64) -> Self { rational64u_to_f64(&val) } }
impl From<iR64> for f64 { fn from (val: iR64) -> Self { rational64s_to_f64(&val) } }

impl From<uR64> for u32 { fn from (val: uR64) -> Self { rational64u_to_f64(&val) as u32 } }
impl From<iR64> for u32 { fn from (val: iR64) -> Self { rational64s_to_f64(&val) as u32 } }
impl From<iR64> for i32 { fn from (val: iR64) -> Self { rational64s_to_f64(&val) as i32 } }