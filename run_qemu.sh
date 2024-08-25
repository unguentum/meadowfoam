cargo build &&
mkdir -p img_content/EFI/BOOT &&
cp target/x86_64-unknown-uefi/debug/meadowfoam.efi img_content/EFI/BOOT/BOOTX64.efi &&
qemu-system-x86_64 -net none --bios $OVMF_FD -m 256M -hdb fat:rw:img_content/ -s #-S
