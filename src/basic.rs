use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
  String(String),
  Integer(i64),
  Float(f64),
  Array(Vec<Value>),
  Nil,
}

impl Value {
  pub fn to_string(&self, delimiters: bool) -> String {
    match self {
      Value::String(string) => string.clone(),
      Value::Integer(num) => num.to_string(),
      Value::Float(num) => num.to_string(),
      Value::Array(array) => {
        let mut string = String::new();
        if delimiters {
          string.push_str("{");
        }

        for (idx, value) in array.iter().enumerate() {
          string.push_str(&value.to_string(delimiters));
          if idx != array.len() - 1 && delimiters {
            string.push_str(", ");
          }
        }

        if delimiters {
          string.push_str("}");
        }
        string
      }
      Value::Nil => "nil".to_string(),
    }
  }

  pub fn is_truthy(&self) -> bool {
    match self {
      Value::String(string) => !string.is_empty(),
      Value::Integer(num) => *num != 0,
      Value::Float(num) => *num != 0.0,
      Value::Array(array) => !array.is_empty(),
      Value::Nil => false,
    }
  }

  pub fn comparison_value(&self) -> Result<f64, String> {
    match self {
      Value::String(string) => Err(format!("Cannot compare string {:?}!", string)),
      Value::Integer(num) => Ok(*num as f64),
      Value::Float(num) => Ok(*num),
      Value::Array(array) => Err(format!("Cannot compare array {:?}!", array)),
      Value::Nil => Ok(0.0),
    }
  }

  pub fn to_integer(&self) -> Result<i64, String> {
    match self {
      Value::String(string) => Ok(string.parse::<i64>().unwrap()),
      Value::Integer(num) => Ok(*num),
      Value::Float(num) => Ok(*num as i64),
      Value::Nil => Ok(0),
      Value::Array(array) => Err(format!("Cannot convert array {:?} to integer!", array)),
    }
  }

  pub fn to_float(&self) -> Result<f64, String> {
    match self {
      Value::String(string) => Ok(string.parse::<f64>().unwrap()),
      Value::Integer(num) => Ok(*num as f64),
      Value::Float(num) => Ok(*num),
      Value::Nil => Ok(0.0),
      Value::Array(array) => Err(format!("Cannot convert array {:?} to float!", array)),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
  For {
    name: String,
    start: Box<Node>,
    end: Box<Node>,
    step: Box<Node>,
  },
  // for name = start to end step step
  If {
    cond: Box<Node>,
    then: Box<Node>,
    else_: Box<Node>,
  },
  // if cond then then else else
  Assign {
    name: String,
    value: Box<Node>,
  },
  // name = value
  BinOp {
    op: Token,
    left: Box<Node>,
    right: Box<Node>,
  },
  // left op right
  UnOp {
    op: Token,
    right: Box<Node>,
  },
  // op right
  BuiltinCommand {
    name: String,
    args: Vec<Node>,
  },
  // name arg1 arg2 ... argN
  Integer(i64),
  // integer
  Float(f64),
  // float
  String(String),
  // string
  VarGet(String),
  // var
  Array(Vec<Node>),
  // {node1, node2, ... nodeN}
  EmptyArray(Box<Node>),
  // [size]
  IndexGet {
    name: String,
    index: Box<Node>,
  },
  // name[index]
  IndexSet {
    name: String,
    index: Box<Node>,
    value: Box<Node>,
  },
  // name[index] = value
  End,
  // end
  Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Integer(i64),
  Float(f64),
  String(String),
  LParen,
  RParen,
  LSquare,
  RSquare,
  LCurly,
  RCurly,
  Eq,
  EqEq,
  Neq,
  Lt,
  Gt,
  Lte,
  Gte,
  Add,
  Sub,
  Mul,
  Div,
  Percent,
  Comma,
  Identifier(String),
  For,
  Next,
  To,
  Step,
  If,
  Then,
  Else,
  Goto,
  Gosub,
  Ret,
  End,
  Pipe,
  Ampersand,
  Exclamation,
}

#[derive(Debug, Clone)]
pub struct Line {
  pub line_no: usize,
  pub node: Node,
  pub contents: String,
}

impl PartialEq<Line> for Line {
  fn eq(&self, other: &Line) -> bool {
    self.line_no == other.line_no
  }
}

impl PartialOrd<Line> for Line {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for Line {}

impl Ord for Line {
  fn cmp(&self, other: &Self) -> Ordering {
    self.line_no.cmp(&other.line_no)
  }
}

static INVALID_LINE_NO: usize = 0;

pub struct ParseOptions {
  pub builtin_commands: Vec<&'static str>,
}

pub struct BASIC {
  pub program: Vec<Line>,
  pub vars: HashMap<String, Value>,
  pub line_no: usize,
  pub call_stack: Vec<usize>,
  pub while_stack: Vec<usize>,
  pub for_stack: Vec<(usize, Value, Value)>,
  pub symbols: HashMap<u8, Token>,
  pub keywords: HashMap<&'static str, Token>,
  pub options: ParseOptions,
  pub no_increment_instr_counter: bool,
  pub refresh: bool,
}

impl BASIC {
  pub(crate) fn new(
    symbols: HashMap<u8, Token>,
    keywords: HashMap<&'static str, Token>,
    options: ParseOptions,
  ) -> BASIC {
    BASIC {
      program: Vec::<Line>::new(),
      vars: HashMap::<String, Value>::new(),
      line_no: 0,
      call_stack: Vec::<usize>::new(),
      while_stack: Vec::<usize>::new(),
      for_stack: Vec::<(usize, Value, Value)>::new(),
      symbols,
      keywords,
      options,
      no_increment_instr_counter: false,
      refresh: false,
    }
  }

  pub fn add_line(&mut self, src: String) -> Result<Option<Node>, String> {
    let tokens = self.lex_line(&src)?;
    println!("tokens: {:?}\n", tokens);
    let line = self.parse_line(&tokens, src)?;
    println!("line: {:?}\n", line);

    if line.line_no == 0 {
      return Ok(Some(line.node));
    }

    if line.node == Node::Nil {
      self.remove_line(line.line_no);
      return Ok(None);
    }

    if let Some(idx) = self.program.iter().position(|x| x.line_no == line.line_no) {
      self.program[idx] = line;
    } else {
      self.program.push(line);
      self.program.sort();
    }

    Ok(None)
  }

  fn remove_line(&mut self, line_no: usize) {
    self.program.retain(|x| x.line_no != line_no);
  }

  pub fn reset_program_state(&mut self) {
    self.vars.clear();
    self.call_stack.clear();
    self.while_stack.clear();
    self.for_stack.clear();
    self.line_no = 0;
  }

  pub fn parse_line(&self, tokens: &Vec<Token>, original: String) -> Result<Line, String> {
    if tokens.is_empty() {
      return Err("Empty line!".to_string());
    }

    let mut line_no = INVALID_LINE_NO;
    let begin_idx = match tokens[0] {
      Token::Integer(num) => {
        line_no = num as usize;
        1
      }
      _ => 0,
    };

    if begin_idx == tokens.len() {
      return Ok(Line {
        line_no,
        node: Node::Nil,
        contents: original,
      });
    }

    let (ending_idx, node) = self.stmt(begin_idx, tokens)?;

    if ending_idx != tokens.len() {
      return Err(format!(
        "Expected end of line, got {:?} on line {}",
        tokens[ending_idx], line_no
      ));
    }

    Ok(Line {
      line_no,
      node,
      contents: original,
    })
  }

  pub fn stmt(&self, mut idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    match tokens.get(idx) {
      Some(Token::Identifier(name)) => {
        match name.as_str() {
          "for" => {
            idx += 1;
            let name = match tokens[idx] {
              Token::Identifier(ref name) => {
                let n = name.clone();
                idx += 1;
                n
              }
              _ => return Err(format!("Expected identifier, got {:?}", tokens[idx])),
            };

            if tokens[idx] != Token::Eq {
              return Err(format!("Expected '=', got {:?}", tokens[idx]));
            }
            idx += 1;

            let (new_idx, start) = self.expr(idx, tokens)?;
            idx = new_idx;

            if tokens[idx] != Token::To {
              return Err(format!("Expected 'to', got {:?}", tokens[idx]));
            }
            idx += 1;

            let (new_idx, end) = self.expr(idx, tokens)?;
            idx = new_idx;

            let step = match tokens.get(idx) {
              Some(Token::Step) => {
                idx += 1;
                let (new_idx, step) = self.expr(idx, tokens)?;
                idx = new_idx;
                step
              }
              _ => Node::Integer(1),
            };

            return Ok((
              idx,
              Node::For {
                name,
                start: Box::new(start),
                end: Box::new(end),
                step: Box::new(step),
              },
            ));
          }
          _ => {}
        }

        if (&self.options.builtin_commands).contains(&name.as_str()) {
          let mut args = Vec::<Node>::new();
          idx += 1;

          while idx < tokens.len() {
            let (new_idx, arg) = self.expr(idx, tokens)?;
            args.push(arg);
            idx = new_idx;
          }

          return Ok((
            idx,
            Node::BuiltinCommand {
              name: (*name).clone(),
              args,
            },
          ));
        }

        self.expr(idx, tokens)
      }
      _ => self.expr(idx, tokens),
    }
  }

  pub fn expr(&self, idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    self.or(idx, tokens)
  }

  pub fn or(&self, idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    self.bin_op(idx, tokens, Self::and, Self::and, vec![Token::Pipe])
  }

  pub fn and(&self, idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    self.bin_op(idx, tokens, Self::cmp, Self::cmp, vec![Token::Ampersand])
  }

  pub fn cmp(&self, mut idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    if idx >= tokens.len() {
      return Err("Expected expression, got end of line!".to_string());
    }

    match tokens[idx] {
      Token::Exclamation => {
        idx += 1;
        let (new_idx, right) = self.cmp(idx, tokens)?;
        idx = new_idx;
        Ok((
          idx,
          Node::UnOp {
            op: Token::Exclamation,
            right: Box::new(right),
          },
        ))
      }
      _ => Ok(self.bin_op(
        idx,
        tokens,
        Self::add,
        Self::add,
        vec![
          Token::Lt,
          Token::Gt,
          Token::Lte,
          Token::Gte,
          Token::EqEq,
          Token::Neq,
        ],
      )?),
    }
  }

  pub fn add(&self, idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    self.bin_op(
      idx,
      tokens,
      Self::mul,
      Self::mul,
      vec![Token::Add, Token::Sub],
    )
  }

  pub fn mul(&self, mut idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    match tokens[idx] {
      Token::Add | Token::Sub => {
        let op = tokens[idx].clone();
        idx += 1;
        let (new_idx, right) = self.mul(idx, tokens)?;
        idx = new_idx;
        Ok((
          idx,
          Node::UnOp {
            op,
            right: Box::new(right),
          },
        ))
      }
      _ => self.bin_op(
        idx,
        tokens,
        Self::atom,
        Self::atom,
        vec![Token::Mul, Token::Div, Token::Percent],
      ),
    }
  }

  pub fn atom(&self, mut idx: usize, tokens: &Vec<Token>) -> Result<(usize, Node), String> {
    match &tokens[idx] {
      Token::Integer(num) => {
        idx += 1;
        Ok((idx, Node::Integer(num.clone())))
      }
      Token::Float(num) => {
        idx += 1;
        Ok((idx, Node::Float(num.clone())))
      }
      Token::String(string) => {
        idx += 1;
        Ok((idx, Node::String(string.clone())))
      }
      Token::Identifier(name) => {
        idx += 1;
        if idx < tokens.len()
          && tokens[idx] == Token::LParen
          && (&self.options.builtin_commands).contains(&name.as_str())
        {
          idx += 1;
          let mut args = Vec::<Node>::new();
          while idx < tokens.len() && tokens[idx] != Token::RParen {
            let (new_idx, arg) = self.expr(idx, tokens)?;
            args.push(arg);
            idx = new_idx;
            if idx < tokens.len() && tokens[idx] == Token::Comma {
              idx += 1;
            }
          }
          idx += 1;
          return Ok((
            idx,
            Node::BuiltinCommand {
              name: (*name).clone(),
              args,
            },
          ));
        }

        if idx < tokens.len() && tokens[idx] == Token::LSquare {
          idx += 1;
          let (new_idx, index) = self.expr(idx, tokens)?;
          idx = new_idx;
          if tokens[idx] != Token::RSquare {
            return Err(format!("Expected ']', got {:?}", tokens[idx]));
          }
          idx += 1;
          if idx < tokens.len() && tokens[idx] == Token::Eq {
            idx += 1;
            let (new_idx, value) = self.expr(idx, tokens)?;
            idx = new_idx;
            return Ok((
              idx,
              Node::IndexSet {
                name: (*name).clone(),
                index: Box::new(index),
                value: Box::new(value),
              },
            ));
          }
          return Ok((
            idx,
            Node::IndexGet {
              name: (*name).clone(),
              index: Box::new(index),
            },
          ));
        }

        if idx < tokens.len() && tokens[idx] == Token::Eq {
          idx += 1;
          let (new_idx, value) = self.expr(idx, tokens)?;
          idx = new_idx;
          return Ok((
            idx,
            Node::Assign {
              name: (*name).clone(),
              value: Box::new(value),
            },
          ));
        }

        Ok((idx, Node::VarGet(name.clone())))
      }
      Token::LSquare => {
        // array initialized with nils
        idx += 1;
        let (new_idx, size) = self.expr(idx, tokens)?;
        idx = new_idx;
        if tokens[idx] != Token::RSquare {
          return Err(format!("Expected ']', got {:?}", tokens[idx]));
        }
        idx += 1;
        Ok((idx, Node::EmptyArray(Box::new(size))))
      }
      Token::LCurly => {
        // array
        idx += 1;
        let mut array = Vec::<Node>::new();
        while idx < tokens.len() && tokens[idx] != Token::RCurly {
          let (new_idx, node) = self.expr(idx, tokens)?;
          idx = new_idx;
          array.push(node);
          if idx < tokens.len() && tokens[idx] == Token::Comma {
            idx += 1;
          }
        }
        idx += 1;
        Ok((idx, Node::Array(array)))
      }
      Token::LParen => {
        idx += 1;
        let (new_idx, node) = self.expr(idx, tokens)?;
        idx = new_idx;
        if tokens[idx] != Token::RParen {
          return Err(format!("Expected ')', got {:?}", tokens[idx]));
        }
        idx += 1;
        Ok((idx, node))
      }
      _ => Err(format!("Expected atom, got {:?}", tokens[idx])),
    }
  }

  pub fn bin_op(
    &self,
    mut idx: usize,
    tokens: &Vec<Token>,
    lhs: fn(&BASIC, usize, &Vec<Token>) -> Result<(usize, Node), String>,
    rhs: fn(&BASIC, usize, &Vec<Token>) -> Result<(usize, Node), String>,
    ops: Vec<Token>,
  ) -> Result<(usize, Node), String> {
    let (new_idx, mut left) = lhs(self, idx, tokens)?;
    idx = new_idx;

    while idx < tokens.len() && ops.contains(&tokens[idx]) {
      let op = tokens[idx].clone();
      idx += 1;
      let (new_idx, right) = rhs(self, idx, tokens)?;
      idx = new_idx;
      left = Node::BinOp {
        op,
        left: Box::new(left),
        right: Box::new(right),
      };
    }

    Ok((idx, left))
  }

  pub fn lex_line(&self, str: &str) -> Result<Vec<Token>, String> {
    let str = str.as_bytes();
    let mut idx = 0;
    let mut tokens = Vec::<Token>::new();

    while idx < str.len() {
      match str[idx] {
        b'0'..=b'9' => {
          let mut num = 0;
          while idx < str.len() && str[idx] >= b'0' && str[idx] <= b'9' {
            num = num * 10 + (str[idx] - b'0') as i64;
            idx += 1;
          }

          if idx == str.len() {
            tokens.push(Token::Integer(num));
            break;
          }

          if str[idx] == b'.' {
            idx += 1;
            let mut dec = 0.0;
            let mut div = 1.0;
            while idx < str.len() && str[idx] >= b'0' && str[idx] <= b'9' {
              dec = dec * 10.0 + (str[idx] - b'0') as f64;
              div *= 10.0;
              idx += 1;
            }
            tokens.push(Token::Float(num as f64 + dec / div));
          } else {
            tokens.push(Token::Integer(num));
          }
        }
        b'"' => {
          idx += 1;
          let mut string = String::new();
          while idx < str.len() && str[idx] != b'"' {
            string.push(str[idx] as char);
            idx += 1;
          }
          idx += 1;
          tokens.push(Token::String(string));
        }
        b'<' => {
          idx += 1;
          if idx < str.len() && str[idx] == b'>' {
            tokens.push(Token::Neq);
            idx += 1;
          } else if idx < str.len() && str[idx] == b'=' {
            tokens.push(Token::Lte);
            idx += 1;
          } else {
            tokens.push(Token::Lt);
          }
        }
        b'>' => {
          idx += 1;
          if idx < str.len() && str[idx] == b'=' {
            tokens.push(Token::Gte);
            idx += 1;
          } else {
            tokens.push(Token::Gt);
          }
        }
        b'=' => {
          idx += 1;
          if idx < str.len() && str[idx] == b'=' {
            tokens.push(Token::EqEq);
            idx += 1;
          } else {
            tokens.push(Token::Eq);
          }
        }
        b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
          let mut var = String::new();
          while idx < str.len()
            && (str[idx] >= b'a' && str[idx] <= b'z'
            || str[idx] >= b'A' && str[idx] <= b'Z'
            || str[idx] == b'_'
            || str[idx] >= b'0' && str[idx] <= b'9')
          {
            var.push(str[idx] as char);
            idx += 1;
          }

          if idx < str.len() && (str[idx] == b'$' || str[idx] == b'%') {
            var.push(str[idx] as char);
            idx += 1;
          }

          if let Some(tok) = self.keywords.get(&var.as_str()) {
            tokens.push((*tok).clone());
          } else {
            tokens.push(Token::Identifier(var));
          }
        }
        b'\t' | b' ' | b'\n' | b'\r' => {
          idx += 1;
        }
        _ => {
          if let Some(tok) = self.symbols.get(&str[idx]) {
            tokens.push((*tok).clone());
            idx += 1;
          } else {
            return Err(format!("Unknown token: {}", str[idx] as char));
          }
        }
      }
    }

    Ok(tokens)
  }
}
