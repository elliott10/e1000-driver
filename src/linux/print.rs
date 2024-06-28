macro_rules! info {
    ($($arg:tt)*) => (
        kernel::prelude::pr_info!($($arg)*);
    )
}
macro_rules! debug (
    ($($arg:tt)*) => (
       kernel::prelude::pr_debug!($($arg)*)
    )
);
macro_rules! warn (
    ($($arg:tt)*) => (
        kernel::prelude::pr_warn!($($arg)*)
    )
);
macro_rules! error (
    ($($arg:tt)*) => (
        kernel::prelude::pr_err!($($arg)*)
    )
);
