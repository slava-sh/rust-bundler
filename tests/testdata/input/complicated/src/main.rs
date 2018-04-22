extern crate my_lib;

use my_lib::{a, b};

fn main() {
    a::a();
    ::b::b();
    ::my_lib::c::d::d();
}
