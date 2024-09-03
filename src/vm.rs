#[derive(Clone, Debug, PartialEq)]
enum OpCode {
  Return,
  LoadConst(usize),
  Add,
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
  Integer(isize),
}

impl std::ops::Add for Value {
  type Output = Self;
  fn add(self, other: Self) -> Self::Output {
    use Value::*;
    match (self, other) {
      (Integer(x), Integer(y)) => Integer(x + y)
    }
  }
}

const MAX_STACK_SIZE: usize = 256;
#[derive(Clone, Debug, PartialEq)]
struct VM {
  stack: Vec<Value>,
  constants: Vec<Value>,
  code: Vec<OpCode>,
}

#[derive(Clone, Debug, PartialEq)]
enum VMError {
  ConstAccessError,
  StackEmpty,
  StackOverflow,
}

impl VM {
  fn new(constants: Vec<Value>, code: Vec<OpCode>) -> Self {
    Self {
      stack: Vec::new(),
      constants,
      code,
    }
  }

  fn push(&mut self, v: Value) -> Result<(), VMError> {
    if self.stack.len() > MAX_STACK_SIZE {
      Err(VMError::StackOverflow)
    } else {
      self.stack.push(v);
      Ok(())
    }
  }

  fn pop(&mut self) -> Result<Value, VMError> {
    self.stack.pop().ok_or(VMError::StackEmpty)
  }

  fn execute_all(&mut self) -> Result<(), VMError> {
    let code = self.code.clone().into_iter();
    for op in code {
      self.execute(op)?;
    }
    Ok(())
  }

  fn execute(&mut self, op: OpCode) -> Result<(), VMError> {
    use OpCode::*;
    match op {
      Return => Ok(()),
      LoadConst(i) => self.load_const(i),
      Add => self.add(),
    }
  }

  fn load_const(&mut self, index: usize) -> Result<(), VMError> {
    let constant = self.constants.get(index).ok_or(VMError::ConstAccessError)?.clone();
    self.push(constant)
  }

  fn add(&mut self) -> Result<(), VMError> {
    let x = self.pop()?;
    let y = self.pop()?;
    self.push(x + y)
  }
}

mod tests {
  #[test]
  fn test_execute_all() {
    use super::{OpCode::*, Value, VM};
    let constants = vec![Value::Integer(2), Value::Integer(2)];
    let code = vec![
      LoadConst(0),
      LoadConst(1),
      Add,
      Return
    ];
    let mut vm = VM::new(constants, code);
    vm.execute_all().unwrap();
    assert_eq!(vm.stack, vec![Value::Integer(4)]);
  }
}
