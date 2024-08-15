// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::u32;

const MAX_TERM_COUNT:        usize = 42;
const CONVERGENCE_TOLERANCE: f64   = 1e-9;

type r64u = (u32, u32);
type r64i = (i32, i32);

fn add_next_fraction_term
(
	term:                &u32,
	convergent:          &r64u,
	previous_convergent: &r64u,
)
-> r64u
{
	return (
		term * convergent.0 + previous_convergent.0,
		term * convergent.1 + previous_convergent.1,
	);
}

fn
rational64s_to_float
(
	fraction: &r64i
)
-> f64
{
	fraction.0 as f64 / fraction.1 as f64
}

fn
rational64u_to_float
(
	fraction: &r64u
)
-> f64
{
	fraction.0 as f64 / fraction.1 as f64
}

fn 
float_to_rational64s
(
	real_number:     f64,
	max_denominator: u32
)
-> r64i
{
	let best_approximation = float_to_rational64u(real_number, max_denominator);
	return (
		if real_number < 0.0 
			{ 0-best_approximation.0 as i32 } 
		else 
			{ best_approximation.0 as i32 },
		best_approximation.1 as i32
	);
}

fn 
float_to_rational64u
(
	real_number:     f64,
	max_denominator: u32
)
-> r64u
{
	// Make sure that we are dealing with positive real numbers
	let real_number = real_number.abs();

	// Can't work with non-positive max denominator values (previously we made
	// sure that we only work with positive real numbers/fractions)
	if max_denominator <= 0
	{
		return (0, 0);
	}

	// Check if we are given a NaN value
	if real_number.is_nan()
	{
		return (0, 0);
	}
	
	// Check if real number is too large for us to handle
	if real_number > u32::MAX as f64 - 0.5
	{
		return (u32::MAX, 1);
	}

	let mut reciprocal_residual     = real_number;
	let mut continued_fraction_term = real_number.floor();

	let mut previous_convergent = (1u32,                           0u32);
	let mut convergent          = (continued_fraction_term as u32, 1u32);


	let mut n = 0;
	for term_count in 2..MAX_TERM_COUNT
	{
		// Basically the value after the decimal point
		let next_residual = reciprocal_residual - continued_fraction_term;

		// If the difference is smaller than our tolerance we can return the 
		// current representation
		if next_residual.abs() <= CONVERGENCE_TOLERANCE
		{
			return (
				convergent.0,
				convergent.1
			);
		}

		reciprocal_residual     = 1.0 / next_residual;
		continued_fraction_term = reciprocal_residual.floor();

		
		n = (max_denominator - previous_convergent.1) / convergent.1;
		if convergent.0 > 0
		{
			n = std::cmp::min(
				(u32::MAX - previous_convergent.0) / convergent.0, 
				n
			);	
		}

		if continued_fraction_term >= n as f64 { break; }

		let next_convergent = add_next_fraction_term(&(continued_fraction_term as u32), &convergent, &previous_convergent);
		previous_convergent = convergent;
		convergent          = next_convergent;
	}

	let mut best_approximation = convergent;

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
		(
			(n as f64 > lower_bound)
			|| 
			(
				(real_number - rational64u_to_float(&semiconvergent)).abs()
				< (real_number - rational64u_to_float(&convergent)).abs()
			)
		)
		{
			best_approximation = semiconvergent;
		}
	}

	return best_approximation;
}