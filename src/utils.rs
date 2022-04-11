#[macro_export]
macro_rules! print_debug {
  ($cond:expr, $msg:expr $(, $farg:expr)* ) => {
    if $cond {
        println!($msg,$( $farg ),*);
    }
  };
}
