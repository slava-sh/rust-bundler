extern crate my_lib;

use my_lib::{a, b, c};

fn main() {
    a::a();
    self::b::b();
    self::c::d::d();
}
