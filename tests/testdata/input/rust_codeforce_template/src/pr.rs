// pr means print

#[inline(always)]
#[allow(dead_code)]
pub fn ln<T: std::fmt::Display>(x : T) {
    print!("{}\n", x)
}


#[inline(always)] #[allow(dead_code)]
pub fn pelem<T: std::fmt::Display>(x : &T) { print!("{}", x) }
#[inline(always)] #[allow(dead_code)]
pub fn pb() { print!(" ") }
#[inline(always)] #[allow(dead_code)]
pub fn pnl() { print!("\n") }