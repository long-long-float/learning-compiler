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

#[derive(Clone, PartialEq)]
enum LiveRangeCell {
    Dead, Birth, Live, EndPoint
}

impl LiveRangeCell {
    fn is_live(&self) -> bool {
        self == &LiveRangeCell::Birth || self == &LiveRangeCell::Live
    }
}

fn allocate_registers2(opcodes: Vec<OpeCode>) -> Vec<OpeCode> {
    // レジスタは1から順に使用されていると仮定
    let register_num = opcodes.iter().map(|op| {
        match op.clone() {
            OpeCode::Add { dst, src1, src2 } => {
                vec![dst, src1, src2].iter().max_by_key(|reg| reg.id).unwrap().id
            },
            OpeCode::LdI { dst, value } => dst.id,
            OpeCode::Store { dst, src } => src.id,
            OpeCode::Load { dst, src } => dst.id,
            OpeCode::Print { src } => src.id,
        }
    }).max().unwrap() + 1;

    let mut live_range: Vec<Vec<LiveRangeCell>> = Vec::new();
    for _ in 0..register_num {
        let mut row: Vec<LiveRangeCell> = Vec::new();
        row.resize(opcodes.len(), LiveRangeCell::Dead);
        live_range.push(row);
    }

    for (i, opcode) in opcodes.iter().enumerate() {
        match opcode.clone() {
            OpeCode::Add { dst, src1, src2 } => {
                live_range[dst.id][i]  = LiveRangeCell::Birth;
                live_range[src1.id][i] = LiveRangeCell::Live;
                live_range[src2.id][i] = LiveRangeCell::Live;
            },
            OpeCode::LdI { dst, value } => {
                live_range[dst.id][i]  = LiveRangeCell::Birth;
            },
            OpeCode::Store { dst, src } => {
                live_range[src.id][i]  = LiveRangeCell::Live;
            },
            OpeCode::Load { dst, src } => {
                live_range[dst.id][i]  = LiveRangeCell::Birth;
            },
            OpeCode::Print { src } => {
                live_range[src.id][i]  = LiveRangeCell::Live;
            },
        }
    }

    let mut living: Vec<bool> = Vec::new();
    living.resize(register_num, false);

    for reg_id in 0..live_range.len() {
        let row = &mut live_range[reg_id];
        for i in (0..row.len()).rev() {
            let cell = row[i].clone();
            match cell {
                LiveRangeCell::Dead => {
                    if living[reg_id] {
                        row[i] = LiveRangeCell::Live;
                    }
                },
                LiveRangeCell::Live => {
                    if !living[reg_id] {
                        row[i] = LiveRangeCell::EndPoint;
                    }
                    living[reg_id] = true;
                },
                LiveRangeCell::Birth => {
                    living[reg_id] = false;
                },
                LiveRangeCell::EndPoint => {},
            }
        }
    }

    for (reg_id, row) in live_range.iter().enumerate() {
        if reg_id == 0 {
            continue;
        }

        print!("{}: ", reg_id);
        for cell in row.iter() {
            let ch = match *cell {
                LiveRangeCell::Dead => '.',
                LiveRangeCell::Birth => '*',
                LiveRangeCell::Live => '-',
                LiveRangeCell::EndPoint => 'x',
            };
            print!("{}", ch);
        }
        println!("");
    }

    // 干渉グラフ
    let mut interf_matrix: Vec<Vec<bool>> = Vec::new();
    for _ in 0..register_num {
        let mut row: Vec<bool> = Vec::new();
        row.resize(register_num, false);
        interf_matrix.push(row);
    }

    for (reg_id1, row1) in live_range.clone().iter().enumerate() {
        for (reg_id2, row2) in live_range.iter().enumerate() {
            if reg_id1 == reg_id2 {
                continue
            }

            for i in 0..row1.len() {
                if row1[i].is_live() && row2[i].is_live() {
                    let (reg_id1, reg_id2) = if reg_id1 > reg_id2 {
                            (reg_id2, reg_id1)
                        } else {
                            (reg_id1, reg_id2)
                        };

                    interf_matrix[reg_id1][reg_id2] = true;
                }
            }
        }
    }

    for row in &interf_matrix {
        for cell in row {
            print!("{}", if *cell { 'X' } else { '.' });
        }
        println!("");
    }

    let mut result: Vec<OpeCode> = Vec::new();

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

        OpeCode::Print{ src: reg!(7) }, // => 10
    ];

    for opcode in &opcodes {
        println!("{}", opcode.to_string());
    }

    println!("");

    // let opcodes2 = allocate_registers1(opcodes);
    let opcodes2 = allocate_registers2(opcodes);

    for opcode in &opcodes2 {
        println!("{}", opcode.to_string());
    }

    run_vm(opcodes2);
}
