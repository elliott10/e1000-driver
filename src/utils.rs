use core::arch::asm;

pub fn fence() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence iorw, iorw");
    }
}
pub fn fence_w() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ow, ow");
    }
}
pub fn fence_r() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ir, ir");
    }
}
