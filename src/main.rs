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

    for _ in 0..threads {
        let n_counter_clone = Arc::clone(&n_counter);
        let file_clone = Arc::clone(&file);
        thread::spawn(move || {
            loop {
                let n_size = n_counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let count = 2_u128.pow(n_size as u32);
                let mut values: Vec<i32> = vec![0; n_size as usize];
                let mut result = String::new();
                let mut largest: (String, i32) = (String::new(), 0);

                for i in 0..count {
                    fill_array_inc(&mut values, i, n_size);
                    let output = iterate(&mut values);
                    result += &*output.0;

                    if largest.1 < output.1 {
                        largest = output;
                    }
                }

                let final_output: String = "Largest: ".to_owned() + &*largest.0 + "\n" + &*result + "\n";
                file_clone.lock().unwrap().write_all(final_output.as_bytes()).unwrap();
                println!("{} Has Completed", n_size);
            }
        });
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
    fn fill_array(values: &mut Vec<i32>) {
        for i in 0..values.len() {
            values[i] = rand::rng().random_bool(0.5) as i32;
        }
    }

    fn fill_array_inc(values: &mut Vec<i32>, count: u128, n_size: u128) {
        values.clear();
        let string = format!("{count:b}");
        let sign_extend = n_size - string.len() as u128;

        let sign_extended_string = if sign_extend > 0 {
            "0".repeat(sign_extend.try_into().unwrap()) + &string
        } else {
            string
        };

        for char in sign_extended_string.chars() {
            values.push(char as i32 - 48);
        }
    }

    /// Iterates through the array.
    /// Subtracts until all values are 0
    fn iterate(values: &mut Vec<i32>) -> (String, i32) {
        let mut saved: String = String::from(format!("{:?}", values) + " : ");
        let mut iter: i32 = 0;
        while !is_zero(values) {
            iter += 1;
            subtract(values);
        }
        saved += &iter.to_string();
        saved += "\n";
        (saved, iter)
    }

    /// Checks if all values are zero
    fn is_zero(values: &Vec<i32>) -> bool {
        for i in 0..values.len() {
            if values[i] != 0 {
                return false;
            }
        }
        true
    }

    /// Subtracts the next array value from the previous one.
    /// Wraps around once hitting the end
    fn subtract(values: &mut Vec<i32>) {
        let original_val: i32 = values[0];
        for i in 0..values.len() {
            if i + 1 < values.len() {
                values[i] = (values[i] - values[i + 1]).abs();
            } else {
                values[i] = (values[i] - original_val).abs();
            }
        }
    }

    /// Checks if a file with the filename exists
    fn file_exists(filename: &str) -> bool {
        fs::metadata(filename).is_ok()
    }
}
