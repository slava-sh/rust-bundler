pub mod a {
    pub fn a() {
        println!("a::a()");
    }
}
pub mod b {
    use a;
    pub fn b() {
        a::a();
    }
}
pub mod c {
    pub use d::d;
    mod d {
        pub fn d() {
            println!("c::d::d()");
        }
    }
}
extern crate other_lib;
fn main() {
    a::a();
    ::b::b();
    ::c::d();
    other_lib::some_func();
}
