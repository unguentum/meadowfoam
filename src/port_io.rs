const COM1 : u16 = 0x3F8;

fn outb(port : u16, value : u8){
	unsafe { asm!("out dx, al", in("dx") port, in("al") value); }
}

fn inb(port : u16) -> u8 {
	let mut ret : u8;
	unsafe { asm!("in al, dx", out("al") ret, in("dx") port); }
	ret
}

fn com0_write_byte(c : u8){
	loop {
		let status = inb(COM1 + 5);
		if status & (1 << 5) != 0 {
			break;
		}
	}
	outb(COM1, c);
}

fn com0_write(s : &[u8]){
	for c in s {
		com0_write_byte(*c);
	}
}

fn com0_init(){
	let mut register_value = inb(COM1 + 3);
	register_value |= 1 << 7;
	outb(COM1 + 3, register_value);
	outb(COM1 + 0, 12);
	outb(COM1 + 1, 0);
	outb(COM1 + 2, 0b111);
	let mut register_value = inb(COM1 + 3);
	register_value &= !(1 << 7);
	outb(COM1 + 3, register_value);	
}
