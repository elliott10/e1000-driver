use core::arch::asm;

pub(crate) fn fence() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence iorw, iorw");
    }

    #[cfg(not(target_arch = "riscv64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
pub(crate) fn fence_w() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ow, ow");
    }

    #[cfg(not(target_arch = "riscv64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
pub(crate) fn fence_r() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ir, ir");
    }

    #[cfg(not(target_arch = "riscv64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
