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
    pub mod d {
        pub fn d() {
            println!("c::d::d()");
        }
    }
}
fn main() {
    a::a();
    ::b::b();
    ::c::d::d();
}
