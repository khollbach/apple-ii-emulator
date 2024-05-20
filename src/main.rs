use std::io::{self, Read};

use apple_ii_emulator::{Cpu, MEM_LEN};

fn main() {
    let mut prog = vec![];
    io::stdin().read_to_end(&mut prog).unwrap();

    let mut ram = vec![0; MEM_LEN];
    ram[..prog.len()].copy_from_slice(&prog);
    Cpu::new(ram).run_until_halt(0x072e);
}

fn _test_1_plus_1() {
    let mut prog = vec![];
    io::stdin().read_to_end(&mut prog).unwrap();

    let mut ram = vec![0; MEM_LEN];
    ram[..prog.len()].copy_from_slice(&prog);
    let ram = Cpu::new(ram).run_until_halt(0x0000);

    assert_eq!(ram[0], 2);
}
