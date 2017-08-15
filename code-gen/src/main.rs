use std::collections::HashMap;
use std::collections::HashSet;
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

// 先頭からN-2までのレジスタを割り当てて、残りはStore, Loadしてメモリに置く
fn allocate_registers1(opcodes: Vec<OpeCode>, REGISTER_NUM: usize) -> Vec<OpeCode> {
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
    Dead, Birth, Live, Used, EndPoint
}

impl LiveRangeCell {
    fn is_live(&self) -> bool {
        self == &LiveRangeCell::Birth || self == &LiveRangeCell::Live || self == &LiveRangeCell::Used
    }

    fn is_used(&self) -> bool {
        self == &LiveRangeCell::Birth || self == &LiveRangeCell::Used || self == &LiveRangeCell::EndPoint
    }
}

fn is_empty<T: PartialEq>(matrix_graph: &Vec<Vec<T>>, null_value: T) -> bool {
    for row in matrix_graph {
        for elem in row {
            if elem != &null_value {
                return false
            }
        }
    }
    return true
}

// Chatinのアルゴリズム(干渉グラフを用いる)
fn allocate_registers2(opcodes: Vec<OpeCode>, REGISTER_NUM: usize) -> Vec<OpeCode> {
    // レジスタは1から順に使用されていると仮定
    let register_num = opcodes.iter().map(|op| {
        match op.clone() {
            OpeCode::Add { dst, src1, src2 } => {
                vec![dst, src1, src2].iter().max_by_key(|reg| reg.id).unwrap().id
            },
            OpeCode::LdI { dst, value: _ } => dst.id,
            OpeCode::Store { dst: _, src } => src.id,
            OpeCode::Load { dst, src: _ } => dst.id,
            OpeCode::Print { src } => src.id,
        }
    }).max().unwrap() + 1;

    // 生存区間の生成
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
                live_range[src1.id][i] = LiveRangeCell::Used;
                live_range[src2.id][i] = LiveRangeCell::Used;
            },
            OpeCode::LdI { dst, value: _ } => {
                live_range[dst.id][i]  = LiveRangeCell::Birth;
            },
            OpeCode::Store { dst: _, src } => {
                live_range[src.id][i]  = LiveRangeCell::Used;
            },
            OpeCode::Load { dst, src: _ } => {
                live_range[dst.id][i]  = LiveRangeCell::Birth;
            },
            OpeCode::Print { src } => {
                live_range[src.id][i]  = LiveRangeCell::Used;
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
                LiveRangeCell::Live | LiveRangeCell::Used => {
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

    let mut reg_map: HashMap<usize, usize> = HashMap::new();
    let mut spilled_reg: Vec<usize> = Vec::new();

    loop {
        let register_num = live_range.len();

        // for (reg_id, row) in live_range.iter().enumerate() {
        //     if reg_id == 0 {
        //         continue;
        //     }
        //
        //     print!("{:02}: ", reg_id);
        //     for cell in row.iter() {
        //         let ch = match *cell {
        //             LiveRangeCell::Dead => '.',
        //             LiveRangeCell::Birth => '*',
        //             LiveRangeCell::Live => '-',
        //             LiveRangeCell::Used => '=',
        //             LiveRangeCell::EndPoint => 'x',
        //         };
        //         print!("{}", ch);
        //     }
        //     println!("");
        // }

        // 干渉グラフの生成
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
                        interf_matrix[reg_id1][reg_id2] = true;
                        interf_matrix[reg_id2][reg_id1] = true;
                    }
                }
            }
        }

        // for (i, row) in interf_matrix.iter().enumerate() {
        //     for (j, cell) in row.iter().enumerate() {
        //         if i == 0 || j == 0 {
        //             continue;
        //         }
        //
        //         let ch = if *cell {
        //                 'X'
        //             } else if i == j {
        //                 '\\'
        //             } else {
        //                 '.'
        //             };
        //         print!("{}", ch);
        //     }
        //     println!("");
        // }

        let mut spill_list: Vec<usize> = Vec::new();
        let mut removed_regs: Vec<usize> = Vec::new();

        let mut interf_matrix_cloned = interf_matrix.clone();

        while !is_empty(&interf_matrix_cloned, false) || removed_regs.len() < register_num {
            // iとつながっているノードの個数
            let im = interf_matrix_cloned.clone();
            let degs = im.iter().map(|row| row.iter().filter(|&&cell| cell).count()).collect::<Vec<_>>();

            let threshold = REGISTER_NUM - 2;

            let reg_id = degs.iter().enumerate().position(|(reg_id, &deg)| {
                    deg < threshold && removed_regs.iter().find(|&&r| r == reg_id) == None
                })
                .unwrap_or_else(|| {
                    let reg_id = degs.iter().position(|&deg| deg >= threshold).unwrap();
                    spill_list.push(reg_id);
                    reg_id
                });
            // 干渉グラフからreg_idを取り除く
            for i in 0..register_num {
                interf_matrix_cloned[i][reg_id] = false;
                interf_matrix_cloned[reg_id][i] = false;
            }
            removed_regs.push(reg_id);
        }

        if spill_list.len() == 0 {
            // 塗る
            for &reg_id in removed_regs.iter().rev() {
                let mut is_painted: Vec<bool> = Vec::new();
                is_painted.resize(REGISTER_NUM + 1, false);

                for (reg_id2, &connected) in interf_matrix[reg_id].iter().enumerate() {
                    if connected {
                        if let Some(&color) = reg_map.get(&reg_id2) {
                            is_painted[color] = true;
                        }
                    }
                }

                let mut color = 1;
                for _ in 0..is_painted.len() {
                    if !is_painted[color] {
                        break;
                    }
                    color += 1;
                }

                reg_map.insert(reg_id, color);
            }

            break;
        } else {
            // spill
            // for &reg_id in &spill_list {
            //     for i in 0..live_range[reg_id].len() {
            //         if live_range[reg_id][i].is_used() {
            //             let mut new_range: Vec<LiveRangeCell> = Vec::new();
            //             new_range.resize(opcodes.len(), LiveRangeCell::Dead);
            //             new_range[i] = LiveRangeCell::Birth;
            //             live_range.push(new_range);
            //         }
            //     }
            // }

            spilled_reg = spill_list.clone();

            for &reg_id in &spill_list {
                for i in 0..live_range[reg_id].len() {
                    live_range[reg_id][i] = LiveRangeCell::Dead;
                }
            }
        }
    }

    // for (r1, r2) in &reg_map {
    //     println!("{} -> {}", r1, r2);
    // }

    for &reg_id in &spilled_reg {
        println!("{}", reg_id);
    }

    let mut result: Vec<OpeCode> = Vec::new();

    // register id -> address
    let mut reg_addr_map: HashMap<usize, usize> = HashMap::new();

    // for spilled registers
    let alloc_dst_reg = |reg_id: usize, original_reg_id: usize, reg_addr_map: &mut HashMap<usize, usize>| {
        let temp_reg = REGISTER_NUM - 1;

        if spilled_reg.iter().find(|&&r| r == reg_id) == None {
            (reg!(reg_id), None)
        } else {
            let new_addr = reg_addr_map.len();
            reg_addr_map.entry(original_reg_id).or_insert(new_addr);

            (reg!(temp_reg), Some(Integer::new(*reg_addr_map.get(&original_reg_id).unwrap() as i32)))
        }
    };

    let alloc_src_reg = |reg_id: usize, original_reg_id: usize, temp_reg: usize, reg_addr_map: &mut HashMap<usize, usize>, result: &mut Vec<OpeCode>| {
        if spilled_reg.iter().find(|&&r| r == reg_id) == None {
            reg!(reg_id)
        } else {
            let new_addr = reg_addr_map.len();
            reg_addr_map.entry(original_reg_id).or_insert(new_addr);

            let addr = Integer::new(*reg_addr_map.get(&original_reg_id).unwrap() as i32);
            result.push(OpeCode::Load{ dst: reg!(temp_reg), src: addr });

            reg!(temp_reg)
        }
    };

    for opcode in &opcodes {
        let opcode = opcode.clone();
        match opcode {
            OpeCode::LdI { dst, value } => {
                let &mdst = reg_map.get(&dst.id).unwrap();

                match alloc_dst_reg(mdst, dst.id, &mut reg_addr_map) {
                    (reg, None) => result.push(OpeCode::LdI{ dst: reg, value: value }),
                    (reg, Some(addr)) => {
                        result.push(OpeCode::LdI{ dst: reg.clone(), value: value});
                        result.push(OpeCode::Store{ dst: addr, src: reg});
                    },
                }
            },
            OpeCode::Add { dst, src1, src2 } => {
                let &mdst = reg_map.get(&dst.id).unwrap();
                let &msrc1 = reg_map.get(&src1.id).unwrap();
                let &msrc2 = reg_map.get(&src2.id).unwrap();

                let msrc1 = alloc_src_reg(msrc1, src1.id, REGISTER_NUM - 1, &mut reg_addr_map, &mut result);
                let msrc2 = alloc_src_reg(msrc2, src2.id, REGISTER_NUM,     &mut reg_addr_map, &mut result);

                match alloc_dst_reg(mdst, dst.id, &mut reg_addr_map) {
                    (reg, None) => result.push(OpeCode::Add{ dst: reg, src1: msrc1, src2: msrc2 }),
                    (reg, Some(addr)) => {
                        result.push(OpeCode::Add{ dst: reg.clone(), src1: msrc1, src2: msrc2 });
                        result.push(OpeCode::Store{ dst: addr, src: reg});
                    },
                }
            },
            OpeCode::Store { dst, src } => {
                let &msrc = reg_map.get(&src.id).unwrap();
                let msrc = alloc_src_reg(msrc, src.id, REGISTER_NUM, &mut reg_addr_map, &mut result);
                result.push(OpeCode::Store{ dst: dst, src: msrc });
            },
            OpeCode::Load { dst, src } => {
                let &mdst = reg_map.get(&dst.id).unwrap();

                match alloc_dst_reg(mdst, dst.id, &mut reg_addr_map) {
                    (reg, None) => result.push(OpeCode::Load{ dst: reg, src: src }),
                    (reg, Some(addr)) => {
                        result.push(OpeCode::Load{ dst: reg.clone(), src: src });
                        result.push(OpeCode::Store{ dst: addr, src: reg});
                    },
                }
            },
            OpeCode::Print { src } => {
                let &msrc = reg_map.get(&src.id).unwrap();
                let msrc = alloc_src_reg(msrc, src.id, REGISTER_NUM, &mut reg_addr_map, &mut result);
                result.push(OpeCode::Print{ src: msrc });
            },
        }
    }

    result
}

fn run_vm(opcodes: Vec<OpeCode>, REGISTER_NUM: usize) {
    let mut reg = Vec::new();
    reg.resize(REGISTER_NUM + 1, 0);
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

    // println!("");
    // println!("registers");
    // for i in 1..reg.len() {
    //     println!("  %{} = {}", i, reg[i]);
    // }
    // println!("memory");
    // for addr in 0..mem.len() {
    //     if mem[addr] != 0 {
    //         println!("  {}: {}", addr, mem[addr]);
    //     }
    // }
}

fn main() {
    let opcodes: Vec<OpeCode> = vec![
        // OpeCode::LdI{ dst: reg!(1), value: int!(1)},
        // OpeCode::LdI{ dst: reg!(2), value: int!(2)},
        // OpeCode::LdI{ dst: reg!(3), value: int!(3)},
        // OpeCode::LdI{ dst: reg!(4), value: int!(4)},
        //
        // OpeCode::Add{ dst: reg!(5), src1: reg!(1), src2: reg!(2)},
        // OpeCode::Add{ dst: reg!(6), src1: reg!(5), src2: reg!(3)},
        // OpeCode::Add{ dst: reg!(7), src1: reg!(6), src2: reg!(4)},
        //
        // OpeCode::Print{ src: reg!(7) }, // => 10

        OpeCode::LdI{ dst: reg!(1), value: int!(1)},
        OpeCode::LdI{ dst: reg!(2), value: int!(2)},
        OpeCode::LdI{ dst: reg!(3), value: int!(3)},
        OpeCode::LdI{ dst: reg!(4), value: int!(4)},
        OpeCode::LdI{ dst: reg!(5), value: int!(5)},
        OpeCode::LdI{ dst: reg!(6), value: int!(6)},

        OpeCode::Add{ dst: reg!(7), src1: reg!(1), src2: reg!(2)},
        OpeCode::Add{ dst: reg!(8), src1: reg!(3), src2: reg!(4)},
        OpeCode::Add{ dst: reg!(9), src1: reg!(5), src2: reg!(6)},

        OpeCode::Print{ src: reg!(7) }, // => 3
        OpeCode::Print{ src: reg!(8) }, // => 7
        OpeCode::Print{ src: reg!(9) }, // => 11
    ];

    // for opcode in &opcodes {
    //     println!("{}", opcode.to_string());
    // }

    // println!("");

    println!("reg num, algo1, algo2");

    for i in 4..10 {
        let opcodes1 = allocate_registers1(opcodes.clone(), i);
        let opcodes2 = allocate_registers2(opcodes.clone(), i);

        println!("algo1");
        for opcode in &opcodes1 {
            println!("{}", opcode.to_string());
        }
        println!("");

        println!("algo2");
        for opcode in &opcodes2 {
            println!("{}", opcode.to_string());
        }
        println!("");

        println!("{}, {}, {}", i, opcodes1.len(), opcodes2.len());

        // run_vm(opcodes1, i);
        // run_vm(opcodes2, i);
    }


}
