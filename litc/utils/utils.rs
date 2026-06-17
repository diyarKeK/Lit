#[allow(non_upper_case_globals, dead_code)]
static mut error_quantity: u32 = 0;

#[macro_export]
macro_rules! generate_error {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);

        println!(
            "\x1B[1;31m[Error]:\x1B[0m {}",
            msg
        );
        std::process::exit(1);
    }};
}