extern crate my_lib;
extern crate other_lib;

use my_lib::{a, b};

fn main() {
    a::a();
    ::b::b();
    ::my_lib::c::d();
    other_lib::some_func();
}
