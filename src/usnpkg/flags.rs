use std::fmt;
// struct Reason(pub u32);
//
bitflags! {
    flags Flags: u32 {
        const FLAG_A       = 0b00000001,
        const FLAG_B       = 0b00000010,
        const FLAG_C       = 0b00000100,
        const FLAG_ABC     = FLAG_A.bits
                           | FLAG_B.bits
                           | FLAG_C.bits,
    }
}
//
// impl fmt::Display for Reason {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f,"{:?}",self.0)
//     }
// }
