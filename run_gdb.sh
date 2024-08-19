gdb target/x86_64-unknown-none/debug/meadowfoam -ex "target remote :1234" -ex "set pagination off" -ex "b meadowfoam::kernel_entry" -ex "lay src" -ex "foc n" -ex "c"
