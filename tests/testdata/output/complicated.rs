pub mod a {
    pub fn a() {
        println!("a::a()");
    }
}
pub mod b {
    use crate::a;
    pub fn b() {
        a::a();
    }
}
pub mod c {
    pub mod d {
        pub fn d() {
            println!("c::d::d()");
        }
    }
}
fn main() {
    a::a();
    self::b::b();
    self::c::d::d();
}
