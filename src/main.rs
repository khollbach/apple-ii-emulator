use std::io::{self, Read};

use apple_ii_emulator::{Cpu, MEM_LEN};

fn main() {
    let mut prog = vec![];
    io::stdin().read_to_end(&mut prog).unwrap();

    let mut ram = vec![0; MEM_LEN];
    ram[..prog.len()].copy_from_slice(&prog);
    let ram = Cpu::new(ram).run_until_halt();

    assert_eq!(ram[0], 2);
}
