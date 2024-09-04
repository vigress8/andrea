use int_enum::IntEnum;

#[derive(Clone, Debug, PartialEq, IntEnum)]
#[repr(u8)]
enum OpCode {
    Return = 0,
    Goto = 1,
    GotoIf = 2,
    Load = 3,
    Load2 = 4,
    LoadConst = 5,
    IAdd = 6,
    ISub = 7,
    IMul = 8,
    IDiv = 9,
    INeg = 10,
    ICmpEQ = 11,
    ICmpGT = 12,
    ICmpGE = 13,
    ICmpLT = 14,
    ICmpLE = 15,
    LoadVar = 16,
    StoreVar = 17,
    Call = 18,
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Integer(isize),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq)]
enum VMError {
    ConstIndexError,
    IllegalGoto,
    StackEmpty,
    StackOverflow,
    TypeError,
    UnboundVariable,
    UnexpectedEOF,
    UnknownOpCode,
}

fn combine_bytes(b1: u8, b2: u8) -> usize {
    (b1 as usize) << 8 | b2 as usize
}

const MAX_STACK_SIZE: usize = 256;
#[derive(Clone, Debug)]
struct VM<'a> {
    stack: Vec<Value>,
    variables: Vec<Value>,
    constants: Vec<Value>,
    chunk: &'a [u8],
    current: usize,
}

impl<'a> VM<'a> {
    fn new(variables: Vec<Value>, constants: Vec<Value>, chunk: &'a [u8]) -> Self {
        Self {
            stack: Vec::new(),
            variables,
            constants,
            chunk,
            current: 0,
        }
    }
}

type VMResult<T> = Result<T, VMError>;
impl VM<'_> {
    fn push(&mut self, v: Value) -> VMResult<()> {
        if self.stack.len() >= MAX_STACK_SIZE {
            Err(VMError::StackOverflow)
        } else {
            self.stack.push(v);
            Ok(())
        }
    }

    fn pop(&mut self) -> VMResult<Value> {
        self.stack.pop().ok_or(VMError::StackEmpty)
    }

    fn peek(&mut self) -> VMResult<u8> {
        self.chunk
            .get(self.current)
            .copied()
            .ok_or(VMError::UnexpectedEOF)
    }

    fn advance(&mut self) -> VMResult<u8> {
        let &b = self.chunk.get(self.current).ok_or(VMError::UnexpectedEOF)?;
        self.current += 1;
        Ok(b)
    }

    fn eof(&self) -> bool {
        self.current >= self.chunk.len()
    }

    fn expect_integer(&mut self) -> VMResult<isize> {
        if let Value::Integer(i) = self.pop()? {
            Ok(i)
        } else {
            Err(VMError::TypeError)
        }
    }

    fn expect_bool(&mut self) -> VMResult<bool> {
        if let Value::Bool(p) = self.pop()? {
            Ok(p)
        } else {
            Err(VMError::TypeError)
        }
    }

    fn execute_all(&mut self) -> VMResult<()> {
        while !self.eof() {
            self.execute()?;
        }
        Ok(())
    }

    fn execute(&mut self) -> VMResult<()> {
        use OpCode::*;
        let Ok(op) = self.advance()?.try_into() else {
            Err(VMError::UnknownOpCode)?
        };
        match op {
            Return => self.ret(),
            Goto => self.goto(),
            GotoIf => self.goto_if(),
            Load => self.load(),
            Load2 => self.load2(),
            LoadConst => self.load_const(),
            IAdd => self.i_add(),
            ISub => self.i_sub(),
            IMul => self.i_mul(),
            IDiv => self.i_div(),
            INeg => self.i_neg(),
            ICmpEQ => self.i_cmpeq(),
            ICmpGT => self.i_cmpgt(),
            ICmpGE => self.i_cmpge(),
            ICmpLT => self.i_cmplt(),
            ICmpLE => self.i_cmple(),
            LoadVar => self.load_var(),
            StoreVar => self.store_var(),
            Call => self.call(),
        }
    }

    fn ret(&mut self) -> Result<(), VMError> {
        self.current = self.chunk.len();
        Ok(())
    }

    fn goto(&mut self) -> VMResult<()> {
        let b1 = self.advance()?;
        let b2 = self.advance()?;
        let index = combine_bytes(b1, b2);
        if index >= self.chunk.len() {
            Err(VMError::IllegalGoto)
        } else {
            self.current = index;
            Ok(())
        }
    }

    fn goto_if(&mut self) -> VMResult<()> {
        let p = self.expect_bool()?;
        let b1 = self.advance()?;
        let b2 = self.advance()?;
        let index = combine_bytes(b1, b2);
        if p {
            if index >= self.chunk.len() {
                Err(VMError::IllegalGoto)
            } else {
                self.current = index;
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn load(&mut self) -> VMResult<()> {
        let b = self.advance()? as isize;
        self.push(Value::Integer(b))
    }

    fn load2(&mut self) -> VMResult<()> {
        let b1 = self.advance()?;
        let b2 = self.advance()?;
        self.push(Value::Integer(combine_bytes(b1, b2) as isize))
    }

    fn load_const(&mut self) -> VMResult<()> {
        let index = self.advance()? as usize;
        let constant = self
            .constants
            .get(index)
            .ok_or(VMError::ConstIndexError)?
            .clone();
        self.push(constant)
    }

    fn load_var(&mut self) -> VMResult<()> {
        let b1 = self.advance()?;
        let b2 = self.advance()?;
        let index = combine_bytes(b1, b2);
        let variable = self
            .variables
            .get(index)
            .ok_or(VMError::UnboundVariable)?
            .clone();
        self.push(variable)
    }

    fn store_var(&mut self) -> VMResult<()> {
        let b1 = self.advance()?;
        let b2 = self.advance()?;
        let index = combine_bytes(b1, b2);
        let value = self.pop()?;
        if self.variables.get(index).is_some() {
            self.variables[index] = value;
        } else {
            self.variables.push(value);
        }
        Ok(())
    }

    fn call(&mut self) -> VMResult<()> {
        todo!()
    }

    fn i_add(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Integer(x + y))
    }

    fn i_sub(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Integer(x - y))
    }

    fn i_mul(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Integer(x * y))
    }

    fn i_div(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Integer(x / y))
    }

    fn i_neg(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        self.push(Value::Integer(-x))
    }

    fn i_cmpeq(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Bool(x == y))
    }

    fn i_cmpgt(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Bool(x > y))
    }

    fn i_cmpge(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Bool(x >= y))
    }

    fn i_cmplt(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Bool(x < y))
    }

    fn i_cmple(&mut self) -> VMResult<()> {
        let x = self.expect_integer()?;
        let y = self.expect_integer()?;
        self.push(Value::Bool(x <= y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use OpCode::*;

    #[test]
    fn test_execute_all() {
        let constants = vec![Value::Integer(2500), Value::Integer(378)];
        let chunk = &[
            LoadConst as u8,
            0,
            LoadConst as u8,
            1,
            IAdd as u8,
            Return as u8,
        ];
        let mut vm = VM::new(vec![], constants, chunk);
        vm.execute_all().unwrap();
        assert_eq!(vm.stack, vec![Value::Integer(2878)]);

        let chunk = &[Load as u8, 5, Load2 as u8, 3, 4, IMul as u8, Return as u8];
        let mut vm = VM::new(vec![], vec![], chunk);
        vm.execute_all().unwrap();
        assert_eq!(vm.stack, vec![Value::Integer(3860)]);
    }

    #[test]
    fn test_goto() {
        let chunk = &[Load as u8, 1, Goto as u8, 0, 0];

        let mut vm = VM::new(vec![], vec![], chunk);
        assert_eq!(vm.execute_all(), Err(VMError::StackOverflow));
        assert_eq!(vm.stack, vec![Value::Integer(1); MAX_STACK_SIZE]);
    }

    #[test]
    fn test_factorial() {
        // n = 5
        let variables = vec![Value::Integer(5)];

        #[rustfmt::skip]
        let chunk = &[
            // x = 1
            Load     as u8,    1,
            StoreVar as u8, 0, 1,

            // while not n <= 1
            Load     as u8,    1,
            LoadVar  as u8, 0, 0,
            ICmpLE   as u8,
            GotoIf   as u8, 0, 36,

            // x = x * n
            LoadVar  as u8, 0, 1,
            LoadVar  as u8, 0, 0,
            IMul     as u8,
            StoreVar as u8, 0, 1,

            // n = n - 1
            Load     as u8,    1,
            LoadVar  as u8, 0, 0,
            ISub     as u8,
            StoreVar as u8, 0, 0,

            // Jump to loop start
            Goto     as u8, 0, 5,

            // return x
            LoadVar  as u8, 0, 1,
            Return   as u8,
        ];

        let mut vm = VM::new(variables, vec![], chunk);
        vm.execute_all().unwrap();
        assert_eq!(vm.stack, vec![Value::Integer(120)]);
    }
}
