use super::super::register::Flag;
use super::instruction::{Instruction, InstructionOpcode, InstructionState, InstructionStep};
use super::opcodes::*;

pub struct InstructionCache {
    interrupt_service_routine: Instruction,
    instructions: [Instruction; 256],
    cb_instructions: [Instruction; 256],
}

impl Default for InstructionCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionCache {
    pub fn new() -> Self {
        Self {
            interrupt_service_routine: interrupt_service_routine(),
            instructions: Self::gen_instructions(),
            cb_instructions: Self::gen_cb_instructions(),
        }
    }

    fn gen_instructions() -> [Instruction; 256] {
        [
            nop(),
            ld_r16_u16!(bc),
            ld_mem_a!(bc),
            inc_r16!(bc),
            inc_r8!(bc, hi),
            dec_r8!(bc, hi),
            ld_r8_u8!(bc, hi),
            rlca(),
            ld_u16_sp(),
            add_hl_r16!(bc),
            ld_a_mem!(bc),
            dec_r16!(bc),
            inc_r8!(bc, lo),
            dec_r8!(bc, lo),
            ld_r8_u8!(bc, lo),
            rrca(),
            stop(),
            ld_r16_u16!(de),
            ld_mem_a!(de),
            inc_r16!(de),
            inc_r8!(de, hi),
            dec_r8!(de, hi),
            ld_r8_u8!(de, hi),
            rla(),
            jr_i8!(),
            add_hl_r16!(de),
            ld_a_mem!(de),
            dec_r16!(de),
            inc_r8!(de, lo),
            dec_r8!(de, lo),
            ld_r8_u8!(de, lo),
            rra(),
            jr_i8!(NZ),
            ld_r16_u16!(hl),
            ld_mem_a!(hlplus),
            inc_r16!(hl),
            inc_r8!(hl, hi),
            dec_r8!(hl, hi),
            ld_r8_u8!(hl, hi),
            daa(),
            jr_i8!(Z),
            add_hl_r16!(hl),
            ld_a_mem!(hlplus),
            dec_r16!(hl),
            inc_r8!(hl, lo),
            dec_r8!(hl, lo),
            ld_r8_u8!(hl, lo),
            cpl(),
            jr_i8!(NC),
            ld_r16_u16!(sp),
            ld_mem_a!(hlminus),
            inc_r16!(sp),
            inc_hl(),
            dec_hl(),
            ld_hlmem_u8(),
            scf(),
            jr_i8!(C),
            add_hl_r16!(sp),
            ld_a_mem!(hlminus),
            dec_r16!(sp),
            inc_r8!(af, hi),
            dec_r8!(af, hi),
            ld_r8_u8!(af, hi),
            ccf(),
            // ld b, r8
            ld_r8_r8!(bc, hi <= bc, hi),
            ld_r8_r8!(bc, hi <= bc, lo),
            ld_r8_r8!(bc, hi <= de, hi),
            ld_r8_r8!(bc, hi <= de, lo),
            ld_r8_r8!(bc, hi <= hl, hi),
            ld_r8_r8!(bc, hi <= hl, lo),
            ld_r8_r8!(bc, hi <= HL),
            ld_r8_r8!(bc, hi <= af, hi),
            // ld c, r8
            ld_r8_r8!(bc, lo <= bc, hi),
            ld_r8_r8!(bc, lo <= bc, lo),
            ld_r8_r8!(bc, lo <= de, hi),
            ld_r8_r8!(bc, lo <= de, lo),
            ld_r8_r8!(bc, lo <= hl, hi),
            ld_r8_r8!(bc, lo <= hl, lo),
            ld_r8_r8!(bc, lo <= HL),
            ld_r8_r8!(bc, lo <= af, hi),
            // ld d, r8
            ld_r8_r8!(de, hi <= bc, hi),
            ld_r8_r8!(de, hi <= bc, lo),
            ld_r8_r8!(de, hi <= de, hi),
            ld_r8_r8!(de, hi <= de, lo),
            ld_r8_r8!(de, hi <= hl, hi),
            ld_r8_r8!(de, hi <= hl, lo),
            ld_r8_r8!(de, hi <= HL),
            ld_r8_r8!(de, hi <= af, hi),
            // ld e, r8
            ld_r8_r8!(de, lo <= bc, hi),
            ld_r8_r8!(de, lo <= bc, lo),
            ld_r8_r8!(de, lo <= de, hi),
            ld_r8_r8!(de, lo <= de, lo),
            ld_r8_r8!(de, lo <= hl, hi),
            ld_r8_r8!(de, lo <= hl, lo),
            ld_r8_r8!(de, lo <= HL),
            ld_r8_r8!(de, lo <= af, hi),
            // ld h, r8
            ld_r8_r8!(hl, hi <= bc, hi),
            ld_r8_r8!(hl, hi <= bc, lo),
            ld_r8_r8!(hl, hi <= de, hi),
            ld_r8_r8!(hl, hi <= de, lo),
            ld_r8_r8!(hl, hi <= hl, hi),
            ld_r8_r8!(hl, hi <= hl, lo),
            ld_r8_r8!(hl, hi <= HL),
            ld_r8_r8!(hl, hi <= af, hi),
            // ld l, r8
            ld_r8_r8!(hl, lo <= bc, hi),
            ld_r8_r8!(hl, lo <= bc, lo),
            ld_r8_r8!(hl, lo <= de, hi),
            ld_r8_r8!(hl, lo <= de, lo),
            ld_r8_r8!(hl, lo <= hl, hi),
            ld_r8_r8!(hl, lo <= hl, lo),
            ld_r8_r8!(hl, lo <= HL),
            ld_r8_r8!(hl, lo <= af, hi),
            // ld (HL), r8
            ld_memhl_r8!(HL <= bc, hi),
            ld_memhl_r8!(HL <= bc, lo),
            ld_memhl_r8!(HL <= de, hi),
            ld_memhl_r8!(HL <= de, lo),
            ld_memhl_r8!(HL <= hl, hi),
            ld_memhl_r8!(HL <= hl, lo),
            halt(),
            ld_memhl_r8!(HL <= af, hi),
            // ld a, r8
            ld_r8_r8!(af, hi <= bc, hi),
            ld_r8_r8!(af, hi <= bc, lo),
            ld_r8_r8!(af, hi <= de, hi),
            ld_r8_r8!(af, hi <= de, lo),
            ld_r8_r8!(af, hi <= hl, hi),
            ld_r8_r8!(af, hi <= hl, lo),
            ld_r8_r8!(af, hi <= HL),
            ld_r8_r8!(af, hi <= af, hi),
            // add a, r8
            add_a_r8!(b),
            add_a_r8!(c),
            add_a_r8!(d),
            add_a_r8!(e),
            add_a_r8!(h),
            add_a_r8!(l),
            add_a_r8!(hl),
            add_a_r8!(a),
            // adc a, r8
            adc_a_r8!(b),
            adc_a_r8!(c),
            adc_a_r8!(d),
            adc_a_r8!(e),
            adc_a_r8!(h),
            adc_a_r8!(l),
            adc_a_r8!(hl),
            adc_a_r8!(a),
            // sub a, r8
            sub_a_r8!(b),
            sub_a_r8!(c),
            sub_a_r8!(d),
            sub_a_r8!(e),
            sub_a_r8!(h),
            sub_a_r8!(l),
            sub_a_r8!(hl),
            sub_a_r8!(a),
            // sbc a, r8
            sbc_a_r8!(b),
            sbc_a_r8!(c),
            sbc_a_r8!(d),
            sbc_a_r8!(e),
            sbc_a_r8!(h),
            sbc_a_r8!(l),
            sbc_a_r8!(hl),
            sbc_a_r8!(a),
            // and a, r8
            and_a_r8!(b),
            and_a_r8!(c),
            and_a_r8!(d),
            and_a_r8!(e),
            and_a_r8!(h),
            and_a_r8!(l),
            and_a_r8!(hl),
            and_a_r8!(a),
            // xor a, r8
            xor_a_r8!(b),
            xor_a_r8!(c),
            xor_a_r8!(d),
            xor_a_r8!(e),
            xor_a_r8!(h),
            xor_a_r8!(l),
            xor_a_r8!(hl),
            xor_a_r8!(a),
            // or a, r8
            or_a_r8!(b),
            or_a_r8!(c),
            or_a_r8!(d),
            or_a_r8!(e),
            or_a_r8!(h),
            or_a_r8!(l),
            or_a_r8!(hl),
            or_a_r8!(a),
            // cp a, r8
            cp_a_r8!(b),
            cp_a_r8!(c),
            cp_a_r8!(d),
            cp_a_r8!(e),
            cp_a_r8!(h),
            cp_a_r8!(l),
            cp_a_r8!(hl),
            cp_a_r8!(a),
            ret_cc!(NZ),
            pop_r16!(bc),
            jp_cc_u16!(NZ),
            jp_u16!(),
            call_cc_u16!(NZ),
            push_r16!(bc),
            add_a_r8!(u8),
            rst_yy!(0),
            ret_cc!(Z),
            ret!(),
            jp_cc_u16!(Z),
            instruction!(InstructionStep::Instant(|_, _| unimplemented!("CB PREFIX"))),
            call_cc_u16!(Z),
            call_u16!(),
            adc_a_r8!(u8),
            rst_yy!(0x08),
            ret_cc!(NC),
            pop_r16!(de),
            jp_cc_u16!(NC),
            unused_opcode!("0xD3"),
            call_cc_u16!(NC),
            push_r16!(de),
            sub_a_r8!(u8),
            rst_yy!(0x10),
            ret_cc!(C),
            ret!(i),
            jp_cc_u16!(C),
            unused_opcode!("0xDB"),
            call_cc_u16!(C),
            unused_opcode!("0xDD"),
            sbc_a_r8!(u8),
            rst_yy!(0x18),
            ld_ff00_u8_a(),
            pop_r16!(hl),
            ld_ff00_c_a(),
            unused_opcode!("0xE3"),
            unused_opcode!("0xE4"),
            push_r16!(hl),
            and_a_r8!(u8),
            rst_yy!(0x20),
            add_sp_i8(),
            jp_hl(),
            ld_mem_u16_a(),
            unused_opcode!("0xEB"),
            unused_opcode!("0xEC"),
            unused_opcode!("0xED"),
            xor_a_r8!(u8),
            rst_yy!(0x28),
            ld_a_ff00_u8(),
            pop_r16!(af),
            ld_a_ff00_c(),
            di(),
            unused_opcode!("0xF4"),
            push_r16!(af),
            or_a_r8!(u8),
            rst_yy!(0x30),
            ld_hl_sp_i8(),
            ld_sp_hl(),
            ld_a_mem_u16(),
            ei(),
            unused_opcode!("0xFC"),
            unused_opcode!("0xFD"),
            cp_a_r8!(u8),
            rst_yy!(0x38),
        ]
    }

    fn gen_cb_instructions() -> [Instruction; 256] {
        [
            cb_op_instr!(rlc, b),
            cb_op_instr!(rlc, c),
            cb_op_instr!(rlc, d),
            cb_op_instr!(rlc, e),
            cb_op_instr!(rlc, h),
            cb_op_instr!(rlc, l),
            cb_op_hl_instr!(rlc, hl),
            cb_op_instr!(rlc, a),
            cb_op_instr!(rrc, b),
            cb_op_instr!(rrc, c),
            cb_op_instr!(rrc, d),
            cb_op_instr!(rrc, e),
            cb_op_instr!(rrc, h),
            cb_op_instr!(rrc, l),
            cb_op_hl_instr!(rrc, hl),
            cb_op_instr!(rrc, a),
            cb_op_instr!(rl, b),
            cb_op_instr!(rl, c),
            cb_op_instr!(rl, d),
            cb_op_instr!(rl, e),
            cb_op_instr!(rl, h),
            cb_op_instr!(rl, l),
            cb_op_hl_instr!(rl, hl),
            cb_op_instr!(rl, a),
            cb_op_instr!(rr, b),
            cb_op_instr!(rr, c),
            cb_op_instr!(rr, d),
            cb_op_instr!(rr, e),
            cb_op_instr!(rr, h),
            cb_op_instr!(rr, l),
            cb_op_hl_instr!(rr, hl),
            cb_op_instr!(rr, a),
            cb_op_instr!(sla, b),
            cb_op_instr!(sla, c),
            cb_op_instr!(sla, d),
            cb_op_instr!(sla, e),
            cb_op_instr!(sla, h),
            cb_op_instr!(sla, l),
            cb_op_hl_instr!(sla, hl),
            cb_op_instr!(sla, a),
            cb_op_instr!(sra, b),
            cb_op_instr!(sra, c),
            cb_op_instr!(sra, d),
            cb_op_instr!(sra, e),
            cb_op_instr!(sra, h),
            cb_op_instr!(sra, l),
            cb_op_hl_instr!(sra, hl),
            cb_op_instr!(sra, a),
            cb_op_instr!(swap, b),
            cb_op_instr!(swap, c),
            cb_op_instr!(swap, d),
            cb_op_instr!(swap, e),
            cb_op_instr!(swap, h),
            cb_op_instr!(swap, l),
            cb_op_hl_instr!(swap, hl),
            cb_op_instr!(swap, a),
            cb_op_instr!(srl, b),
            cb_op_instr!(srl, c),
            cb_op_instr!(srl, d),
            cb_op_instr!(srl, e),
            cb_op_instr!(srl, h),
            cb_op_instr!(srl, l),
            cb_op_hl_instr!(srl, hl),
            cb_op_instr!(srl, a),
            bit!(0, b),
            bit!(0, c),
            bit!(0, d),
            bit!(0, e),
            bit!(0, h),
            bit!(0, l),
            bit!(0, hl),
            bit!(0, a),
            bit!(1, b),
            bit!(1, c),
            bit!(1, d),
            bit!(1, e),
            bit!(1, h),
            bit!(1, l),
            bit!(1, hl),
            bit!(1, a),
            bit!(2, b),
            bit!(2, c),
            bit!(2, d),
            bit!(2, e),
            bit!(2, h),
            bit!(2, l),
            bit!(2, hl),
            bit!(2, a),
            bit!(3, b),
            bit!(3, c),
            bit!(3, d),
            bit!(3, e),
            bit!(3, h),
            bit!(3, l),
            bit!(3, hl),
            bit!(3, a),
            bit!(4, b),
            bit!(4, c),
            bit!(4, d),
            bit!(4, e),
            bit!(4, h),
            bit!(4, l),
            bit!(4, hl),
            bit!(4, a),
            bit!(5, b),
            bit!(5, c),
            bit!(5, d),
            bit!(5, e),
            bit!(5, h),
            bit!(5, l),
            bit!(5, hl),
            bit!(5, a),
            bit!(6, b),
            bit!(6, c),
            bit!(6, d),
            bit!(6, e),
            bit!(6, h),
            bit!(6, l),
            bit!(6, hl),
            bit!(6, a),
            bit!(7, b),
            bit!(7, c),
            bit!(7, d),
            bit!(7, e),
            bit!(7, h),
            bit!(7, l),
            bit!(7, hl),
            bit!(7, a),
            res!(0, b),
            res!(0, c),
            res!(0, d),
            res!(0, e),
            res!(0, h),
            res!(0, l),
            res!(0, hl),
            res!(0, a),
            res!(1, b),
            res!(1, c),
            res!(1, d),
            res!(1, e),
            res!(1, h),
            res!(1, l),
            res!(1, hl),
            res!(1, a),
            res!(2, b),
            res!(2, c),
            res!(2, d),
            res!(2, e),
            res!(2, h),
            res!(2, l),
            res!(2, hl),
            res!(2, a),
            res!(3, b),
            res!(3, c),
            res!(3, d),
            res!(3, e),
            res!(3, h),
            res!(3, l),
            res!(3, hl),
            res!(3, a),
            res!(4, b),
            res!(4, c),
            res!(4, d),
            res!(4, e),
            res!(4, h),
            res!(4, l),
            res!(4, hl),
            res!(4, a),
            res!(5, b),
            res!(5, c),
            res!(5, d),
            res!(5, e),
            res!(5, h),
            res!(5, l),
            res!(5, hl),
            res!(5, a),
            res!(6, b),
            res!(6, c),
            res!(6, d),
            res!(6, e),
            res!(6, h),
            res!(6, l),
            res!(6, hl),
            res!(6, a),
            res!(7, b),
            res!(7, c),
            res!(7, d),
            res!(7, e),
            res!(7, h),
            res!(7, l),
            res!(7, hl),
            res!(7, a),
            set!(0, b),
            set!(0, c),
            set!(0, d),
            set!(0, e),
            set!(0, h),
            set!(0, l),
            set!(0, hl),
            set!(0, a),
            set!(1, b),
            set!(1, c),
            set!(1, d),
            set!(1, e),
            set!(1, h),
            set!(1, l),
            set!(1, hl),
            set!(1, a),
            set!(2, b),
            set!(2, c),
            set!(2, d),
            set!(2, e),
            set!(2, h),
            set!(2, l),
            set!(2, hl),
            set!(2, a),
            set!(3, b),
            set!(3, c),
            set!(3, d),
            set!(3, e),
            set!(3, h),
            set!(3, l),
            set!(3, hl),
            set!(3, a),
            set!(4, b),
            set!(4, c),
            set!(4, d),
            set!(4, e),
            set!(4, h),
            set!(4, l),
            set!(4, hl),
            set!(4, a),
            set!(5, b),
            set!(5, c),
            set!(5, d),
            set!(5, e),
            set!(5, h),
            set!(5, l),
            set!(5, hl),
            set!(5, a),
            set!(6, b),
            set!(6, c),
            set!(6, d),
            set!(6, e),
            set!(6, h),
            set!(6, l),
            set!(6, hl),
            set!(6, a),
            set!(7, b),
            set!(7, c),
            set!(7, d),
            set!(7, e),
            set!(7, h),
            set!(7, l),
            set!(7, hl),
            set!(7, a),
        ]
    }

    pub(in crate::cpu) fn get(&mut self, opcode: InstructionOpcode) -> &mut Instruction {
        match opcode {
            InstructionOpcode::Unprefixed(opcode) => &mut self.instructions[opcode as usize],
            InstructionOpcode::Prefixed(opcode) => &mut self.cb_instructions[opcode as usize],
            InstructionOpcode::InterruptServiceRoutine => &mut self.interrupt_service_routine,
        }
    }
}
