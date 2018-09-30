use std::io::prelude::*;
use std::fs::File;

pub fn read_file(path: &str) -> String {
    let mut file = File::open(path).unwrap();
    let mut s = String::new();
    let _ = file.read_to_string(&mut s);

    s
}

pub fn save_file(path: &str, buffer: &Vec<String>) {
    let mut file = File::create(path).unwrap();
    let mut s = String::new();

    for l in buffer.iter() {
        s.push_str(l);
        s.push_str("\n");
    }

    let result = file.write(&s.into_bytes());
    match result {
        Ok(_) => {},
        Err(e) => println!["{}", e],
    }
}

pub fn number_of_digits(n: usize) -> usize {
    let mut i = 0;
    let mut n = n;
    while n != 0 {
        n /= 10;
        i += 1;
    }
    i
}
