use std::collections::HashMap;

#[derive(Debug)]
pub struct SoftSwitches {
    states: HashMap<SoftSwitch, bool>,
}

/// See Apple //e Technical Reference Manual, Appendix F: Frequently Used
/// Tables, starting on page 258.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoftSwitch {
    Altchar,
    _80Col,
    _80Store,
    Page2,
    Text,
    Mixed,
    Hires,
    IouEnable,
    Dhires,

    /// Write protect language card RAM.
    WriteProtect,
    /// In language card RAM, select bank 2 for $d000..$e000.
    Bnk2,
    /// Enable language card RAM for reading, instead of ROM.
    Lcram,
    /// todo: not quite sure what this does
    Altzp,
}

impl SoftSwitches {
    pub fn new() -> Self {
        // todo: do any switches have default values other than false ?
        Self {
            states: HashMap::new(),
        }
    }

    pub fn is_set(&self, switch: SoftSwitch) -> bool {
        self.states.get(&switch).copied().unwrap_or(false)
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match self.access(addr, AccessType::Read) {
            Some(true) => 0x80,
            Some(false) | None => 0,
        }
    }

    pub fn write(&mut self, addr: u16) {
        let ret = self.access(addr, AccessType::Write);
        assert!(ret.is_none());
    }

    fn access(&mut self, addr: u16, rw: AccessType) -> Option<bool> {
        let [lo, hi] = addr.to_le_bytes();
        assert_eq!(hi, 0xc0);

        if lo & 0x80 != 0 {
            self.bank_select(lo, rw);
            return None;
        }

        let (switch, op) = soft_switch_info(lo, rw);
        match op {
            Operation::Clear => self.states.insert(switch, false),
            Operation::Set => self.states.insert(switch, true),
            Operation::Query => return Some(self.is_set(switch)),
        };
        None
    }

    fn bank_select(&mut self, lo: u8, rw: AccessType) {
        assert_eq!(lo & 0xf0, 0x80);
        assert_eq!(lo & 0b_0100, 0);
        if rw == AccessType::Write {
            // The docs say you accesses to these switches should be reads. But
            // the IIe self-test rom routines write to them, so we'll allow it.
            eprintln!("warning: write to bank select soft switch: $c0{:02x}", lo);
        }

        // NOTE: this is flipped from what you'd probably expect.
        let bank_1 = lo & 0b_1000 != 0;
        self.states.insert(SoftSwitch::Bnk2, !bank_1);

        // Again, note the flip.
        //
        // todo: technically, you're supposed to access the soft switch twice in
        // a row to enable read/write mode. We're ignoring that detail for now.
        // (Note that read-only mode takes effect right away; it's only
        // read/write mode that takes two accesses.)
        let write_enable = lo & 0b_0001 != 0;
        self.states.insert(SoftSwitch::WriteProtect, !write_enable);

        // If the two lowest bits are the same, read RAM. Otherwise, read ROM.
        let read_ram = lo & 0b_0010 == lo & 0b_0001;
        self.states.insert(SoftSwitch::Lcram, read_ram);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessType {
    Read,
    Write,
}

enum Operation {
    Clear,
    Set,
    Query,
}

fn soft_switch_info(lo: u8, rw: AccessType) -> (SoftSwitch, Operation) {
    use AccessType::*;
    use Operation::*;
    use SoftSwitch::*;

    // This information is from tables 2-10 and 4-6 in the TRM.
    match (lo, rw) {
        //
        // Table 2-10. Display Soft Switches
        //
        (0x0e, Write) => (Altchar, Clear),
        (0x0f, Write) => (Altchar, Set),
        (0x1e, Read) => (Altchar, Query),

        (0x0c, Write) => (_80Col, Clear),
        (0x0d, Write) => (_80Col, Set),
        (0x1f, Read) => (_80Col, Query),

        (0x00, Write) => (_80Store, Clear),
        (0x01, Write) => (_80Store, Set),
        (0x18, Read) => (_80Store, Query),

        (0x54, Read | Write) => (Page2, Clear),
        (0x55, Read | Write) => (Page2, Set),
        (0x1c, Read) => (Page2, Query),

        (0x50, Read | Write) => (Text, Clear),
        (0x51, Read | Write) => (Text, Set),
        (0x1a, Read) => (Text, Query),

        (0x52, Read | Write) => (Mixed, Clear),
        (0x53, Read | Write) => (Mixed, Set),
        (0x1b, Read) => (Mixed, Query),

        (0x56, Read | Write) => (Hires, Clear),
        // NOTE: there's a typo in the TRM. The appendix version of table 2-10
        // says $c059 instead of $c057. Weird.
        (0x57, Read | Write) => (Hires, Set),
        (0x1d, Read) => (Hires, Query),

        (0x7e, Write) => (IouEnable, Clear),
        (0x7f, Write) => (IouEnable, Set),
        (0x7e, Read) => (IouEnable, Query),

        (0x5e, Read | Write) => (Dhires, Clear),
        (0x5f, Read | Write) => (Dhires, Set),
        (0x7f, Read) => (Dhires, Query),

        //
        // Table 4-6. Bank Select Switches
        //
        (0x11, Read) => (Bnk2, Query),
        (0x12, Read) => (Lcram, Query),

        (0x08, Write) => (Altzp, Clear),
        (0x09, Write) => (Altzp, Set),
        (0x16, Read) => (Altzp, Query),

        _ => panic!("$c0{lo:02x}"),
    }
}
