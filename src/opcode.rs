#[derive(Debug, Clone, Copy, PartialEq, int_enum::IntEnum)]
#[repr(u8)]
pub enum OpCode {
    Return = 0,
    Goto = 1,
    GotoIf = 2,
    Load = 3,
    Store = 4,
    ImmI = 5,
    ImmF = 6,
    ImmW = 7,
    AddI = 8,
    SubI = 9,
    MulI = 10,
    DivI = 11,
    CmpEqI = 12,
    CmpGtI = 13,
    CmpGeI = 14,
    CmpLtI = 15,
    CmpLeI = 16,
}