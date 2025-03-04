#![feature(string_remove_matches)]
#![feature(integer_atomics)]

use chrono::Local;
use rand::Rng;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::AtomicU128;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use text_io::read;

fn main() {
    print!("Select Part (A/B)");
    let part: String = read!();

    if part.to_uppercase() == "A" {
        loop {
            let mut values: Vec<u32> = vec![0; 4];
            println!("4 Random Values");
            fill_array_rand(&mut values);
            println!("Iteration 1: {:?}", values);
            let _ = iterate(&mut values, true);
            print!("\n");

            values.clear();
            println!("4 Consecutive Tribonacci Values");
            fill_array_trib(&mut values);
            println!("Iteration 1: {:?}", values);
            let _ = iterate(&mut values, true);

            println!("Run Again? (Y/N)");
            let again: String = read!();

            if again.to_uppercase() != "Y" {
                break;
            }
        }

    } else {
        let threads: u8;
        print!("Number of threads: ");
        threads = read!();

        let n_counter = Arc::new(AtomicU128::new(2));

        let file = Arc::new(Mutex::new(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(get_filename())
                .unwrap(),
        ));

        for thread in 0..threads {
            let n_counter_clone = Arc::clone(&n_counter);
            let file_clone = Arc::clone(&file);
            thread::spawn(move || {
                println!("Thread: {}", thread);
                loop {
                    let n_size = n_counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let count = 2_u128.pow(n_size as u32);
                    let mut values: Vec<u32> = vec![0; n_size as usize];
                    let mut result = String::new();
                    let mut largest: (String, i32) = (String::new(), 0);

                    for i in 0..count {
                        fill_array_inc(&mut values, i, n_size);
                        let output = iterate(&mut values, false);
                        result += &*output.0;

                        if largest.1 < output.1 {
                            largest = output;
                        }
                    }

                    let final_output: String = "Length ".to_owned() + &*n_size.to_string() + &*" Largest: ".to_owned() + &*largest.0 + "\n";
                    file_clone.lock().unwrap().write_all(final_output.as_bytes()).unwrap();
                    println!("{} Has Completed", n_size);
                }
            });
        }

        loop {
        }
    }
}

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
fn fill_array_rand(values: &mut Vec<u32>) {
    for i in 0..values.len() {
        values[i] = rand::rng().random_range(0..10000);
    }
}

/// Fill with random consecutive Tribannoci values
fn fill_array_trib(values: &mut Vec<u32>) {
    let random_index = rand::rng().random_range(0..25);
    values.append(&mut generate_trib_sequence(random_index));
}

/// Gets the 4th tribanocci values after the start index (including the start index)
fn generate_trib_sequence(start_index: i32) -> Vec<u32> {
    let mut previous_vals: [u32; 3] = [0, 0, 1];
    let mut return_vals: Vec<u32> = vec![0, 0, 1];
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

/// Fill with binary value
fn fill_array_inc(values: &mut Vec<u32>, count: u128, n_size: u128) {
    values.clear();
    let string = format!("{count:b}");
    let sign_extend = n_size - string.len() as u128;

    let sign_extended_string = if sign_extend > 0 {
        "0".repeat(sign_extend.try_into().unwrap()) + &string
    } else {
        string
    };

    for char in sign_extended_string.chars() {
        values.push(char as u32 - 48);
    }
}

/// Iterates through the array.
/// Subtracts until all values are 0
fn iterate(values: &mut Vec<u32>, debug: bool) -> (String, i32) {
    let mut saved: String = String::from(format!("{:?}", values) + " : ");
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

/// Checks if all values are zero
fn is_zero(values: &Vec<u32>) -> bool {
    for i in 0..values.len() {
        if values[i] != 0 {
            return false;
        }
    }
    true
}

/// Subtracts the next array value from the previous one.
/// Wraps around once hitting the end
fn subtract(values: &mut Vec<u32>) {
    let original_val: u32 = values[0];
    for i in 0..values.len() {
        if i + 1 < values.len() {
            values[i] = (values[i] as i64 - values[i + 1] as i64).abs() as u32;
        } else {
            values[i] = (values[i] as i64 - original_val as i64).abs() as u32;
        }
    }
}

/// Checks if a file with the filename exists
fn file_exists(filename: &str) -> bool {
    fs::metadata(filename).is_ok()
}
