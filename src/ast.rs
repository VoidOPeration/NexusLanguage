// src/ast.rs

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    Identifier(String),
    Boolean(bool),
    Null,
    
    Binary {
        left: Box<Expr>,
        op: String, 
        right: Box<Expr>,
    },

    Unary {
        op: String,
        right: Box<Expr>,
    },

    Slice {
        object: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },

    SafeAccess {
        object: Box<Expr>,
        name: String,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<(Option<String>, Expr)>,
    },
    
    // Доступ к полю/методу: obj.field или system.out
    Get {
        object: Box<Expr>,
        name: String,
    },
    
    Assign {
        name: String, // В будущем тут может быть Expr (для a.b = 1), пока простое имя
        value: Box<Expr>,
    },
    Index { object: Box<Expr>, index: Box<Expr> },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VariableDecl {
        name: String,
        mutable: bool, 
        var_type: String, 
        value: Option<Expr>,
    },
    
    Expression(Expr),
    Return(Option<Expr>),
    
    If {
        condition: Expr,
        then_branch: Box<Stmt>, 
        else_branch: Option<Box<Stmt>>,
    },
    
    Block(Vec<Stmt>),

    Do {
        condition: Expr,
        body: Box<Stmt>,
    },
    
    For {
        var_name: String,
        iterable: Expr,
        body: Box<Stmt>,
    },

    Defer(Box<Stmt>),
    Delete(Expr),
    
    // Новый оператор Switch
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
        default: Option<Vec<Stmt>>,
    },

    Contract {
        requires: Vec<(Expr, String)>,
        ensures: Vec<(Expr, String)>,
    },
}

// Кейс для Switch
#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub pattern: Expr, // Пока упростим: 1 или "str". Ряды (1..10) можно сделать отдельным типом Pattern
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Function {
        return_type: String,
        name: String,
        params: Vec<(String, String, Option<Expr>)>, 
        body: Vec<Stmt>,
    },
    Struct {
        name: String,
        fields: Vec<(String, String, bool)>, 
        methods: Vec<Decl>,
    },
    Enum {
        name: String,
        variants: Vec<String>,
    },
    Interface {
        name: String,
        fields: Vec<(String, String, bool)>, 
        methods: Vec<Decl>,
        structs: Vec<Decl>,
    },
    Record {
        name: String,
        fields: Vec<(String, String)>,
    },
}

#[derive(Debug)]
pub struct Program {
    pub package: String,
    pub imports: Vec<String>,
    pub declarations: Vec<Decl>,
}