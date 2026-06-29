// src/vm.rs
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Read};

use crate::ast::{Program, Decl, Stmt, Expr};

// --- ОПРЕДЕЛЕНИЕ ИНСТРУКЦИЙ (BYTECODE) ---

#[derive(Debug, Clone)]
pub enum OpCode {
    LoadConst(i64),
    LoadVar(String),
    StoreVar(String),
    Add,
    Sub,
    Mul,
    Div,
    CmpLt,
    CmpGt,
    CmpEq,
    JmpIfFalse(usize),
    Jmp(usize),
    Return,
    Halt,
}

// --- КОМПИЛЯТОР В БАЙТ-КОД ---

pub struct ByteCompiler {
    instructions: Vec<OpCode>,
}

impl ByteCompiler {
    pub fn new() -> Self {
        Self { instructions: Vec::new() }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<Vec<OpCode>, String> {
        for decl in &program.declarations {
            if let Decl::Function { name, body, .. } = decl {
                if name == "main" {
                    self.compile_block(body)?;
                    self.instructions.push(OpCode::Return);
                    return Ok(self.instructions.clone());
                }
            }
        }
        Err("No main function found".to_string())
    }

    fn compile_block(&mut self, stmts: &Vec<Stmt>) -> Result<(), String> {
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(stmts) => {
                self.compile_block(stmts)?;
            },
            Stmt::VariableDecl { name, value, .. } => {
                match value {
                    Some(v) => {
                        self.compile_expr(v)?;
                        self.instructions.push(OpCode::StoreVar(name.clone()));
                    },
                    None => {}
                }
            },
            Stmt::Return(opt_expr) => {
                if let Some(e) = opt_expr {
                    self.compile_expr(e)?;
                } else {
                    self.instructions.push(OpCode::LoadConst(0));
                }
                self.instructions.push(OpCode::Return);
            },
            Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
            },
            Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expr(condition)?;
                
                let else_jump_idx = self.instructions.len();
                self.instructions.push(OpCode::JmpIfFalse(0)); 
                
                self.compile_stmt(then_branch)?;
                let end_jump_idx = self.instructions.len();
                self.instructions.push(OpCode::Jmp(0)); 

                let else_start_idx = self.instructions.len();
                self.instructions[else_jump_idx] = OpCode::JmpIfFalse(else_start_idx);
                
                if let Some(else_stmt) = else_branch {
                    self.compile_stmt(else_stmt)?;
                }
                
                let end_idx = self.instructions.len();
                self.instructions[end_jump_idx] = OpCode::Jmp(end_idx);
            },
                        Stmt::Switch { value, cases, default } => {
                self.compile_expr(value)?;
                
                let end_jump_idx = self.instructions.len(); // Заглушка
                self.instructions.push(OpCode::Jmp(0)); // Конец свича
                
                // Проходим по кейсам
                for case in cases {
                    // Сравниваем value (на стеке) с pattern
                    self.compile_expr(&case.pattern)?;
                    self.instructions.push(OpCode::CmpEq);
                    
                    let next_case_idx = self.instructions.len();
                    self.instructions.push(OpCode::JmpIfFalse(0));
                    
                    // Body
                    self.compile_block(&case.body)?;
                    
                    // Прыжок в конец свича
                    // (Нужно патчить end_jump_idx)
                }
                
                // Default
                if let Some(d) = default {
                    self.compile_block(d)?;
                }
                
                // Патчим конец
                let end_idx = self.instructions.len();
                self.instructions[end_jump_idx] = OpCode::Jmp(end_idx);
            },
            _ => {}
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Integer(n) => self.instructions.push(OpCode::LoadConst(*n)),
            Expr::Identifier(name) => self.instructions.push(OpCode::LoadVar(name.clone())),
            Expr::Binary { left, op, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op.as_str() {
                    "+" => self.instructions.push(OpCode::Add),
                    "-" => self.instructions.push(OpCode::Sub),
                    "*" => self.instructions.push(OpCode::Mul),
                    "/" => self.instructions.push(OpCode::Div),
                    "<" => self.instructions.push(OpCode::CmpLt),
                    ">" => self.instructions.push(OpCode::CmpGt),
                    "==" => self.instructions.push(OpCode::CmpEq),
                    _ => return Err(format!("Unknown op: {}", op)),
                }
            },
            Expr::Assign { name, value } => {
                self.compile_expr(value)?;
                self.instructions.push(OpCode::StoreVar(name.clone()));
                self.instructions.push(OpCode::LoadVar(name.clone()));
            },
            _ => {}
        }
        Ok(())
    }
}

// --- ВИРТУАЛЬНАЯ МАШИНА ---

pub struct VM {
    stack: Vec<i64>,
    globals: HashMap<String, i64>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self, instructions: &[OpCode]) -> Result<i64, String> {
        let mut ip = 0;
        
        loop {
            if ip >= instructions.len() { break; }
            
            let opcode = &instructions[ip];

            match opcode {
                OpCode::LoadConst(n) => self.stack.push(*n),
                OpCode::LoadVar(name) => {
                    let val = self.globals.get(name).cloned().unwrap_or(0);
                    self.stack.push(val);
                },
                OpCode::StoreVar(name) => {
                    let val = self.stack.pop().ok_or("Stack empty on store")?;
                    self.globals.insert(name.clone(), val);
                },
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                },
                OpCode::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a - b);
                },
                OpCode::Mul => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a * b);
                },
                OpCode::Div => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a / b);
                },
                OpCode::CmpLt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(if a < b { 1 } else { 0 });
                },
                OpCode::CmpGt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(if a > b { 1 } else { 0 });
                },
                OpCode::CmpEq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(if a == b { 1 } else { 0 });
                },
                OpCode::JmpIfFalse(target) => {
                    let val = self.stack.pop().unwrap();
                    if val == 0 {
                        ip = *target;
                        continue;
                    }
                },
                OpCode::Jmp(target) => {
                    ip = *target;
                    continue;
                },
                OpCode::Return => {
                    return self.stack.pop().ok_or("No return value".to_string());
                },
                OpCode::Halt => return Ok(0),
            }
            ip += 1;
        }
        Ok(0)
    }
}

// --- СЕРИАЛИЗАЦИЯ ---

impl OpCode {
    pub fn to_string_repr(&self) -> String {
        match self {
            OpCode::LoadConst(n) => format!("CONST {}", n),
            OpCode::LoadVar(s) => format!("LOAD {}", s),
            OpCode::StoreVar(s) => format!("STORE {}", s),
            OpCode::Add => "ADD".to_string(),
            OpCode::Sub => "SUB".to_string(),
            OpCode::Mul => "MUL".to_string(),
            OpCode::Div => "DIV".to_string(),
            OpCode::CmpLt => "LT".to_string(),
            OpCode::CmpGt => "GT".to_string(),
            OpCode::CmpEq => "EQ".to_string(),
            OpCode::JmpIfFalse(addr) => format!("JMPF {}", addr),
            OpCode::Jmp(addr) => format!("JMP {}", addr),
            OpCode::Return => "RET".to_string(),
            OpCode::Halt => "HALT".to_string(),
        }
    }
}

pub fn save_bytecode(instructions: &[OpCode], path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    for op in instructions {
        writeln!(file, "{}", op.to_string_repr())?;
    }
    Ok(())
}

pub fn load_bytecode(path: &str) -> std::io::Result<Vec<OpCode>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let mut instructions = Vec::new();
    for line in contents.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }
        
        let op = match parts[0] {
            "CONST" => OpCode::LoadConst(parts[1].parse().unwrap()),
            "LOAD" => OpCode::LoadVar(parts[1].to_string()),
            "STORE" => OpCode::StoreVar(parts[1].to_string()),
            "ADD" => OpCode::Add,
            "SUB" => OpCode::Sub,
            "MUL" => OpCode::Mul,
            "DIV" => OpCode::Div,
            "LT" => OpCode::CmpLt,
            "GT" => OpCode::CmpGt,
            "EQ" => OpCode::CmpEq,
            "JMPF" => OpCode::JmpIfFalse(parts[1].parse().unwrap()),
            "JMP" => OpCode::Jmp(parts[1].parse().unwrap()),
            "RET" => OpCode::Return,
            "HALT" => OpCode::Halt,
            _ => panic!("Unknown opcode in file: {}", line),
        };
        instructions.push(op);
    }
    Ok(instructions)
}