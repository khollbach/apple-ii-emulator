use std::{
    env,
    fs::File,
    io::{self, Read},
};

use apple_ii_emulator::{Cpu, MEM_LEN};

fn main() {
    let mut prog = vec![];

    // io::stdin().read_to_end(&mut prog).unwrap();
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 2);
    let mut file = File::open(&args[1]).unwrap();
    file.read_to_end(&mut prog).unwrap();

    let mut ram = vec![0; MEM_LEN];
    // ram[0x0a..].copy_from_slice(&prog);
    // ram[0x0a..=0x3848].copy_from_slice(&prog);
    ram[0x0a..=0x3618].copy_from_slice(&prog);

    let cpu = Cpu::new(ram).run_until_halt(0x0400);
    eprintln!("{:?}", cpu);
}

fn _test_1_plus_1() {
    let mut prog = vec![];
    io::stdin().read_to_end(&mut prog).unwrap();

    let mut ram = vec![0; MEM_LEN];
    ram[..prog.len()].copy_from_slice(&prog);
    let cpu = Cpu::new(ram).run_until_halt(0x0000);

    assert_eq!(cpu.ram[0], 2);
}
