// Please Read [Rule about third-party code is changing](https://codeforces.com/blog/entry/8790)
extern crate my_lib;
use my_lib::read;
use my_lib::pr;
// Currently bundler https://github.com/Endle/rust-bundler/tree/codeforce doesn't support use *

fn solve() {

}

// Below is template
fn main() {
    let testcases: i32 = read!();
    for _ in 0..testcases { solve(); }
}

#[inline(always)]
#[allow(dead_code)]
fn read_ivec(n:usize) -> Vec<i32> {
    let mut vec = Vec::with_capacity(n);
    for _ in 0..n {
        let x:i32 = read!();
        vec.push(x);
    }
    return vec;
}

// Based on https://users.rust-lang.org/t/how-to-get-min-max-min-index-max-index/45324/3?u=zhenbo_endle
#[inline(always)]
#[allow(dead_code)]
fn find_max_min_pos<T: std::cmp::PartialOrd + Copy>(slice: &[T]) -> (T, T, usize, usize){
    std::assert!(slice.len() > 0);
    let mut max = &slice[0];
    let mut min = &slice[0];
    let mut max_pos: usize = 0;
    let mut min_pos: usize = 0;

    for index in 1..slice.len() {
        if slice[index] < *min { min = &slice[index]; min_pos = index;}
        if slice[index] > *max { max = &slice[index]; max_pos = index;}
    }
    (*max, *min, max_pos, min_pos)
}


// Tricks
// println!("{:?}", vec);
