#![feature(string_remove_matches)]

use std::fs;
use std::fs::File;
use std::io::Write;
use rand::Rng;
use text_io::read;
use chrono::Local;

fn main() {
    println!("Write to file? (Y/N)");
    let character: String = read!();
    let write_to_file: bool = if character.to_uppercase() == "Y" { true } else { false };
    const MAX_VALUE: i32 = 100000;

    loop {
        println!("Enter the size of the Ducci Sequence: ");
        let amount: usize = read!();
        println!("Size: {}", amount);
        let mut values: Vec<i32> = vec![0; amount];
        fill_array(&mut values);
        println!("Values: {:?}", &values);
        let iteration_result: (i32, String) = iterate(& mut values, write_to_file);
        let iterations: i32 = iteration_result.0;
        println!("Ducci Sequence Completed In {:?} Iterations.", iterations);
        if write_to_file {
            println!("Saving data to file");
            let data: String = iteration_result.1;
            let date = Local::now().format("%Y-%m-%d").to_string();
            let mut filename = format!("output_{}.txt", date);
            let mut ending: i32 = 0;
            while file_exists(&filename) {
                filename.remove_matches(format!(" ({})", ending).as_str());
                filename.remove_matches(".txt");
                ending += 1;
                filename += format!(" ({}).txt", ending).as_str();
            }
            let mut file = File::create(filename).unwrap();
            file.write_all(data.as_bytes()).unwrap();
        }
    }

    /// Fills the array with randomized values
    fn fill_array(values: &mut Vec<i32>) {
        for i in 0..values.len() {
            values[i] = rand::rng().random_range(0..MAX_VALUE);
        }
    }

    /// Iterates through the array.
    /// Subtracts until all values are 0
    fn iterate(values: &mut Vec<i32>, save: bool) -> (i32, String) {
        let mut saved: String = String::from(format!("{:?}", values) + "\n");
        let mut iter: i32 = 0;
        while !is_zero(values) {
            iter += 1;
            subtract(values);
            if save {
                saved += &*(format!("{:?}", values) + "\n");
            }
        }
        (iter, saved)
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
