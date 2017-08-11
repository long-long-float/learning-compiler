use std::collections::HashMap;
use std::ops::Deref;
use std::any::Any;

trait Operand {
    fn to_string(&self) -> String;
}

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

impl Operand for Register {
    fn to_string(&self) -> String {
        format!("%{}", self.id)
    }
}

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

impl Operand for Integer {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

trait OpeCode {
    fn to_string(&self) -> String;
}

macro_rules! def_opecode {
    ($opname:ident, $($name:ident : $t:ident)*) => {
        struct $opname {
            $(
                $name: $t,
            )*
        }

        impl $opname {
            fn new($( $name: $t, )*) -> $opname {
                $opname {
                    $( $name: $name, )*
                }
            }
        }

        impl OpeCode for $opname {
            fn to_string(&self) -> String {
                format!("$opname {}, {}, {}", $(self.$name.to_string(),)*)
            }
        }
    };
}

// ここではマクロは使えない?
// def_opecode![Add, dst: Register, src1: Register, src2: Register];
// def_opecode![LdI, dst: Register, value: Integer];

struct Add {
    dst: Register,
    src1: Register,
    src2: Register,
}

impl Add {
    fn new(dst: Register, src1: Register, src2: Register) -> Add {
        Add {
            dst: dst, src1: src1, src2: src2,
        }
    }
}

impl OpeCode for Add {
    fn to_string(&self) -> String {
        format!("add {}, {}, {}", self.dst.to_string(), self.src1.to_string(), self.src2.to_string())
    }
}

struct LdI {
    dst: Register,
    value: Integer,
}

impl LdI {
    fn new(dst: Register, value: Integer) -> LdI {
        LdI {
            dst: dst, value: value,
        }
    }
}

impl OpeCode for LdI {
    fn to_string(&self) -> String {
        format!("ldi {}, {}", self.dst.to_string(), self.value.to_string())
    }
}

struct Store {
    dst: Integer,  // address
    src: Register,
}

impl Store {
    fn new(dst: Integer, src: Register) -> Store {
        Store {
            dst: dst, src: src,
        }
    }
}

impl OpeCode for Store {
    fn to_string(&self) -> String {
        format!("store {} ,{}", self.dst.to_string(), self.src.to_string())
    }
}

struct Load {
    dst: Register,
    src: Integer,  // address
}

impl Load {
    fn new(dst: Register, src: Integer) -> Load {
        Load {
            dst: dst, src: src,
        }
    }
}

impl OpeCode for Load {
    fn to_string(&self) -> String {
        format!("load {} ,{}", self.dst.to_string(), self.src.to_string())
    }
}


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
fn allocate_registers1(opcodes: Vec<Box<OpeCode>>) -> Vec<Box<OpeCode>> {
    let mut result: Vec<Box<OpeCode>> = Vec::new();

    // // register id <-> address
    // let mut reg_addr_map: HashMap<i32, i32> = HashMap::new();
    //
    // for opcode in &opcodes {
    //     let opcode: &Box<OpeCode> = opcode;
    //     match opcode.downcast_ref::<LdI>() {
    //         Some(&LdI { dst, value }) => {
    //             if dst.id <= REGISTER_NUM - 1 {
    //                 result.insert(opcode);
    //             } else {
    //                 reg_addr_map.entry(dst.id).or_insert(reg_addr_map.len());
    //
    //                 result.insert(LdI::new(reg!(REGISTER_NUM), value));
    //                 result.insert(Store::new(reg_addr_map.get(&dst.id).unwrap(), reg!(REGISTER_NUM)))
    //             }
    //         }
    //     }
    // }

    result
}

fn main() {
    let opcodes: Vec<Box<OpeCode>> = boxed_vec![
        LdI::new(reg!(1), int!(1)),
        LdI::new(reg!(2), int!(2)),
        LdI::new(reg!(3), int!(3)),
        LdI::new(reg!(4), int!(4)),
        LdI::new(reg!(5), int!(5))

        // Add::new(reg!(5), reg!(1), reg!(2)),
        // Add::new(reg!(6), reg!(5), reg!(3)),
        // Add::new(reg!(7), reg!(6), reg!(4))
    ];

    for opcode in &opcodes {
        println!("{}", opcode.to_string());
    }

    println!("");

    let opcodes2 = allocate_registers1(opcodes);

    for opcode in &opcodes2 {
        println!("{}", opcode.to_string());
    }
}
