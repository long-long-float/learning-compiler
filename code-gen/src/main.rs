use std::collections::HashMap;
use std::fmt;

trait Operand {
}

#[derive(Clone)]
struct Register {
    id: i32,     // 1 base
    size: usize,
}

impl Register {
    fn new(id: i32) -> Register {
        Register {
            id: id,
            size: 32,
        }
    }
}

impl Operand for Register {}

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", self.id)
    }
}

#[derive(Clone)]
struct Integer {
    value: i32,
}

impl Integer {
    fn new(value: i32) -> Integer {
        Integer {
            value: value
        }
    }
}

impl Operand for Integer {}

impl fmt::Debug for Integer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone)]
enum OpeCode {
    Add { dst: Register, src1: Register, src2: Register },
    LdI { dst: Register, value: Integer },
    Store { dst: Integer, src: Register },
    Load { dst: Register, src: Integer },
}

impl OpeCode {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

// trait OpeCode {
//     fn to_string(&self) -> String;
// }
//
// macro_rules! def_opecode {
//     ($opname:ident, $($name:ident : $t:ident)*) => {
//         struct $opname {
//             $(
//                 $name: $t,
//             )*
//         }
//
//         impl $opname {
//             fn new($( $name: $t, )*) -> $opname {
//                 $opname {
//                     $( $name: $name, )*
//                 }
//             }
//         }
//
//         impl OpeCode for $opname {
//             fn to_string(&self) -> String {
//                 format!("$opname {}, {}, {}", $(self.$name.to_string(),)*)
//             }
//         }
//     };
// }
//
// // ここではマクロは使えない?
// // def_opecode![Add, dst: Register, src1: Register, src2: Register];
// // def_opecode![LdI, dst: Register, value: Integer];
//

macro_rules! boxed_vec {
    ($( $op:expr ),*) => {
        vec![
            $(
                Box::new($op),
            )*
        ]
    };
}

macro_rules! reg {
    ($id:expr) => {
        Register::new($id)
    };
}

macro_rules! int {
    ($value:expr) => {
        Integer::new($value)
    };
}

const REGISTER_NUM: i32 = 4;

// 先頭からN-1までのレジスタを割り当てて、残りは1つのレジスタを使いStore, Loadする
fn allocate_registers1(opcodes: Vec<OpeCode>) -> Vec<OpeCode> {
    let mut result: Vec<OpeCode> = Vec::new();

    // register id -> address
    let mut reg_addr_map: HashMap<i32, usize> = HashMap::new();

    let temp_reg = REGISTER_NUM;

    let mut alloc_dst_reg = |reg: Register, reg_addr_map: &mut HashMap<i32, usize>| {
        if reg.id <= REGISTER_NUM - 1 {
            (reg, None)
        } else {
            let new_addr = reg_addr_map.len();
            reg_addr_map.entry(reg.id).or_insert(new_addr);

            (reg!(temp_reg), Some(Integer::new(*reg_addr_map.get(&reg.id).unwrap() as i32)))
        }
    };

    let mut alloc_src_reg = |reg: Register, reg_addr_map: &mut HashMap<i32, usize>, result: &mut Vec<OpeCode>| {
        if reg.id <= REGISTER_NUM - 1 {
            reg
        } else {
            let new_addr = reg_addr_map.len();
            reg_addr_map.entry(reg.id).or_insert(new_addr);

            let addr = Integer::new(*reg_addr_map.get(&reg.id).unwrap() as i32);
            result.push(OpeCode::Load{ dst: reg!(temp_reg), src: addr });

            reg!(temp_reg)
        }
    };

    for opcode in &opcodes {
        let opcode = opcode.clone();
        match opcode {
            OpeCode::LdI { dst, value } => {
                match alloc_dst_reg(dst, &mut reg_addr_map) {
                    (reg, None) => result.push(OpeCode::LdI{ dst: reg, value: value }),
                    (reg, Some(addr)) => {
                        result.push(OpeCode::LdI{ dst: reg.clone(), value: value});
                        result.push(OpeCode::Store{ dst: addr, src: reg});
                    },
                }
            },
            OpeCode::Add { dst, src1, src2 } => {
                let src1 = alloc_src_reg(src1, &mut reg_addr_map, &mut result);
                let src2 = alloc_src_reg(src2, &mut reg_addr_map, &mut result);

                match alloc_dst_reg(dst, &mut reg_addr_map) {
                    (reg, None) => result.push(OpeCode::Add{ dst: reg, src1: src1, src2: src2 }),
                    (reg, Some(addr)) => {
                        result.push(OpeCode::Add{ dst: reg.clone(), src1: src1, src2: src2 });
                        result.push(OpeCode::Store{ dst: addr, src: reg });
                    },
                }
            },
            _ => {}
        }
    }

    result
}

fn run_vm(opcodes: Vec<OpeCode>) {
    for opcode in &opcodes {
    }
}

fn main() {
    let opcodes: Vec<OpeCode> = vec![
        OpeCode::LdI{ dst: reg!(1), value: int!(1)},
        OpeCode::LdI{ dst: reg!(2), value: int!(2)},
        OpeCode::LdI{ dst: reg!(3), value: int!(3)},
        OpeCode::LdI{ dst: reg!(4), value: int!(4)},

        OpeCode::Add{ dst: reg!(5), src1: reg!(1), src2: reg!(2)},
        OpeCode::Add{ dst: reg!(6), src1: reg!(5), src2: reg!(3)},
        OpeCode::Add{ dst: reg!(7), src1: reg!(6), src2: reg!(4)},
    ];

    for opcode in &opcodes {
        println!("{}", opcode.to_string());
    }

    println!("");

    let opcodes2 = allocate_registers1(opcodes);

    for opcode in &opcodes2 {
        println!("{}", opcode.to_string());
    }

    run_vm(opcodes2);
}
