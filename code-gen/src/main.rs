trait Operand {
    fn to_string(&self) -> String;
}

struct Register {
    id: i32,
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

struct OpCode {
    name: String,
    operands: Vec<Box<Operand>>,
}

impl OpCode {
    fn new(name: &str, operands: Vec<Box<Operand>>) -> OpCode {
        OpCode {
            name: name.to_string(),
            operands: operands,
        }
    }

    fn to_string(&self) -> String {
        let operands_str = self.operands.iter().map(|operand| operand.to_string()).collect::<Vec<_>>().join(", ");
        format!("{} {}", self.name, operands_str)
    }
}

macro_rules! op {
    ($name:expr, $( $op:expr ),*) => {
        OpCode::new($name, vec![
            $(
                Box::new($op),
            )*
        ])
    };
}

fn main() {
    let opcodes = vec![
        op!("ldi", Register::new(1), Integer::new(1)),
        op!("ldi", Register::new(2), Integer::new(2)),
        op!("ldi", Register::new(3), Integer::new(3)),
        op!("ldi", Register::new(4), Integer::new(4)),

        op!("add", Register::new(5), Register::new(1), Register::new(2)),
        op!("add", Register::new(6), Register::new(5), Register::new(3)),
        op!("add", Register::new(7), Register::new(6), Register::new(4)),
    ];

    for opcode in &opcodes {
        println!("{}", opcode.to_string());
    }
}
