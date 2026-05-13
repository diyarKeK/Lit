#[macro_export]
macro_rules! generate_error {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);

        println!(
            "\x1B[1;31m[Build failed]\x1B[0m {}",
            msg
        );
        process::exit(1);
    }};
}