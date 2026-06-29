#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Package, Using, Public, Local, Static, 
    Func, Struct, Interface, Enum, 
    If, Else, Switch, Do, For, Return, 
    Mut, New, Delete, Defer, In,
    Type, Const, Comptime, Arena, SelfKw,
    True, False, Default, Break, Continue,
    As, Record, Case, 
    
    Int, Short, Long, Float, Double, Bool, Char, Byte, String, Utype,
    Null, Nullptr, Void,
    Ptr, Ref, Slice, Area, Result,
    
    Identifier(String),  // myVar
    StringLiteral(String), // "hello"
    IntLiteral(i64),     // 123
    FloatLiteral(f64),   // 12.5
    
    Plus, Minus, Star, Slash, Percent,
    Eq, EqEq, Bang, BangEq, // =, ==, !, !=
    Lt, Gt, LtEq, GtEq,     // <, >, <=, >=
    Arrow,                  // ->
    ColonColon,             // ::
    Dot, Comma, Colon, Semi, // . , : ;
    DotDot, Question, // .. ?
    
    DoubleQuestion, 
    DoubleBang,
    Underscore,      
    
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,

    Unknown(char),
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub line: usize,
    pub column: usize,
}