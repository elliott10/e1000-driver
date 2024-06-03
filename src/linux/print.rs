macro_rules! info {
    ($($arg:tt)*) => (
        $crate::pr_info!($($arg)*);
    )
}
macro_rules! debug (
    ($($arg:tt)*) => (
        $crate::pr_debug!($($arg)*)
    )
);
macro_rules! warn (
    ($($arg:tt)*) => (
        $crate::pr_warn!($($arg)*)
    )
);
macro_rules! error (
    ($($arg:tt)*) => (
        $crate::pr_err!($($arg)*)
    )
);
