use crate::heap::Heap;
use crate::opcode::OpCode;
use crate::value::Value;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    locals: Vec<Value>,
    heap: Heap,
}

pub type Chunk = Vec<u8>;

impl VM {
    pub fn push(&mut self, val: Value) {
        self.stack.push(val)
    } 
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn get_bool(&mut self) -> bool {
        let Value::Word(w) = self.pop() else {unreachable!()};
        w != 0
    }

    pub fn get_integer(&mut self) -> i64 {
        let Value::Integer(i) = self.pop() else {unreachable!()};
        i
    }

    pub fn get_word(&mut self) -> u64 {
        let Value::Word(w) = self.pop() else {unreachable!()};
        w
    }

    pub fn get_float(&mut self) -> f64 {
        let Value::Float(f) = self.pop() else {unreachable!()};
        f
    }
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ..Default::default()
        }
    }

    pub fn mark_objects(&self) {
        for val in &self.stack {
            if let Some(ptr) = val.get_object_ptr() {
                ptr.mark();
            }
        }
    }

    pub fn eof(&self) -> bool {
        self.ip >= self.chunk.len()
    }

    pub fn advance(&mut self) -> u8 {
        let b = self.chunk[self.ip];
        eprintln!("ip: {}", self.ip);
        self.ip += 1;
        b
    }

    pub fn advance2(&mut self) -> u16 {
        u16::from_be_bytes([self.advance(), self.advance()])
    }

    pub fn advance4(&mut self) -> u32 {
        u32::from_be_bytes([
            self.advance(),
            self.advance(),
            self.advance(),
            self.advance(),
        ])
    }

    pub fn advance8(&mut self) -> u64 {
        u64::from_be_bytes([
            self.advance(),
            self.advance(),
            self.advance(),
            self.advance(),
            self.advance(),
            self.advance(),
            self.advance(),
            self.advance(),
        ])
    }

    pub fn execute_all(&mut self) {
        while !self.eof() {
            self.execute();
        }
    }

    pub fn execute(&mut self) {
        use OpCode::*;
        let op = self.advance().try_into().unwrap();
        match op {
            Return => self.ret(),
            Goto => self.goto(),
            GotoIf => self.goto_if(),
            Load => self.load(),
            Store => self.store(),
            ImmI => self.imm_i(),
            ImmF => self.imm_f(),
            ImmW => self.imm_w(),
            AddI => self.add_i(),
            SubI => self.sub_i(),
            MulI => self.mul_i(),
            DivI => self.div_i(),
            CmpEqI => self.cmpeq_i(),
            CmpGtI => self.cmpgt_i(),
            CmpGeI => self.cmpge_i(),
            CmpLtI => self.cmplt_i(),
            CmpLeI => self.cmple_i(),
        }
    }

    fn ret(&mut self) {
        self.ip = self.chunk.len();
    }
    
    fn goto(&mut self) {
        let index = self.advance2() as usize;
        self.ip = index;
    }

    fn goto_if(&mut self) {
        let p = self.get_bool();
        let index = self.advance2() as usize;
        if p {
                self.ip = index;
        }
    }

    fn load(&mut self) {
        let index = self.advance2() as usize;
        let variable = self.locals[index];
        eprintln!("Loading {variable:?} from index {index}");
        self.push(variable)
    }

    fn store(&mut self) {
        let index = self.advance2() as usize;
        let value = self.pop();
        eprintln!("Storing {value:?} at index {index}");
        if self.locals.get(index).is_some() {
            self.locals[index] = value;
        } else {
            self.locals.push(value);
        }
    }

    fn imm_i(&mut self) {
        let i = self.advance8() as i64;
        self.stack.push(Value::Integer(i))
    }

    fn imm_f(&mut self) {
        let f = self.advance8() as f64;
        self.stack.push(Value::Float(f))
    }

    fn imm_w(&mut self) {
        let w = self.advance8();
        self.stack.push(Value::Word(w))
    }

    fn add_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        self.push(Value::Integer(x + y))
    }

    fn sub_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        eprintln!("Subtracting {x} - {y}");
        self.push(Value::Integer(x - y))
    }

    fn mul_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        eprintln!("Multiplying {x} * {y}");
        self.push(Value::Integer(x * y))
    }

    fn div_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        self.push(Value::Integer(x / y))
    }

    fn cmpeq_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        self.push(Value::Word((x == y) as u64))
    }

    fn cmpgt_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        eprintln!("Testing {x} > {y}");
        self.push(Value::Word((x > y) as u64))
    }

    fn cmpge_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        self.push(Value::Word((x >= y) as u64))
    }

    fn cmplt_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        self.push(Value::Word((x < y) as u64))
    }

    fn cmple_i(&mut self) {
        let x = self.get_integer();
        let y = self.get_integer();
        self.push(Value::Word((x <= y) as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::OpCode::*;

    #[test]
    fn test_factorial() {
        #[rustfmt::skip]
        let factorial = vec![
            ImmI  as u8, 0, 0, 0, 0, 0, 0, 0, 5,
            Store as u8, 0, 0,

            // x = 1
            ImmI  as u8, 0, 0, 0, 0, 0, 0, 0, 1,
            Store as u8, 0, 1,

            // while n > 1 {
            Load   as u8, 0, 0,
            ImmI   as u8, 0, 0, 0, 0, 0, 0, 0, 1,
            CmpGtI as u8,
            GotoIf as u8, 0, 69,

            // x = x * n
            Load  as u8, 0, 1,
            Load  as u8, 0, 0,
            MulI  as u8,
            Store as u8, 0, 1,

            // n = n - 1
            ImmI  as u8, 0, 0, 0, 0, 0, 0, 0, 1,
            Load  as u8, 0, 0,
            SubI  as u8,
            Store as u8, 0, 0,

            // }
            Goto  as u8, 0, 24,

            // return x
            Load   as u8, 0, 1,
            Return as u8,
        ];

        let mut vm = VM {
            chunk: factorial,
            ..Default::default()
        };
        vm.execute_all();
        assert_eq!(vm.stack, vec![Value::Integer(120)]);
    }
}