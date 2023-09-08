use std::fs::File;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;
use std::io::{Read, Write};
use crate::lex_parse_basic::{Node, Token, Value};
use crate::koneko::{Koneko, secs_since_start};
use crate::palette::Sweetie16;

impl Koneko {
  pub fn vec2i_from_value(value: &Value) -> Result<(i32, i32), String> {
    Ok(match value {
      Value::Array(array) => {
        if array.len() != 2 {
          return Err(format!("Expected array of length 2, got {:?}", array));
        }

        let x = array[0].to_integer()? as i32;
        let y = array[1].to_integer()? as i32;

        (x, y)
      }
      _ => return Err(format!("Expected array, got {:?}", value))
    })
  }

  pub fn palette_idx_from_value(value: &Value) -> Result<u8, String> {
    Ok(match value {
      Value::Integer(num) => *num as u8,
      Value::String(str) => {
        match str.as_str() {
          "orange" | "org" => Sweetie16::Orange.into(),
          "red" => Sweetie16::Red.into(),
          "yellow" | "yel" => Sweetie16::Yellow.into(),
          "green" | "grn" => Sweetie16::DarkGreen.into(),
          "blue" | "blu" => Sweetie16::DarkBlue.into(),
          "light_blue" => Sweetie16::LightBlue.into(),
          "deep_blue" => Sweetie16::DeepBlue.into(),
          "light_green" => Sweetie16::LightGreen.into(),
          "pink" => Sweetie16::Pink.into(),
          "aqua" => Sweetie16::Aqua.into(),
          "dark_gray" => Sweetie16::DarkGray.into(),
          "medium_gray" => Sweetie16::MediumGray.into(),
          "light_gray" => Sweetie16::LightGray.into(),
          "purple" | "pur" => Sweetie16::Purple.into(),
          "black" | "blk" => Sweetie16::Black.into(),
          "white" | "wht" => Sweetie16::White.into(),
          _ => return Err(format!("Unknown color {}", str))
        }
      }
      _ => return Err(format!("Expected integer, got {:?}", value))
    })
  }

  fn expect_n_args(args: &[Node], n: usize) -> Result<(), String> {
    if args.len() != n {
      return Err(format!("Expected {} arguments, got {}", n, args.len()));
    }
    Ok(())
  }

  pub fn interpret(&mut self, node: Node) -> Result<Value, String> {
    match node {
      Node::Integer(num) => Ok(Value::Integer(num)),
      Node::Float(num) => Ok(Value::Float(num)),
      Node::String(string) => Ok(Value::String(string.clone())),
      Node::VarGet(name) => {
        if let Some(value) = self.basic.vars.get(name.as_str()) {
          Ok(value.clone())
        } else {
          Err(format!("Variable {} not found!", name))
        }
      }
      Node::Assign { name, value } => {
        let value = self.interpret(*value)?;
        self.basic.vars.insert(name.clone(), value.clone());
        Ok(value)
      }
      Node::For { name, start, end, step } => {
        if let Some(_value) = self.basic.vars.get(name.as_str()) {
          return Err(format!("Variable {} already exists!", name));
        }

        let start = self.interpret(*start)?;
        let end = self.interpret(*end)?;
        let step = self.interpret(*step)?;

        self.basic.vars.insert(name.clone(), start.clone());
        self.basic.for_stack.push((self.basic.line_no, end, step));

        Ok(Value::Nil)
      }
      Node::If { cond, then, else_ } => {
        let cond = self.interpret(*cond)?;
        if cond.is_truthy() {
          self.interpret(*then)
        } else {
          self.interpret(*else_)
        }
      }
      Node::BinOp { op, left, right } => {
        let left = self.interpret(*left)?;
        let right = self.interpret(*right)?;

        match op {
          Token::Add => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left + right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left + right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 + right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left + *right as f64)),
              (Value::String(left), Value::String(right)) =>
                Ok(Value::String(left.clone() + right.clone().as_str())),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Sub => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left - right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left - right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 - right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left - *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Percent => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left % right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left % right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 % right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left % *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Mul => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left * right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left * right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 * right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left * *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Div => {
            match (&left, &right) {
              (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left / right)),
              (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left / right)),
              (Value::Integer(left), Value::Float(right)) => Ok(Value::Float(*left as f64 / right)),
              (Value::Float(left), Value::Integer(right)) => Ok(Value::Float(left / *right as f64)),
              _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
            }
          }
          Token::Lt => {
            Ok(Value::Integer((left.comparison_value()? < right.comparison_value()?) as i64))
          }
          Token::Gt => {
            Ok(Value::Integer((left.comparison_value()? > right.comparison_value()?) as i64))
          }
          Token::Gte => {
            Ok(
              Value::Integer(
                (left.comparison_value()? > right.comparison_value()? ||
                  (left.comparison_value()? - right.comparison_value()?).abs() < 0.0000001) as i64))
          }
          Token::Lte => {
            Ok(
              Value::Integer(
                (left.comparison_value()? < right.comparison_value()? ||
                  (left.comparison_value()? - right.comparison_value()?).abs() < 0.0000001) as i64))
          }
          Token::EqEq => {
            Ok(
              Value::Integer(
                (left == right) as i64))
          }
          Token::Ampersand => {
            let lhs = left.is_truthy();
            let rhs = right.is_truthy();
            Ok(Value::Integer((lhs && rhs) as i64))
          }
          Token::Pipe => {
            let lhs = left.is_truthy();
            let rhs = right.is_truthy();
            Ok(Value::Integer((lhs || rhs) as i64))
          }
          _ => Err(format!("Cannot compare {:?} and {:?} with op {:?}", left, right, op))
        }
      }
      Node::UnOp { op, right } => {
        match op {
          Token::Exclamation => {
            let right = self.interpret(*right)?;
            Ok(Value::Integer(!right.is_truthy() as i64))
          }
          Token::Sub => {
            let right = self.interpret(*right)?;
            match right {
              Value::Integer(num) => Ok(Value::Integer(-num)),
              Value::Float(num) => Ok(Value::Float(-num)),
              _ => Err(format!("Cannot negate {:?}", right))
            }
          }
          Token::Add => {
            let right = self.interpret(*right)?;
            match right {
              Value::Integer(num) => Ok(Value::Integer(num)),
              Value::Float(num) => Ok(Value::Float(num)),
              _ => Err(format!("Cannot negate {:?}", right))
            }
          }
          _ => {
            return Err(format!("Unknown unary operator {:?}", op));
          }
        }
      }
      Node::BuiltinCommand { name, args } => {
        match name.as_str() {
          "refresh" => {
            Self::expect_n_args(&args, 0)?;
            self.basic.refresh = true;
            Ok(Value::Nil)
          }
          "rnd" => {
            Self::expect_n_args(&args, 2)?;
            let mut rng = rand::thread_rng();

            let min = self.interpret(args[0].clone())?.to_float()?;
            let max = self.interpret(args[1].clone())?.to_float()?;

            Ok(Value::Float(rng.gen_range(min..max)))
          }
          "gosub" => {
            Self::expect_n_args(&args, 1)?;

            let orig_line_no = self.interpret(args[0].clone())?.to_integer()? as usize;
            let line_no =
              self.basic.program.iter().position(|x| x.line_no == orig_line_no)
                .ok_or(format!("Gosub: Could not find line {}", orig_line_no))?;

            self.basic.call_stack.push(self.basic.line_no);
            self.basic.line_no = line_no;
            self.basic.no_increment_instr_counter = true;
            Ok(Value::Nil)
          }
          "delay" => {
            Self::expect_n_args(&args, 1)?;

            sleep(Duration::from_millis(self.interpret(args[0].clone())?.to_integer()? as u64));
            Ok(Value::Nil)
          }
          "sin" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?;
            match value {
              Value::Integer(num) => Ok(Value::Float((num as f64).sin())),
              Value::Float(num) => Ok(Value::Float(num.sin())),
              _ => Err(format!("Expected integer or float, got {:?}", value))
            }
          }
          "cos" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?;
            match value {
              Value::Integer(num) => Ok(Value::Float((num as f64).cos())),
              Value::Float(num) => Ok(Value::Float(num.cos())),
              _ => Err(format!("Expected integer or float, got {:?}", value))
            }
          }
          "time" => {
            Self::expect_n_args(&args, 0)?;

            let a = secs_since_start();
            Ok(Value::Float(a))
          }
          "end" => {
            Self::expect_n_args(&args, 0)?;

            self.basic.line_no = self.basic.program.len();
            Ok(Value::Nil)
          }
          "print" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?;
            self.print(value.to_string(true));
            Ok(Value::Nil)
          }
          "str" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?;
            Ok(Value::String(value.to_string(false)))
          }
          "chr" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?;
            match value {
              Value::Integer(num) => {
                if num < 0 || num > 255 {
                  return Err(format!("Expected integer between 0 and 255, got {}", num));
                }
                Ok(Value::String((num as u8 as char).to_string()))
              }
              _ => Err(format!("Expected integer, got {:?}", value))
            }
          }
          "int" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?;
            Ok(Value::Integer(value.to_integer_raw()?))
          }
          "poly" => {
            if args.len() < 2 {
              return Err(format!("Expected at least 2 arguments, got {}", args.len()));
            }

            if let Value::Array(elements) = self.interpret(args[0].clone())? {
              if elements.len() > 2 && args.len() == 2 {
                let mut points = Vec::<(i32, i32)>::new();
                for i in 0..elements.len() {
                  let point = Self::vec2i_from_value(&elements[i].clone())?;
                  points.push(point);
                }
                let color = Self::palette_idx_from_value(&self.interpret(args[1].clone())?)?;

                self.poly(points, color)?;
                return Ok(Value::Nil);
              }
            }

            if args.len() < 4 {
              return Err(format!("Expected at least 4 arguments, got {}", args.len()));
            }

            let mut points = Vec::<(i32, i32)>::new();
            for i in 0..args.len() - 1 {
              let point = Self::vec2i_from_value(&self.interpret(args[i].clone())?)?;
              points.push(point);
            }

            let color = Self::palette_idx_from_value(&self.interpret(args[args.len() - 1].clone())?)?;

            self.poly(points, color)?;
            Ok(Value::Nil)
          }
          "rim" => {
            if args.len() < 2 {
              return Err(format!("Expected at least 2 arguments, got {}", args.len()));
            }

            if let Value::Array(elements) = self.interpret(args[0].clone())? {
              if elements.len() > 2 && args.len() == 2 {
                let mut points = Vec::<(i32, i32)>::new();
                for i in 0..elements.len() {
                  let point = Self::vec2i_from_value(&elements[i])?;
                  points.push(point);
                }
                let color = Self::palette_idx_from_value(&self.interpret(args[1].clone())?)?;

                self.outline(points, color)?;
                return Ok(Value::Nil);
              }
            }

            if args.len() < 4 {
              return Err(format!("Expected at least 4 arguments, got {}", args.len()));
            }

            let mut points = Vec::<(i32, i32)>::new();
            for i in 0..args.len() - 1 {
              let point = Self::vec2i_from_value(&self.interpret(args[i].clone())?)?;
              points.push(point);
            }

            let color = Self::palette_idx_from_value(&self.interpret(args[args.len() - 1].clone())?)?;

            self.outline(points, color)?;
            Ok(Value::Nil)
          }
          "line" => {
            Self::expect_n_args(&args, 3)?;

            let x1y1 = Self::vec2i_from_value(&self.interpret(args[0].clone())?)?;
            let x2y2 = Self::vec2i_from_value(&self.interpret(args[1].clone())?)?;
            let color = Self::palette_idx_from_value(&self.interpret(args[2].clone())?)?;

            self.line(x1y1, x2y2, color);

            Ok(Value::Nil)
          }
          "next" => {
            Self::expect_n_args(&args, 1)?;

            let name = match &args[0] {
              Node::VarGet(name) => name,
              _ => return Err(format!("Expected variable name, got {:?}", args[0]))
            };

            if let Some((line_no, end, step)) = self.basic.for_stack.pop() {
              let mut value = self.basic.vars.get(name).unwrap().clone();
              match value {
                Value::Integer(ref mut num) => {
                  *num += step.to_integer()?;
                }
                Value::Float(ref mut num) => {
                  *num += step.to_float()?;
                }
                _ => return Err(format!("Expected integer or float, got {:?}", value))
              }

              let step_sign = step.to_float()?.signum();

              if value.comparison_value()? * step_sign < end.comparison_value()? {
                self.basic.vars.insert(name.clone(), value);
                self.basic.line_no = line_no;
                self.basic.for_stack.push((line_no, end, step));
                Ok(Value::Nil)
              } else {
                self.basic.vars.remove(name);
                Ok(Value::Nil)
              }
            } else {
              Err("Cannot next; for stack is empty!".to_string())
            }
          }
          "cls" => {
            if args.len() > 1 {
              return Err(format!("Expected 0 or 1 arguments, got {}", args.len()));
            }

            let color = if let Some(arg) = args.get(0) {
              Self::palette_idx_from_value(&self.interpret(arg.clone())?)?
            } else {
              0u8
            };

            self.cls(color);
            Ok(Value::Nil)
          }
          "loop" => {
            Self::expect_n_args(&args, 0)?;

            if self.basic.while_stack.len() == 0 {
              return Err("Cannot loop; while stack is empty!".to_string());
            }

            let line_no = self.basic.while_stack.last().unwrap().clone();
            let cond = match self.basic.program[line_no].node.clone() {
              Node::BuiltinCommand { name, args, } => {
                if name != "while" {
                  return Err(format!("Expected while statement, got {:?}", self.basic.program[line_no].node.clone()));
                }
                if args.len() != 1 {
                  return Err(format!("Expected 1 argument, got {}", args.len()));
                }
                println!("{:?}", args[0].clone());
                self.interpret(args[0].clone()).unwrap()
              }
              _ => return Err(format!("Expected if statement, got {:?}", self.basic.program[line_no].node.clone()))
            };

            if cond.is_truthy() {
              self.basic.line_no = line_no;
              Ok(Value::Nil)
            } else {
              self.basic.while_stack.pop();
              Ok(Value::Nil)
            }
          }
          "while" => {
            Self::expect_n_args(&args, 1)?;

            let cond = self.interpret(args[0].clone())?;
            if cond.is_truthy() {
              self.basic.while_stack.push(self.basic.line_no);
              Ok(Value::Nil)
            } else {
              let mut depth = 0;
              loop {
                if let Node::BuiltinCommand { name, args: _args } = self.basic.program[self.basic.line_no].node.clone() {
                  if name == "loop" {
                    if depth == 0 {
                      break;
                    } else {
                      depth -= 1;
                    }
                  }

                  if name == "while" {
                    depth += 1;
                  }
                }
                self.basic.line_no += 1;
              }
              Ok(Value::Nil)
            }
          }
          "goto" => {
            Self::expect_n_args(&args, 1)?;

            let orig_line_no = self.interpret(args[0].clone())?.to_integer()?;
            let line_no =
              self.basic.program
                .iter()
                .position(|x| x.line_no == orig_line_no as usize)
                .ok_or(format!("Goto: Could not find line {}", orig_line_no))?;

            self.basic.line_no = line_no;
            self.basic.no_increment_instr_counter = true;
            Ok(Value::Nil)
          }
          "ret" => {
            Self::expect_n_args(&args, 0)?;

            if let Some(line_no) = self.basic.call_stack.pop() {
              self.basic.line_no = line_no;
              Ok(Value::Nil)
            } else {
              Err("Cannot return; callstack is empty!".to_string())
            }
          }
          "dot" => {
            Self::expect_n_args(&args, 3)?;

            let x = self.interpret(args[0].clone())?.to_integer()? as i32;
            let y = self.interpret(args[1].clone())?.to_integer()? as i32;

            let color = Self::palette_idx_from_value(&self.interpret(args[2].clone())?)?;

            self.pixel(x, y, color);
            Ok(Value::Nil)
          }
          "rad" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?.to_float()?;
            Ok(Value::Float(value.to_radians()))
          }
          "deg" => {
            Self::expect_n_args(&args, 1)?;

            let value = self.interpret(args[0].clone())?.to_float()?;
            Ok(Value::Float(value.to_degrees()))
          }
          "save" => {
            Self::expect_n_args(&args, 1)?;

            let filename = self.interpret(args[0].clone())?.to_string(false);

            let file = File::create(Path::new(&filename));
            if let Err(err) = file {
              return Err(format!("Could not create file {}: {}", filename, err));
            }

            let mut file = file.unwrap();
            for line in &self.basic.program {
              if let Err(err) = writeln!(file, "{}", line.contents) {
                return Err(format!("Could not write to file {}: {}", &filename, &err));
              }
            }

            Ok(Value::Nil)
          }
          "load" => {
            if args.len() != 1 {
              return Err(format!("Expected 1 argument, got {}", args.len()));
            }

            let filename = match self.interpret(args[0].clone())? {
              Value::String(str) => str,
              _ => return Err(format!("Expected string, got {:?}", self.interpret(args[0].clone())?))
            };

            let file = File::open(Path::new(&filename));

            if let Err(err) = file {
              return Err(format!("Could not open file {}: {}", filename, err));
            }

            let mut file = file.unwrap();
            let mut buffer = String::new();

            if let Err(err) = file.read_to_string(&mut buffer) {
              return Err(format!("Could not read from file {}: {}", filename, err));
            }

            let program_vec = buffer.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
            self.basic.program.clear();

            for line in program_vec {
              if line.len() == 0 {
                continue;
              }

              self.basic.add_line(line)?;
            }

            Ok(Value::Nil)
          }
          "new" => {
            Self::expect_n_args(&args, 0)?;

            self.basic.program.clear();
            self.basic.reset_program_state();
            Ok(Value::Nil)
          }
          "text" => {
            // text "hello!" x y color shadow background
            if args.len() < 4 {
              return Err(format!("Expected at least 4 arguments, got {}", args.len()));
            }

            let text = self.interpret(args[0].clone())?.to_string(false);
            let x = self.interpret(args[1].clone())?.to_integer()? as i32;
            let y = self.interpret(args[2].clone())?.to_integer()? as i32;
            let color = Self::palette_idx_from_value(&self.interpret(args[3].clone())?)?;
            let shadow = if args.len() > 4 {
              Some(self.interpret(args[4].clone())?.to_integer()? as u8)
            } else {
              None
            };

            let background = if args.len() > 5 {
              Some(self.interpret(args[5].clone())?.to_integer()? as u8)
            } else {
              None
            };

            self.text(&*text, x, y, color, shadow, background);
            Ok(Value::Nil)
          }
          "inkey$" => {
            Self::expect_n_args(&args, 0)?;

            let key = if let Some(keycode) = self.keys_down.get(self.keys_idx) {
              keycode.name()
            } else {
              return Ok(Value::Nil);
            };
            self.keys_idx += 1;

            Ok(Value::String(key))
          }
          _ => {
            return Err(format!("Unknown builtin command {}", name));
          }
        }
      }
      Node::End => {
        self.basic.line_no = self.basic.program.len();
        Ok(Value::Nil)
      }
      Node::Nil => {
        Ok(Value::Nil)
      }
      Node::Array(elements) => {
        let mut array = Vec::new();
        for element in elements {
          array.push(self.interpret(element)?);
        }
        Ok(Value::Array(array))
      }
      Node::IndexGet { name, index } => {
        let index = self.interpret(*index)?;
        let index = match index {
          Value::Integer(num) => num as usize,
          _ => return Err(format!("Expected integer, got {:?}", index))
        };

        let array = match self.basic.vars.get(name.as_str()) {
          Some(Value::Array(array)) => array,
          _ => return Err(format!("Expected array, got {:?}", self.basic.vars.get(name.as_str())))
        };

        if index >= array.len() {
          return Err(format!("Index {} out of bounds for array of length {}", index, array.len()));
        }

        Ok(array[index].clone())
      }
      Node::IndexSet { name, index, value } => {
        let index = self.interpret(*index)?.to_integer()? as usize;
        let value = self.interpret(*value)?;

        let array = match self.basic.vars.get_mut(name.as_str()) {
          Some(Value::Array(array)) => array,
          _ => return Err(format!("Expected array, got {:?}", self.basic.vars.get(name.as_str())))
        };

        if index >= array.len() {
          return Err(format!("Index {} out of bounds for array of length {}", index, array.len()));
        }

        array[index] = value;
        Ok(Value::Nil)
      }
      Node::EmptyArray(size) => {
        let size = self.interpret(*size)?.to_integer()? as usize;
        Ok(Value::Array(vec![Value::Nil; size]))
      }
    }
  }

  pub fn exec_current_line(&mut self) -> Result<Value, String> {
    if self.basic.line_no >= self.basic.program.len() {
      return Err("Program buffer empty!".to_string());
    }

    let res = self.interpret(self.basic.program[self.basic.line_no].node.clone());
    if !self.basic.no_increment_instr_counter {
      self.basic.line_no += 1;
    }
    self.basic.no_increment_instr_counter = false;
    res
  }
}