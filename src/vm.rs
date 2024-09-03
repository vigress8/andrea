use int_enum::IntEnum;

#[derive(Clone, Debug, PartialEq, IntEnum)]
#[repr(u8)]
enum OpCode {
  Return = 0,
  Load = 1,
  Load2 = 2,
  LoadConst = 3,
  IAdd = 4,
  ISub = 5,
  IMul = 6,
  IDiv = 7,
  INeg = 8,
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
  Integer(isize),
}

const MAX_STACK_SIZE: usize = 256;
#[derive(Clone, Debug)]
struct VM<'a> {
  stack: Vec<Value>,
  constants: Vec<Value>,
  code: &'a [u8],
  current: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum VMError {
  ConstIndexError,
  MissingOperand,
  StackEmpty,
  StackOverflow,
}

type VMResult<T> = Result<T, VMError>;
impl<'a> VM<'a> {
  fn new(constants: Vec<Value>, code: &'a [u8]) -> Self {
    Self {
      stack: Vec::new(),
      constants,
      code,
      current: 0,
    }
  }

  fn push(&mut self, v: Value) -> VMResult<()> {
    if self.stack.len() > MAX_STACK_SIZE {
      Err(VMError::StackOverflow)
    } else {
      self.stack.push(v);
      Ok(())
    }
  }

  fn pop(&mut self) -> VMResult<Value> {
    self.stack.pop().ok_or(VMError::StackEmpty)
  }

  fn advance(&mut self) -> VMResult<u8> {
    let &b = self.code.get(self.current).ok_or(VMError::MissingOperand)?;
    self.current += 1;
    Ok(b)
  }

  fn eof(&self) -> bool {
    self.current >= self.code.len()
  }

  fn execute_all(&mut self) -> VMResult<()> {
    while !self.eof() {
      self.execute()?;
    }
    Ok(())
  }

  fn execute(&mut self) -> VMResult<()> {
    use OpCode::*;
    let op = self.advance()?;
    if let Ok(op) = op.try_into() {
      match op {
        Return => Ok(()),
        Load => self.load(),
        Load2 => self.load2(),
        LoadConst => self.load_const(),
        IAdd => self.i_add(),
        ISub => self.i_sub(),
        IMul => self.i_mul(),
        IDiv => self.i_div(),
        INeg => self.i_neg(),
      }
    } else {
      unreachable!()
    }
  }

  fn load(&mut self) -> VMResult<()> {
    let b = self.advance()? as isize;
    self.push(Value::Integer(b))
  }

  fn load2(&mut self) -> VMResult<()> {
    let b1 = self.advance()? as isize;
    let b2 = self.advance()? as isize;

    // Combine the two bytes into a single integer.
    self.push(Value::Integer(b1 << 8 | b2))
  }

  fn load_const(&mut self) -> VMResult<()> {
    let index = self.advance()? as usize;
    let constant = self.constants.get(index).ok_or(VMError::ConstIndexError)?.clone();
    self.push(constant)
  }

  fn i_add(&mut self) -> VMResult<()> {
    let Value::Integer(x) = self.pop()?;
    let Value::Integer(y) = self.pop()?;
    self.push(Value::Integer(x + y))
  }

  fn i_sub(&mut self) -> VMResult<()> {
    let Value::Integer(x) = self.pop()?;
    let Value::Integer(y) = self.pop()?;
    self.push(Value::Integer(x - y))
  }

  fn i_mul(&mut self) -> VMResult<()> {
    let Value::Integer(x) = self.pop()?;
    let Value::Integer(y) = self.pop()?;
    self.push(Value::Integer(x * y))
  }

  fn i_div(&mut self) -> VMResult<()> {
    let Value::Integer(x) = self.pop()?;
    let Value::Integer(y) = self.pop()?;
    self.push(Value::Integer(x / y))
  }

  fn i_neg(&mut self) -> VMResult<()> {
    let Value::Integer(x) = self.pop()?;
    self.push(Value::Integer(-x))
  }
}

mod tests {
  use super::*;
  use OpCode::*;

  #[test]
  fn test_execute_all() {
    let constants = vec![Value::Integer(2500), Value::Integer(378)];
    let code = &[
      LoadConst as u8,
      0,
      LoadConst as u8,
      1,
      IAdd as u8,
      Return as u8
    ];
    let mut vm = VM::new(constants, code);
    vm.execute_all().unwrap();
    assert_eq!(vm.stack, vec![Value::Integer(2878)]);


    let code = &[
      Load as u8,
      5,
      Load2 as u8,
      3,
      4,
      IMul as u8,
      Return as u8
    ];
    let mut vm = VM::new(vec![], code);
    vm.execute_all().unwrap();
    assert_eq!(vm.stack, vec![Value::Integer(3860)]);
  }
}
