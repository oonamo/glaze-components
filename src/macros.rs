#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug {
    ($x:expr) => {
        dbg!($x)
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($x:expr) => {
        std::convert::identity($x)
    };
}

#[cfg(debug_assertions)]
pub fn type_name_of_val<T: ?Sized>(_val: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! time {
    ($func:expr) => {{
        use std::time::Instant;
        use $crate::macros::type_name_of_val;
        let before = Instant::now();
        let val = $func();
        let elapsed = before.elapsed();
        println!("function {}", type_name_of_val(&$func));
        println!("elapsed: {elapsed:.2?}");
        val
    }};
    ($func:expr, $($params:expr),*) => {{
        use std::time::Instant;
        use $crate::macros::type_name_of_val;
        let before = Instant::now();
        let val = $func($($params,)*);
        let elapsed = before.elapsed();
        println!("function {}", type_name_of_val(&$func));
        println!("elapsed: {elapsed:.2?}");
        val
    }};
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! time {
    ($func:expr) => {{
        $func()
    }};
    ($func:expr, $($params:expr),*) => {{
        $func($($params,)*)
    }};
}
