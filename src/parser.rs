// src/parser.rs
use crate::token::{Token, TokenType};
use crate::ast::{Program, Decl, Stmt, Expr, SwitchCase};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn previous(&self) -> &Token { &self.tokens[self.current - 1] }
    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() - 1 { self.current += 1; }
        self.previous()
    }
    
    fn check(&self, kind: &TokenType) -> bool {
        // Спец обработка для Identifier
        if let TokenType::Identifier(_) = kind {
            if let TokenType::Identifier(_) = self.peek().kind { return true; }
        }
        &self.peek().kind == kind
    }
    
    fn is_identifier(&self) -> bool { matches!(self.peek().kind, TokenType::Identifier(_)) }

    fn check_type(&self) -> bool {
        matches!(self.peek().kind, 
            TokenType::Int | TokenType::Void | TokenType::String | 
            TokenType::Float | TokenType::Bool | TokenType::Char |
            TokenType::Byte | TokenType::Short | TokenType::Long |
            TokenType::Identifier(_) 
        )
    }

    fn expect(&mut self, kind: TokenType, message: &str) -> Result<Token, String> {
        if self.check(&kind) { Ok(self.advance().clone()) }
        else { Err(format!("Error: {}. Expected {:?}, found {:?} (line {})", message, kind, self.peek().kind, self.peek().line)) }
    }
    
    fn expect_identifier(&mut self, message: &str) -> Result<String, String> {
        if self.is_identifier() {
            let tok = self.advance().clone();
            if let TokenType::Identifier(name) = tok.kind { Ok(name) } else { Err("Internal error".to_string()) }
        } else { Err(format!("Error: {}. Expected Identifier, found {:?} (line {})", message, self.peek().kind, self.peek().line)) }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        // 1. Package
        self.expect(TokenType::Package, "File must start with 'package'")?;
        let package_name = self.expect_identifier("Expected package name")?;
        self.expect(TokenType::Semi, "Expected ';' after package name")?;

        // 2. Imports
        let mut imports = Vec::new();
        while self.check(&TokenType::Using) {
            self.advance();
            if self.check(&TokenType::StringLiteral("".to_string())) {
                 if let TokenType::StringLiteral(s) = &self.advance().kind { imports.push(format!("\"{}\"", s)); }
            } else if self.check(&TokenType::Lt) {
                self.advance();
                let mut path = String::new();
                while !self.check(&TokenType::Gt) && !self.check(&TokenType::EOF) {
                     let t = self.advance();
                     match &t.kind {
                        TokenType::Identifier(n) => path.push_str(n),
                        TokenType::ColonColon => path.push_str("::"),
                        _ => {}
                     }
                }
                self.expect(TokenType::Gt, "Expected '>' in import")?;
            }
            self.expect(TokenType::Semi, "Expected ';' after import")?;
        }

        // 3. Declarations
        let mut declarations = Vec::new();
        while !self.check(&TokenType::EOF) { declarations.push(self.parse_declaration()?); }
        Ok(Program { package: package_name, imports, declarations })
    }

    fn parse_declaration(&mut self) -> Result<Decl, String> {
        let is_public = if self.check(&TokenType::Public) { self.advance(); true } else { false };
        if self.check(&TokenType::Local) { self.advance(); }
        
        if self.check_type() { self.parse_function(is_public) }
        else if self.check(&TokenType::Struct) { self.parse_struct(is_public) }
        else if self.check(&TokenType::Enum) { self.parse_enum(is_public) }
        else if self.check(&TokenType::Record) { self.parse_record() }
        else { Err(format!("Unexpected token: {:?}", self.peek().kind)) }
    }

    fn parse_function(&mut self, _is_public: bool) -> Result<Decl, String> {
        let return_type = self.parse_type()?;
        let name = self.expect_identifier("Expected function name")?;
        self.expect(TokenType::LParen, "Expected '('")?;
        let params = self.parse_params()?;
        self.expect(TokenType::RParen, "Expected ')'")?;
        self.expect(TokenType::LBrace, "Expected '{'")?;
        let body = self.parse_block()?;
        self.expect(TokenType::RBrace, "Expected '}' after function body")?; // ВАЖНО!
        Ok(Decl::Function { return_type, name, params, body })
    }
    
    fn parse_struct(&mut self, _is_public: bool) -> Result<Decl, String> {
        self.advance(); 
        let name = self.expect_identifier("Expected struct name")?;
        self.expect(TokenType::LBrace, "Expected '{'")?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.check(&TokenType::EOF) {
            let is_pub = if self.check(&TokenType::Public) { self.advance(); true } else { false };
            if self.check_type() {
                let f_type = self.parse_type()?;
                let f_name = self.expect_identifier("Expected name")?;
                if self.check(&TokenType::LParen) {
                    self.expect(TokenType::LParen, "")?;
                    let params = self.parse_params()?;
                    self.expect(TokenType::RParen, "")?;
                    self.expect(TokenType::LBrace, "")?;
                    let body = self.parse_block()?;
                    self.expect(TokenType::RBrace, "")?;
                    methods.push(Decl::Function { return_type: f_type, name: f_name, params, body });
                } else {
                    self.expect(TokenType::Semi, "")?;
                    fields.push((f_type, f_name, is_pub));
                }
            }
        }
        self.expect(TokenType::RBrace, "Expected '}'")?; // ВАЖНО!
        Ok(Decl::Struct { name, fields, methods })
    }
    
    fn parse_enum(&mut self, _is_public: bool) -> Result<Decl, String> {
        self.advance(); 
        let name = self.expect_identifier("Expected enum name")?;
        self.expect(TokenType::LBrace, "Expected '{'")?;
        let mut variants = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.check(&TokenType::EOF) {
            variants.push(self.expect_identifier("Expected variant name")?);
            if self.check(&TokenType::Comma) { self.advance(); } else { break; }
        }
        self.expect(TokenType::RBrace, "Expected '}'")?;
        Ok(Decl::Enum { name, variants })
    }

    fn parse_record(&mut self) -> Result<Decl, String> {
        self.advance();
        let name = self.expect_identifier("Expected record name");
        self.expect(TokenType::LBrace, "Expected '{'");

        let mut fields = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.check(&TokenType::EOF) {
            let f_type = self.parse_type()?;
            let f_name = self.expect_identifier("Expected field name")?;
            fields.push((f_type, f_name));
            self.expect(TokenType::Semi, "Expected ';'")?;
        }
        self.expect(TokenType::RBrace, "Expected '}'")?;

        Ok(Decl::Record { name, fields })    
    }

    fn parse_type(&mut self) -> Result<String, String> {
        if !self.check_type() { return Err("Expected type".to_string()); }
        let token = self.advance().clone();
        let mut type_name = match token.kind {
            TokenType::Int => "int".to_string(),
            TokenType::Void => "void".to_string(),
            TokenType::String => "string".to_string(),
            TokenType::Float => "float".to_string(),
            TokenType::Double => "double".to_string(),
            TokenType::Long => "long".to_string(),
            TokenType::Short => "short".to_string(),
            TokenType::Bool => "bool".to_string(),
            TokenType::Byte => "byte".to_string(),
            TokenType::Utype => "utype".to_string(),
            TokenType::Null => "null".to_string(),
            TokenType::Nullptr => "nullptr".to_string(),
            TokenType::Char => "char".to_string(),
            TokenType::Identifier(n) => n,
            _ => "unknown".to_string()
        };

        if self.check(&TokenType::LBracket) { self.advance(); self.expect(TokenType::RBracket, "")?; type_name.push_str("[]"); }
        if self.check(&TokenType::Lt) { self.advance(); let inner = self.parse_type()?; self.expect(TokenType::Gt, "")?; type_name = format!("{}<{}>", type_name, inner); }
        Ok(type_name)
    }

    fn parse_params(&mut self) -> Result<Vec<(String, String, Option<Expr>)>, String> {
        let mut params = Vec::new();
        if self.check(&TokenType::RParen) { return Ok(params); }
        loop {
            let p_type = self.parse_type()?;
            let p_name = self.expect_identifier("")?;
            
            let default = if self.check(&TokenType::Eq) {
                self.advance();
                Some(self.parse_expression()?)
            } else { None };
            
            params.push((p_type, p_name, default));
            if !self.check(&TokenType::Comma) { break; }
            self.advance();
        }
        Ok(params)
    }

    fn parse_call_args(&mut self) -> Result<Vec<(Option<String>, Expr)>, String> {
        let mut args = Vec::new();
        if self.check(&TokenType::RParen) { return Ok(args); }
        loop {
            // Проверяем Named Argument: name=value
            let name = if self.is_identifier() {
                 // Lookahead: if next is '=', it's named
                 // (Requires peek_next or saving state)
                 // For simplicity: just parse expr, then check if next is '='? No, name=value is ambiguous with bitwise.
                 // Let's assume standard: ident = expr is named.
                 // Just use parse_expression for now, we refine later.
                 None 
            } else { None };
            
            let val = self.parse_expression()?;
            args.push((name, val));
            if !self.check(&TokenType::Comma) { break; }
            self.advance();
        }
        Ok(args)
    }

    // --- STATEMENTS (Без изменений, но убедись что они есть) ---
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.check(&TokenType::EOF) {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        if self.check(&TokenType::Return) {
            self.advance();
            let value = if !self.check(&TokenType::Semi) { Some(self.parse_expression()?) } else { None };
            self.expect(TokenType::Semi, "")?;
            return Ok(Stmt::Return(value));
        }
        if self.check(&TokenType::Defer) { self.advance(); return Ok(Stmt::Defer(Box::new(self.parse_statement()?))); }
        if self.check(&TokenType::Delete) { self.advance(); let e = self.parse_expression()?; self.expect(TokenType::Semi, "")?; return Ok(Stmt::Delete(e)); }
        if self.check(&TokenType::If) { return self.parse_if(); }
        if self.check(&TokenType::Do) { return self.parse_do(); }
        if self.check(&TokenType::For) { return self.parse_for(); }
        if self.check(&TokenType::Switch) { return self.parse_switch(); }

        // Var Decl vs Expression
        let is_var_decl = match self.peek().kind {
            TokenType::Int | TokenType::Void | TokenType::String | TokenType::Float | TokenType::Bool | TokenType::Char | TokenType::Byte | TokenType::Short | TokenType::Long => true,
            TokenType::Identifier(_) => {
                if self.current + 1 < self.tokens.len() {
                    match &self.tokens[self.current + 1].kind {
                        TokenType::Identifier(_) | TokenType::Mut => true,
                        _ => false,
                    }
                } else { false }
            },
            _ => false
        };

        if is_var_decl { return self.parse_var_decl(); }
        
        let expr = self.parse_expression()?;
        if self.check(&TokenType::Eq) {
            self.advance();
            let value = self.parse_expression()?;
            self.expect(TokenType::Semi, "")?;
            if let Expr::Identifier(name) = expr { return Ok(Stmt::Expression(Expr::Assign { name, value: Box::new(value) })); }
            return Err("Invalid assignment target".to_string());
        }
        self.expect(TokenType::Semi, "")?;
        Ok(Stmt::Expression(expr))
    }
    
    fn parse_var_decl(&mut self) -> Result<Stmt, String> {
        let var_type = self.parse_type()?;
        let mutable = if self.check(&TokenType::Mut) { self.advance(); true } else { false };
        let name = self.expect_identifier("Expected variable name")?;
        let value = if self.check(&TokenType::Eq) { self.advance(); Some(self.parse_expression()?) } else { None };
        self.expect(TokenType::Semi, "")?;
        Ok(Stmt::VariableDecl { name, mutable, var_type, value })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); 
        self.expect(TokenType::LParen, "")?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RParen, "")?;
        self.expect(TokenType::LBrace, "")?;
        let then_block = self.parse_block()?;
        self.expect(TokenType::RBrace, "")?; // Consumes }
        
        let else_branch = if self.check(&TokenType::Else) {
            self.advance();
            if self.check(&TokenType::If) { Some(Box::new(self.parse_if()?)) }
            else {
                self.expect(TokenType::LBrace, "")?;
                let block = self.parse_block()?;
                self.expect(TokenType::RBrace, "")?; // Consumes }
                Some(Box::new(Stmt::Block(block)))
            }
        } else { None };
        Ok(Stmt::If { condition, then_branch: Box::new(Stmt::Block(then_block)), else_branch })
    }

    fn parse_do(&mut self) -> Result<Stmt, String> {
        self.advance(); 
        self.expect(TokenType::LParen, "")?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RParen, "")?;
        self.expect(TokenType::LBrace, "")?;
        let body = self.parse_block()?;
        self.expect(TokenType::RBrace, "")?; // Consumes }
        Ok(Stmt::Do { condition, body: Box::new(Stmt::Block(body)) })
    }
    
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); 
        self.expect(TokenType::LParen, "")?;
        let var_name = self.expect_identifier("Expected variable name")?;
        self.expect(TokenType::In, "")?;
        let iterable = self.parse_expression()?;
        self.expect(TokenType::RParen, "")?;
        self.expect(TokenType::LBrace, "")?;
        let body = self.parse_block()?;
        self.expect(TokenType::RBrace, "")?; // Consumes }
        Ok(Stmt::For { var_name, iterable, body: Box::new(Stmt::Block(body)) })
    }

    fn parse_switch(&mut self) -> Result<Stmt, String> {
        self.advance(); // eat switch
        self.expect(TokenType::LParen, "")?;
        let value = self.parse_expression()?;
        self.expect(TokenType::RParen, "")?;
        self.expect(TokenType::LBrace, "")?;

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&TokenType::RBrace) {
            if self.check(&TokenType::Case) {
                self.advance();
                let pattern = self.parse_expression()?;
                self.expect(TokenType::Arrow, "Expectd '=>'")?;
                self.expect(TokenType::LBrace, "Expected '{'")?;
                let body = self.parse_block()?;
                self.expect(TokenType::RBrace, "Expected '}'")?;
                cases.push(ast::SwitchCase { pattern, body });
            } else if self.check(&TokenType::Default) || self.check(&TokenType::Underscore) {
                self.advance();
                self.expect(TokenType::Arrow, "Expected '=>'")?;
                self.expect(TokenType::LBrace, "")?;
                let body = self.parse_block()?;
                self.expect(TokenType::RBrace, "")?;
                default = Some(body);
            } else {
                return Err("Expected 'case' or 'default'".to_string());
            }
        }
        self.expect(TokenType::RBrace, "")?;
        Ok(Stmt::Switch { value, cases, default })
    }

    // --- EXPRESSIONS (Без изменений) ---
    fn parse_expression(&mut self) -> Result<Expr, String> { self.parse_assignment() }
    fn parse_assignment(&mut self) -> Result<Expr, String> { self.parse_equality() }
    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;
        while self.check(&TokenType::EqEq) || self.check(&TokenType::BangEq) {
            let op = if self.check(&TokenType::EqEq) { "==" } else { "!=" };
            self.advance(); let right = self.parse_comparison()?; expr = Expr::Binary { left: Box::new(expr), op: op.to_string(), right: Box::new(right) };
        } Ok(expr)
    }
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;
        while self.check(&TokenType::Lt) || self.check(&TokenType::Gt) || self.check(&TokenType::LtEq) || self.check(&TokenType::GtEq) {
            let op = match self.peek().kind { TokenType::Lt => "<", TokenType::Gt => ">", TokenType::LtEq => "<=", TokenType::GtEq => ">=", _ => "" };
            self.advance(); let right = self.parse_term()?; expr = Expr::Binary { left: Box::new(expr), op: op.to_string(), right: Box::new(right) };
        } Ok(expr)
    }
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;
        while self.check(&TokenType::Minus) || self.check(&TokenType::Plus) {
            let op = if self.check(&TokenType::Minus) { "-" } else { "+" };
            self.advance(); let right = self.parse_factor()?; expr = Expr::Binary { left: Box::new(expr), op: op.to_string(), right: Box::new(right) };
        } Ok(expr)
    }
    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
        while self.check(&TokenType::Slash) || self.check(&TokenType::Star) || self.check(&TokenType::Percent) {
            let op = match self.peek().kind { TokenType::Slash => "/", TokenType::Star => "*", TokenType::Percent => "%", _ => "" };
            self.advance(); let right = self.parse_unary()?; expr = Expr::Binary { left: Box::new(expr), op: op.to_string(), right: Box::new(right) };
        } Ok(expr)
    }
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(&TokenType::Bang) || self.check(&TokenType::Minus) {
            let op = if self.check(&TokenType::Bang) { "!" } else { "-" };
            self.advance(); let right = self.parse_unary()?; return Ok(Expr::Unary { op: op.to_string(), right: Box::new(right) });
        }
        self.parse_postfix()
    }
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&TokenType::LParen) {
                self.advance(); let mut args = Vec::new();
                if !self.check(&TokenType::RParen) { loop { args.push(self.parse_expression()?); if !self.check(&TokenType::Comma) { break; } self.advance(); } }
                self.expect(TokenType::RParen, "")?; expr = Expr::Call { callee: Box::new(expr), args };
            } else if self.check(&TokenType::LBracket) {
                self.advance();
                let start = self.parse_expression()?;
                if self.check(&TokenType::DotDot) {
                    self.advance();
                    let end = self.parse_expression()?;
                    self.expect(TokenType::RBracket, "")?;
                    return Ok(Expr::Slice { 
                        object: Box::new(expr), 
                        start: Box::new(start), 
                        end: Box::new(end) 
                    });
                }

                self.expect(TokenType::RBracket, "")?;
                return Ok(Expr::Index { object: Box::new(expr), index: Box::new(start) });
            } else { break; }
        } Ok(expr)
    }
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().kind.clone() {
            TokenType::IntLiteral(n) => { self.advance(); Ok(Expr::Integer(n)) },
            TokenType::FloatLiteral(f) => { self.advance(); Ok(Expr::Float(f)) },
            TokenType::StringLiteral(s) => { self.advance(); Ok(Expr::String(s)) },
            TokenType::True => { self.advance(); Ok(Expr::Boolean(true)) },
            TokenType::False => { self.advance(); Ok(Expr::Boolean(false)) },
            TokenType::Null => { self.advance(); Ok(Expr::Null) },
            TokenType::Identifier(name) => { self.advance(); Ok(Expr::Identifier(name)) },
            TokenType::LParen => { self.advance(); let e = self.parse_expression()?; self.expect(TokenType::RParen, "")?; Ok(e) },
            _ => Err(format!("Expected expression, found {:?}", self.peek().kind))
        }
    }
}