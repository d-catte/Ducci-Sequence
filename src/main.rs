#![feature(string_remove_matches)]

use bigdecimal::{BigDecimal, One};
use chrono::Local;
use num_bigint::{BigUint, ToBigInt};
use rand::Rng;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::{Add, Div, Mul, MulAssign, Sub};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, LazyLock, Mutex};
use std::{fs, i32, u32};
use text_io::read;

/// Whether to use the Golden Ratio or not
const USE_GOLDEN_RATIO: bool = true;
/// The largest index supported by u64 precise tribonacci sequence
const MAX_TRIB_INDEX: i32 = 72;
/// The largest index "supported" by the BigUint tribonacci sequence.
/// Theoretically this could be much larger, but computational limits appear after u16 precision.
const MAX_TRIB_INDEX_BIG_INT: u128 = u16::MAX as u128;
/// How many decimals should be present with BigUint arithmetic
const PRECISION_DECIMALS: u64 = 100;

/// The main code for running the program
fn main() {
    print!("Select Part:\n(A/B): ");
    let part: String = read!();

    if part.to_uppercase() == "A" {
        print!(
            "Max Iterations:\nDefaults to the maximum supported iterations for the computation instead of random values.\n(Y/N): "
        );
        let choice: String = read!();
        let max_iterations: bool = choice.to_uppercase() == "Y";
        print!(
            "Big Int:\nBig Int enables extreme levels of precision (2^2^31 - 1 values) at the cost of computational time and resource usage.\n(Y/N): "
        );
        let choice: String = read!();
        let big_int: bool = choice.to_uppercase() == "Y";

        if big_int {
            // Part A Big Int
            loop {
                let mut values: Vec<BigUint> = Vec::new();
                println!("4 Consecutive Tribonacci Values");
                if max_iterations {
                    fill_array_set_trib_big_int(&mut values, MAX_TRIB_INDEX_BIG_INT);
                } else {
                    fill_array_trib_big_int(&mut values);
                }
                println!("Iteration 1: {:?}", values);
                let output = iterate_big_int(&mut values, false);
                println!("Iterations: {}", output.1);

                print!("Run Again?\n(Y/N): ");
                let again: String = read!();
                if again.to_uppercase() != "Y" {
                    break;
                }
            }
        } else {
            // Part A Non-BigInt
            loop {
                let mut values: Vec<u64> = vec![0; 4];
                println!("4 Random Values");
                fill_array_rand(&mut values);
                println!("Iteration 1: {:?}", values);
                let _ = iterate(&mut values, true);
                println!();

                values.clear();
                println!("4 Consecutive Tribonacci Values");
                if max_iterations {
                    fill_array_set_trib(&mut values, MAX_TRIB_INDEX);
                } else {
                    fill_array_trib(&mut values);
                }
                println!("Iteration 1: {:?}", values);
                let _ = iterate(&mut values, true);

                print!("Run Again?\n(Y/N): ");
                let again: String = read!();
                if again.to_uppercase() != "Y" {
                    break;
                }
            }
        }
    } else {
        // Part B
        let n_counter = Arc::new(AtomicU64::new(2));

        let file = Arc::new(Mutex::new(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(get_filename())
                .unwrap(),
        ));

        loop {
            let n_size = n_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let n_power_of_two = n_size.is_power_of_two();
            let count = 2_u128.pow(n_size as u32);
            let mut largest: (String, i32) = (String::new(), 0);

            // Run across multiple threads
            let results: Vec<(String, i32)> = (0..count)
                .into_par_iter()
                .map(|i| {
                    let mut values: Vec<u64> = vec![0; n_size as usize];
                    fill_array_binary(&mut values, i, n_size);
                    if n_power_of_two {
                        iterate(&mut values, false)
                    } else {
                        checking_iteration(&mut values)
                    }
                })
                .collect();

            for output in results {
                if largest.1 < output.1 {
                    largest = output;
                }
            }

            let final_output: String = "Length ".to_owned()
                + &*n_size.to_string()
                + &*" Largest: ".to_owned()
                + &*largest.0
                + "\n";
            file.lock()
                .unwrap()
                .write_all(final_output.as_bytes())
                .unwrap();
            println!("{} Has Completed", n_size);
        }
    }
}

/// Returns an appropriate file name for storing data
fn get_filename() -> String {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let mut filename = format!("output_{}.txt", date);
    let mut ending: i32 = 0;
    while file_exists(&filename) {
        filename.remove_matches(format!(" ({})", ending).as_str());
        filename.remove_matches(".txt");
        ending += 1;
        filename += format!(" ({}).txt", ending).as_str();
    }
    filename
}

/// Fills the array with randomized values
fn fill_array_rand(values: &mut Vec<u64>) {
    for i in 0..values.len() {
        values[i] = rand::rng().random_range(0..10000);
    }
}

/// Fill with random consecutive Tribannoci values
fn fill_array_trib(values: &mut Vec<u64>) {
    let random_index = rand::rng().random_range(0..=72);
    fill_array_set_trib(values, random_index);
}

/// Fill with a set index of the Tribannoci sequence
fn fill_array_set_trib(values: &mut Vec<u64>, index: i32) {
    values.append(&mut generate_trib_sequence(index));
}

/// Same as fill_array_trib, but compatible with BigUint
fn fill_array_trib_big_int(values: &mut Vec<BigUint>) {
    let random_index = rand::rng().random_range(0..=u128::MAX);
    fill_array_set_trib_big_int(values, random_index);
}

/// Same as fill_array_set_trib, but compatible with BigUint
fn fill_array_set_trib_big_int(values: &mut Vec<BigUint>, index: u128) {
    values.append(&mut generate_trib_squence_big_int(index, USE_GOLDEN_RATIO));
}

/// Gets the 4th tribanocci values after the start index (including the start index)
fn generate_trib_sequence(start_index: i32) -> Vec<u64> {
    let mut previous_vals: [u64; 3] = [0, 0, 1];
    let mut return_vals: Vec<u64> = vec![0, 0, 1];
    for i in 3..=start_index + 3 {
        let next_val = previous_vals[0] + previous_vals[1] + previous_vals[2];
        previous_vals[0] = previous_vals[1];
        previous_vals[1] = previous_vals[2];
        previous_vals[2] = next_val;
        if i >= start_index {
            if return_vals.len() >= 4 {
                return_vals.remove(0);
            }
            return_vals.push(next_val);
        }
    }
    return_vals
}

/// Generates the tribonacci sequence with BigUint precision
/// Compatible with Tribonacci Golden Ratio
fn generate_trib_squence_big_int(start_index: u128, experimental: bool) -> Vec<BigUint> {
    let mut previous_vals: [BigUint; 3] = [BigUint::ZERO, BigUint::ZERO, BigUint::from(1_u8)];
    let mut return_vals: Vec<BigUint> = if experimental {
        generate_trib_golden_ratio_big_int(start_index)
    } else {
        vec![BigUint::ZERO, BigUint::ZERO, BigUint::from(1_u8)]
    };
    if !experimental {
        for i in 3..=start_index + 3 {
            let next_val = &previous_vals[0] + &previous_vals[1] + &previous_vals[2];

            let (first, rest) = previous_vals.split_at_mut(1);
            let (second, third) = rest.split_at_mut(1);

            first[0] = std::mem::replace(
                &mut second[0],
                std::mem::replace(&mut third[0], next_val.clone()),
            );

            if i >= start_index {
                if return_vals.len() >= 4 {
                    return_vals.remove(0);
                }
                return_vals.push(next_val);
            }
        }
    }
    return_vals
}

/// A+ in the Tribonacci Golden Ratio
static A_PLUS: LazyLock<BigDecimal> = LazyLock::new(|| {
    let nineteen = BigDecimal::from(19u8);
    let three = BigDecimal::from(3u8);
    let thirtythree = BigDecimal::from(33u8);
    nineteen
        .add(three.mul(thirtythree.sqrt().unwrap()))
        .cbrt()
        .with_prec(PRECISION_DECIMALS)
});

/// A- in the Tribonacci Golden Ratio
static A_MINUS: LazyLock<BigDecimal> = LazyLock::new(|| {
    let nineteen = BigDecimal::from(19u8);
    let three = BigDecimal::from(3u8);
    let thirtythree = BigDecimal::from(33u16);
    nineteen
        .sub(three.mul(thirtythree.sqrt().unwrap()))
        .cbrt()
        .with_prec(PRECISION_DECIMALS)
});

/// B in the Tribonacci Golden Ratio
static B: LazyLock<BigDecimal> = LazyLock::new(|| {
    let five_hundred_eighty_six = BigDecimal::from(586u16);
    let one_hundred_two = BigDecimal::from(102u16);
    let thirtythree = BigDecimal::from(33u8);
    five_hundred_eighty_six
        .add(one_hundred_two.mul(thirtythree.sqrt().unwrap()))
        .cbrt()
        .with_prec(PRECISION_DECIMALS)
});

/// Denominator in the Tribonacci Golden Ratio
static DENOMINATOR: LazyLock<BigDecimal> = LazyLock::new(|| {
    let two = BigDecimal::from(2u8);
    let four = BigDecimal::from(4u8);
    B.square()
        .sub(two.mul(&*B))
        .add(four)
        .with_prec(PRECISION_DECIMALS)
});

/// Numerator in the Tribonacci Golden Ratio
static NUMERATOR: LazyLock<BigDecimal> = LazyLock::new(|| {
    let one = BigDecimal::from(1u8);
    let one_third = one
        .clone()
        .div(BigDecimal::from(3u8))
        .with_prec(PRECISION_DECIMALS);
    one_third
        .mul(&(A_PLUS.clone() + &*A_MINUS + one))
        .with_prec(PRECISION_DECIMALS)
});

/// Coefficient in the Tribonacci Golden Ratio
static COEFF: LazyLock<BigDecimal> = LazyLock::new(|| {
    let three = BigDecimal::from(3u8);
    three.mul(&*B).with_prec(PRECISION_DECIMALS)
});

/// Generates the tribonacci sequence using the Golden Ratio
fn generate_trib_golden_ratio_big_int(start_index: u128) -> Vec<BigUint> {
    let mut return_vals: Vec<BigUint> = Vec::new();
    for i in start_index..=(start_index + 3) {
        return_vals.push(big_decimal_to_big_int(
            &(COEFF
                .clone()
                .mul(power(&NUMERATOR, i as usize).div(&*DENOMINATOR))
                .with_prec(PRECISION_DECIMALS)),
        ));
    }
    return_vals
}

/// The power function for a BigDecimal
/// This function is multithreaded to help recoup some performance
fn power(input: &BigDecimal, power: usize) -> BigDecimal {
    let original = Arc::new(input.clone());
    let num_threads = 10; // Number of threads to use
    let chunk_size = power / num_threads;
    let remainder = power % num_threads;

    let results: Vec<BigDecimal> = (0..num_threads)
        .into_par_iter()
        .map(|i| {
            let original = Arc::clone(&original);
            let chunk = if i == num_threads - 1 {
                chunk_size + remainder
            } else {
                chunk_size
            };

            let mut result = BigDecimal::one();
            for _ in 0..chunk {
                result = result.with_prec(PRECISION_DECIMALS);
                result.mul_assign(&*original);
            }
            result
        })
        .collect();

    let mut final_result = BigDecimal::one();
    for result in results {
        final_result = final_result.with_prec(PRECISION_DECIMALS);
        final_result.mul_assign(&result);
    }

    final_result
}

/// Converts a BigDecimal to a BigUint
fn big_decimal_to_big_int(decimal: &BigDecimal) -> BigUint {
    let val = decimal.round(0);
    if val.is_integer() {
        BigUint::try_from(val.to_bigint().unwrap()).unwrap()
    } else {
        BigUint::ZERO
    }
}

/// Fill with binary value
fn fill_array_binary(values: &mut Vec<u64>, count: u128, n_size: u64) {
    values.clear();
    let string = format!("{count:b}");
    let sign_extend = n_size - string.len() as u64;

    let sign_extended_string = if sign_extend > 0 {
        "0".repeat(sign_extend.try_into().unwrap()) + &string
    } else {
        string
    };

    for char in sign_extended_string.chars() {
        values.push(char as u64 - 48);
    }
}

/// Iterates through the array.
/// Subtracts until all values are 0
fn iterate(values: &mut Vec<u64>, debug: bool) -> (String, i32) {
    let mut saved: String = format!("{:?}", values) + " : ";
    let mut iter: i32 = 1;
    while !is_zero(values) {
        iter += 1;
        subtract(values);
        if debug {
            println!("Iteration {}: {:?}", iter, values);
        }
    }
    saved += &iter.to_string();
    saved += "\n";
    (saved, iter)
}

/// Iterates through the array.
/// Subtracts until all values are 0 or a repeat occurs
fn checking_iteration(values: &mut Vec<u64>) -> (String, i32) {
    let mut previous_vals: HashSet<String> = HashSet::new();
    let mut saved: String = format!("{:?} : ", values);
    let mut iter: i32 = 1;

    while !is_zero(values) {
        // Found repeated value
        let storage_val = get_storage_str(values);
        if previous_vals.contains(&storage_val) {
            saved = format!("<Repeated> {}{}\n", saved, iter);
            return (saved, iter);
        } else {
            previous_vals.insert(storage_val);
            subtract(values);
        }
        iter += 1;
    }
    saved += &iter.to_string();
    saved += "\n";
    (saved, iter)
}

/// Gets the identifier string for a vector stored in the Hashset
fn get_storage_str(values: &mut Vec<u64>) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join("")
}

/// Same as iterate, but compatible with BigUint
fn iterate_big_int(values: &mut Vec<BigUint>, debug: bool) -> (String, i128) {
    let mut saved: String = format!("{:?}", values) + " : ";
    let mut iter: i128 = 1;
    while !is_zero_big_int(values) {
        iter += 1;
        subtract_big_int(values);
        if debug {
            println!("Iteration {}: {:?}", iter, values);
        }
    }
    saved += &iter.to_string();
    saved += "\n";
    (saved, iter)
}

/// Checks if all values are zero
fn is_zero(values: &Vec<u64>) -> bool {
    for i in 0..values.len() {
        if values[i] != 0 {
            return false;
        }
    }
    true
}

/// Same as is_zero, but compatible with BigUint
fn is_zero_big_int(values: &Vec<BigUint>) -> bool {
    for i in 0..values.len() {
        if values[i] != BigUint::ZERO {
            return false;
        }
    }
    true
}

/// Subtracts the next array value from the previous one.
/// Wraps around once hitting the end
fn subtract(values: &mut Vec<u64>) {
    let original_val: u64 = values[0];
    for i in 0..values.len() {
        if i + 1 < values.len() {
            values[i] = (values[i] as i128 - values[i + 1] as i128).unsigned_abs() as u64;
        } else {
            values[i] = (values[i] as i128 - original_val as i128).unsigned_abs() as u64;
        }
    }
}

/// Same as subtract, but compatible with BigUint
fn subtract_big_int(values: &mut Vec<BigUint>) {
    let original_val: BigUint = values[0].clone();
    for i in 0..values.len() {
        if i + 1 < values.len() {
            values[i] = values[i]
                .clone()
                .max(values[i + 1].clone())
                .clone()
                .sub(values[i + 1].clone().min(values[i].clone()).clone());
        } else {
            values[i] = values[i]
                .clone()
                .max(original_val.clone())
                .clone()
                .sub(original_val.clone().min(values[i].clone()).clone());
        }
    }
}

/// Checks if a file with the filename exists
fn file_exists(filename: &str) -> bool {
    fs::metadata(filename).is_ok()
}
