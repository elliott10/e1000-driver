use core::arch::asm;

pub(crate) fn fence() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence iorw, iorw");
    }
}
pub(crate) fn fence_w() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ow, ow");
    }
}
pub(crate) fn fence_r() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ir, ir");
    }
}
