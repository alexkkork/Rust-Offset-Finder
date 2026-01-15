// Tue Jan 13 2026 - Alex

use crate::pattern::Pattern;

pub fn function_prologue() -> Pattern {
    Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91")
        .with_name("ARM64 Function Prologue")
}

pub fn function_prologue_save_regs() -> Pattern {
    Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9")
        .with_name("ARM64 Function Prologue with Saved Regs")
}

pub fn function_epilogue_ret() -> Pattern {
    Pattern::from_hex("C0 03 5F D6")
        .with_name("ARM64 RET")
}

pub fn function_epilogue_ldp_ret() -> Pattern {
    Pattern::from_hex("FD 7B ?? A8 C0 03 5F D6")
        .with_name("ARM64 LDP + RET")
}

pub fn branch_link() -> Pattern {
    Pattern::from_hex("?? ?? ?? 94")
        .with_name("ARM64 BL")
}

pub fn branch() -> Pattern {
    Pattern::from_hex("?? ?? ?? 14")
        .with_name("ARM64 B")
}

pub fn branch_conditional() -> Pattern {
    Pattern::from_hex("?? ?? ?? 54")
        .with_name("ARM64 B.cond")
}

pub fn adrp() -> Pattern {
    Pattern::from_hex("?? ?? ?? 90")
        .with_name("ARM64 ADRP")
}

pub fn adr() -> Pattern {
    Pattern::from_hex("?? ?? ?? 10")
        .with_name("ARM64 ADR")
}

pub fn ldr_x_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? F9")
        .with_name("ARM64 LDR X, [imm]")
}

pub fn str_x_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? F9")
        .with_name("ARM64 STR X, [imm]")
}

pub fn ldr_w_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? B9")
        .with_name("ARM64 LDR W, [imm]")
}

pub fn str_w_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? B9")
        .with_name("ARM64 STR W, [imm]")
}

pub fn ldrb_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? 39")
        .with_name("ARM64 LDRB [imm]")
}

pub fn strb_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? 39")
        .with_name("ARM64 STRB [imm]")
}

pub fn cmp_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? 71")
        .with_name("ARM64 CMP imm")
}

pub fn cmn_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? 31")
        .with_name("ARM64 CMN imm")
}

pub fn add_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? 91")
        .with_name("ARM64 ADD imm")
}

pub fn sub_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? D1")
        .with_name("ARM64 SUB imm")
}

pub fn mov_imm() -> Pattern {
    Pattern::from_hex("?? ?? ?? 52")
        .with_name("ARM64 MOV imm (MOVZ)")
}

pub fn movk() -> Pattern {
    Pattern::from_hex("?? ?? ?? 72")
        .with_name("ARM64 MOVK")
}

pub fn mov_reg() -> Pattern {
    Pattern::from_hex("?? ?? ?? AA")
        .with_name("ARM64 MOV reg")
}

pub fn cbz() -> Pattern {
    Pattern::from_hex("?? ?? ?? B4")
        .with_name("ARM64 CBZ")
}

pub fn cbnz() -> Pattern {
    Pattern::from_hex("?? ?? ?? B5")
        .with_name("ARM64 CBNZ")
}

pub fn tbz() -> Pattern {
    Pattern::from_hex("?? ?? ?? 36")
        .with_name("ARM64 TBZ")
}

pub fn tbnz() -> Pattern {
    Pattern::from_hex("?? ?? ?? 37")
        .with_name("ARM64 TBNZ")
}

pub fn blr() -> Pattern {
    Pattern::from_hex("?? ?? 3F D6")
        .with_name("ARM64 BLR")
}

pub fn br() -> Pattern {
    Pattern::from_hex("?? ?? 1F D6")
        .with_name("ARM64 BR")
}

pub fn stp_pre_index() -> Pattern {
    Pattern::from_hex("?? ?? ?? A9")
        .with_name("ARM64 STP pre-index")
}

pub fn ldp_post_index() -> Pattern {
    Pattern::from_hex("?? ?? ?? A8")
        .with_name("ARM64 LDP post-index")
}

pub fn nop() -> Pattern {
    Pattern::from_hex("1F 20 03 D5")
        .with_name("ARM64 NOP")
}

pub fn brk() -> Pattern {
    Pattern::from_hex("?? ?? 20 D4")
        .with_name("ARM64 BRK")
}

pub fn svc() -> Pattern {
    Pattern::from_hex("?? ?? 00 D4")
        .with_name("ARM64 SVC")
}

pub struct Arm64PatternSet {
    pub function_prologue: Pattern,
    pub function_epilogue: Pattern,
    pub call: Pattern,
    pub jump: Pattern,
    pub conditional_branch: Pattern,
    pub load: Pattern,
    pub store: Pattern,
    pub compare: Pattern,
}

impl Arm64PatternSet {
    pub fn new() -> Self {
        Self {
            function_prologue: function_prologue(),
            function_epilogue: function_epilogue_ret(),
            call: branch_link(),
            jump: branch(),
            conditional_branch: branch_conditional(),
            load: ldr_x_imm(),
            store: str_x_imm(),
            compare: cmp_imm(),
        }
    }

    pub fn all_patterns(&self) -> Vec<&Pattern> {
        vec![
            &self.function_prologue,
            &self.function_epilogue,
            &self.call,
            &self.jump,
            &self.conditional_branch,
            &self.load,
            &self.store,
            &self.compare,
        ]
    }
}

impl Default for Arm64PatternSet {
    fn default() -> Self {
        Self::new()
    }
}
