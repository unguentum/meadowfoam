cargo build &&
cp target/x86_64-unknown-none/debug/meadowfoam iso_content/boot/ &&
grub-mkrescue -o meadowfoam.iso iso_content &&
qemu-system-x86_64 -net none --bios $OVMF_FD -m 256M -cdrom meadowfoam.iso -s -S #-serial mon:stdio
