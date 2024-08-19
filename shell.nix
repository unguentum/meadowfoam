{ nixpkgs ? import <nixpkgs> {} }:

nixpkgs.mkShell {

  nativeBuildInputs = with nixpkgs; [ rustup gcc qemu OVMF libisoburn gdb grub2_efi ];

  shellHooks = with nixpkgs; ''
	export OVMF_FD=${OVMF.fd}/FV/OVMF.fd
  '';

}
