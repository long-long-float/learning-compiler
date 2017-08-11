use std::collections::HashMap;
use std::fmt;

trait Operand {
}

#[derive(Clone)]
struct Register {
    id: usize,     // 1 base
    size: usize,
}

impl Register {
    fn new(id: usize) -> Register {
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
    Print { src: Register },
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

const REGISTER_NUM: usize = 4;

// 先頭からN-2までのレジスタを割り当てて、残りはStore, Loadしてメモリに置く
fn allocate_registers1(opcodes: Vec<OpeCode>) -> Vec<OpeCode> {
    let mut result: Vec<OpeCode> = Vec::new();

    // register id -> address
    let mut reg_addr_map: HashMap<usize, usize> = HashMap::new();

    let alloc_dst_reg = |reg: Register, reg_addr_map: &mut HashMap<usize, usize>| {
        let temp_reg = REGISTER_NUM - 1;

        if reg.id <= REGISTER_NUM - 2 {
            (reg, None)
        } else {
            let new_addr = reg_addr_map.len();
            reg_addr_map.entry(reg.id).or_insert(new_addr);

            (reg!(temp_reg), Some(Integer::new(*reg_addr_map.get(&reg.id).unwrap() as i32)))
        }
    };

    let alloc_src_reg = |reg: Register, temp_reg: usize, reg_addr_map: &mut HashMap<usize, usize>, result: &mut Vec<OpeCode>| {
        if reg.id <= REGISTER_NUM - 2 {
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
                let src1 = alloc_src_reg(src1, REGISTER_NUM - 1, &mut reg_addr_map, &mut result);
                let src2 = alloc_src_reg(src2, REGISTER_NUM,     &mut reg_addr_map, &mut result);

                match alloc_dst_reg(dst, &mut reg_addr_map) {
                    (reg, None) => result.push(OpeCode::Add{ dst: reg, src1: src1, src2: src2 }),
                    (reg, Some(addr)) => {
                        result.push(OpeCode::Add{ dst: reg.clone(), src1: src1, src2: src2 });
                        result.push(OpeCode::Store{ dst: addr, src: reg });
                    },
                }
            },
            OpeCode::Store { dst, src } => {
                let src = alloc_src_reg(src, REGISTER_NUM, &mut reg_addr_map, &mut result);
                result.push(OpeCode::Store{ dst: dst, src: src });
            },
            OpeCode::Load { dst, src } => {
                match alloc_dst_reg(dst, &mut reg_addr_map) {
                    (reg, None) => result.push(OpeCode::Load{ dst: reg, src: src }),
                    (reg, Some(addr)) => {
                        result.push(OpeCode::Load{ dst: reg.clone(), src: src });
                        result.push(OpeCode::Store{ dst: addr, src: reg });
                    },
                }
            },
            OpeCode::Print { src } => {
                let src = alloc_src_reg(src, REGISTER_NUM, &mut reg_addr_map, &mut result);
                result.push(OpeCode::Print{ src: src });
            },
        }
    }

    result
}

fn run_vm(opcodes: Vec<OpeCode>) {
    let mut reg = [0; REGISTER_NUM + 1];
    let mut mem = [0; 1024];

    for opcode in &opcodes {
        let opcode = opcode.clone();
        match opcode {
            OpeCode::LdI { dst, value } => {
                reg[dst.id] = value.value;
            },
            OpeCode::Add { dst, src1, src2 } => {
                reg[dst.id] = reg[src1.id] + reg[src2.id];
            },
            OpeCode::Store { dst, src } => {
                mem[dst.value as usize] = reg[src.id];
            },
            OpeCode::Load { dst, src } => {
                reg[dst.id] = mem[src.value as usize];
            },
            OpeCode::Print { src } => {
                println!("{}", reg[src.id]);
            },
        }
    }

    println!("");
    println!("registers");
    for i in 1..reg.len() {
        println!("  %{} = {}", i, reg[i]);
    }
    println!("memory");
    for addr in 0..mem.len() {
        if mem[addr] != 0 {
            println!("  {}: {}", addr, mem[addr]);
        }
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

        OpeCode::Print{ src: reg!(7) },
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
