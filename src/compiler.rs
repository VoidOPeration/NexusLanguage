// src/compiler.rs
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;
use inkwell::types::BasicType;
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel};
use inkwell::OptimizationLevel;
use std::path::Path;

use crate::ast::{Program, Decl, Stmt, Expr};

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
    // Хранилище переменных (имя -> указатель)
    variables: std::collections::HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        // -- SDK INIT --
        let void_type = context.void_type();
        let i32_type = context.i32_type();
        let i8_type = context.i8_type();

        let printf_type = i32_type.fn_type(&[context.ptr_type(Default::default()).into()], true);
        module.add_function("printf", printf_type,None);
        
        Compiler { 
            context, 
            module, 
            builder,
            fn_value_opt: None,
            variables: std::collections::HashMap::new(),
        }
    }

    // Запуск компиляции
    pub fn compile(&mut self, program: &Program) -> Result<(), String> {
        self.declare_sdk_functions();
        
        for decl in &program.declarations {
            match decl {
                Decl::Function { name, params, return_type, body } => {
                    self.compile_function(name, params, return_type, body)?;
                },
                Decl::Record { name, fields } => {
                    self.compile_record(name, fields)?;
                },
                _ => {}
            }
        }
        
        // Валидация модуля (проверка ошибок LLVM)
        if self.module.verify().is_err() {
            return Err("Module verification failed".to_string());
        }
        
        Ok(())
    }

    // Печать IR
    pub fn print_ir(&self) {
        self.module.print_to_stderr();
    }

    fn declare_sdk_functions(&self) {
        let ctx = self.context;
        let _i8_type = ctx.i8_type();
        let _i32_type = ctx.i32_type();
        let i64_type = ctx.i64_type();
        let void_type = ctx.void_type();
        
        // Новый способ получения типа указателя (устраняет warning deprecated)
        let ptr_type = ctx.ptr_type(Default::default()); 

        // --- iostream ---
        // void nx_iostream_print(i8* ptr, i64 len)
        let fn_type = void_type.fn_type(&[
            ptr_type.into(),
            i64_type.into(),
        ], false);
        self.module.add_function("nx_iostream_print", fn_type, None);
        self.module.add_function("nx_iostream_println", fn_type, None);
        
        // --- arraylist ---
        let fn_type_ptr = ptr_type.fn_type(&[], false);
        self.module.add_function("nx_arraylist_int_new", fn_type_ptr, None);

        let fn_type_push = void_type.fn_type(&[
            ptr_type.into(),
            _i32_type.into(),
        ], false);
        self.module.add_function("nx_arraylist_int_push", fn_type_push, None);
    }

    fn compile_function(
        &mut self, 
        name: &str, 
        _params: &Vec<(String, String)>, 
        return_type: &str, 
        body: &Vec<Stmt>
    ) -> Result<FunctionValue<'ctx>, String> {
        
        // Очищаем переменные для новой функции
        self.variables.clear();

        // 1. Определяем тип возвращаемого значения
        // LLVM требует конкретный тип, нельзя сделать переменную типа "Type".
        // Поэтому обрабатываем void отдельно.
        let fn_type = if return_type == "void" {
            self.context.void_type().fn_type(&[], false)
        } else {
            // По умолчанию int (i32)
            self.context.i32_type().fn_type(&[], false)
        };

        let function = self.module.add_function(name, fn_type, None);
        let entry = self.context.append_basic_block(function, "entry");
        
        self.builder.position_at_end(entry);
        self.fn_value_opt = Some(function);

        // Компилируем тело
        for stmt in body {
            self.compile_stmt(stmt)?;
        }
        
        // Если функция void, добавляем return void, если его не было
        if return_type == "void" {
            // Проверка: если блок не завершен (нет ret), добавляем
            if self.builder.get_insert_block().and_then(|bb| bb.get_terminator()).is_none() {
                 self.builder.build_return(None).map_err(|e| format!("{:?}", e))?;
            }
        }

        Ok(function)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.compile_expr(e)?
                } else {
                    self.context.i32_type().const_int(0, false)
                };
                self.builder.build_return(Some(&val)).map_err(|e| format!("{:?}", e))?;
                Ok(())
            },
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.compile_stmt(s)?;
                }
                Ok(())
            },
            Stmt::VariableDecl { name, var_type, value, .. } => {
                // Тип переменной
                let ty = if var_type == "int" { 
                    self.context.i32_type().as_basic_type_enum() 
                } else { 
                    self.context.i32_type().as_basic_type_enum() 
                };

                // Аллоцируем память (alloca)
                let ptr = self.builder.build_alloca(ty, name).map_err(|e| format!("{:?}", e))?;
                
                // Инициализация
                if let Some(init_val) = value {
                    let val = self.compile_expr(init_val)?;
                    self.builder.build_store(ptr, val).map_err(|e| format!("{:?}", e))?;
                }
                
                // Сохраняем в таблице символов
                self.variables.insert(name.clone(), ptr);
                Ok(())
            },
            Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
                Ok(())
            },
            Stmt::If { condition, then_branch, else_branch } => {
                let function = self.fn_value_opt.unwrap();

                // Компилируем условие
                let cond = self.compile_expr(condition)?;
                
                // Сравниваем с 0 (в LLVM нет bool, это i1, приводим к i1)
                let cond_bool = self.builder.build_int_compare(IntPredicate::NE, cond, self.context.i32_type().const_int(0, false), "ifcond").map_err(|e| format!("{:?}", e))?;

                let then_block = self.context.append_basic_block(function, "then");
                let else_block = self.context.append_basic_block(function, "else");
                let merge_block = self.context.append_basic_block(function, "merge");

                // Branch
                self.builder.build_conditional_branch(cond_bool, then_block, else_block).map_err(|e| format!("{:?}", e))?;

                // THEN
                self.builder.position_at_end(then_block);
                self.compile_stmt(then_branch)?;
                // Если блок не завершен, прыгаем в merge
                if self.builder.get_insert_block().and_then(|bb| bb.get_terminator()).is_none() {
                    self.builder.build_unconditional_branch(merge_block).map_err(|e| format!("{:?}", e))?;
                }

                // ELSE
                self.builder.position_at_end(else_block);
                if let Some(else_stmt) = else_branch {
                    self.compile_stmt(else_stmt)?;
                }
                if self.builder.get_insert_block().and_then(|bb| bb.get_terminator()).is_none() {
                    self.builder.build_unconditional_branch(merge_block).map_err(|e| format!("{:?}", e))?;
                }

                // MERGE
                self.builder.position_at_end(merge_block);
                Ok(())
            }
            _ => Ok(())
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>, String> {
        match expr {
            Expr::Integer(n) => {
                Ok(self.context.i32_type().const_int(*n as u64, false))
            },
            Expr::Identifier(name) => {
                // Ищем переменную
                if let Some(ptr) = self.variables.get(name) {
                    // Загружаем значение из памяти
                    let loaded = self.builder.build_load(self.context.i32_type(), *ptr, name).map_err(|e| format!("{:?}", e))?;
                    // TryToIntValue trait или метод
                    Ok(loaded.into_int_value())
                } else {
                    Err(format!("Unknown variable: {}", name))
                }
            },
            Expr::Binary { left, op, right } => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                
                let result = match op.as_str() {
                    "+" => self.builder.build_int_add(l, r, "addtmp"),
                    "-" => self.builder.build_int_sub(l, r, "subtmp"),
                    "*" => self.builder.build_int_mul(l, r, "multmp"),
                    "/" => self.builder.build_int_signed_div(l, r, "divtmp"),
                    "<" => {
                        let cmp = self.builder.build_int_compare(IntPredicate::SLT, l, r, "cmptmp").map_err(|e| format!("{:?}", e))?;
                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "bool_ext")
                    },
                    ">" => {
                        let cmp = self.builder.build_int_compare(IntPredicate::SGT, l, r, "cmptmp").map_err(|e| format!("{:?}", e))?;
                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "bool_ext")
                    },
                    "==" => {
                        let cmp = self.builder.build_int_compare(IntPredicate::EQ, l, r, "cmptmp").map_err(|e| format!("{:?}", e))?;
                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "bool_ext")
                    },
                     _ => return Err(format!("Unknown operator: {}", op))
                };
                
                result.map_err(|e| format!("{:?}", e))
            },
            Expr::Assign { name, value } => {
                let val = self.compile_expr(value)?;
                if let Some(ptr) = self.variables.get(name) {
                    self.builder.build_store(*ptr, val).map_err(|e| format!("{:?}", e))?;
                    Ok(val) // Присваивание возвращает значение
                } else {
                    Err(format!("Cannot assign to unknown variable: {}", name))
                }
            },
            Expr::Call { callee, args } => {
                if let Expr::Get { object, name } = callee.as_ref() {
                    if name == "println" {
                        if let Expr::Get { object: inner_obj, name: inner_name } = object.as_ref() {
                            if inner_name == "out" {
                                // Это system.out.println!
                                return self.compile_println(args);
                            }
                        }
                    }
                }
                
                let _func_name = "";
                let _callee_val = 0;

                Ok(self.context.i32_type().const_int(0, false))
            },
            _ => Ok(self.context.i32_type().const_int(0, false))
        }
    }
    pub fn compile_to_object(&self, out_path: &str) -> Result<(), String> {
        Target::initialize_x86(&InitializationConfig::default());
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| format!("{:?}", e))?;

        let machine = target.create_target_machine(
            &triple, 
            "generic", 
            "", 
            OptimizationLevel::Default, 
            RelocMode::Default, 
            CodeModel::Default,
        ).ok_or("Failed to create TargetMachine")?;

        let path = Path::new(out_path);
        machine.write_to_file(&self.module, inkwell::targets::FileType::Object, path)
            .map_err(|e| format!("Failed to write Object file {:?}", e))?;


        Ok(())
    }

    fn compile_println(&mut self, args: &Vec<Expr>) -> Result<IntValue<'ctx>, String> {
        if args.is_empty() { return Err("println expects 1 argument".to_string()); }
        
        let arg = &args[0];

        match arg {
            Expr::String(s) => {
                let global_str = self.builder.build_global_string_ptr(s, "str")
                    .map_err(|e| format!("Failed to create global string: {:?}", e))?;

                let len = self.context.i64_type().const_int(s.len() as u64, false);

                let fn_val = self.module.get_function("nx_iostream_println").unwrap();
                self.builder.build_call(fn_val, &[
                    global_str.as_pointer_value().into(),
                    len.into(),
                ], "println_call");

                Ok(self.context.i32_type().const_int(0, false))
            },
            _ => return Err("Native println supports only strings for now".to_string()),
        }
    }

    fn compile_record(&mut self, name: &str, fields: &Vec<(String, String, Option<Expr>)>) -> Result<(), String> {
        let mut field_type: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![];
        for (_, _) in fields {
            field_type.push(self.context.i32_type().into());
        }

        let struct_type = self.context.opaque_struct_type(name);
        struct_type.set_body(field_types, false);

        let fn_type = struct_type.fn_type(&field_type, false);
        let func = self.module.add_function(&format!("{}_new", name), fn_type, None);

        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let alloc = self.builder.build_alloca(struct_type, "inst");

        for (i, _) in fields.iter().enumerate() {
            let param = func.get_nth_param(i as u32).unwrap();
            let ptr = unsafe { self.builder.build_struct_gep(alloc, i as u32, "field").map_err(|e| format!("{:?}", e))? };
            self.builder.build_store(ptr, param);
        }

        let loaded = self.builder.build_load(struct_type, alloc, "ret_val");
        self.builder.build_return(Some(&loaded));
        
        Ok(())
    }
}