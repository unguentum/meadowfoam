use core::arch::asm;

// this uses the triplefault to cause a cpu reset
// on some devices this does not trigger a reboot
pub unsafe fn triple_fault() -> ! {
    let null_idt: [u64; 2] = [0; 2];
    asm!("lidt [{}]", in(reg) &null_idt);
    asm!("sti");
    asm!("int3");
    loop {}
}
