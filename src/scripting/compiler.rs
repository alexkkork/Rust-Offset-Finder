// Tue Jan 15 2026 - Alex

use crate::scripting::types::ScriptValue;
use std::collections::HashMap;
use std::fmt;

/// Script compiler
pub struct ScriptCompiler {
    optimization_level: u8,
    strict_mode: bool,
}

impl ScriptCompiler {
    pub fn new() -> Self {
        Self {
            optimization_level: 1,
            strict_mode: false,
        }
    }

    pub fn with_optimization(mut self, level: u8) -> Self {
        self.optimization_level = level;
        self
    }

    pub fn strict_mode(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Compile source code to bytecode
    pub fn compile(&self, source: &str) -> Result<CompiledScript, CompileError> {
        let tokens = self.tokenize(source)?;
        let ast = self.parse(&tokens)?;
        let bytecode = self.generate(&ast)?;

        if self.optimization_level > 0 {
            self.optimize(bytecode)
        } else {
            Ok(bytecode)
        }
    }

    /// Tokenize source code
    fn tokenize(&self, source: &str) -> Result<Vec<Token>, CompileError> {
        let mut tokens = Vec::new();
        let mut chars = source.chars().peekable();
        let mut line = 1;
        let mut column = 1;

        while let Some(&c) = chars.peek() {
            let start_col = column;

            match c {
                // Whitespace
                ' ' | '\t' | '\r' => {
                    chars.next();
                    column += 1;
                }
                '\n' => {
                    chars.next();
                    line += 1;
                    column = 1;
                }

                // Comments
                '/' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'/') {
                        // Line comment
                        while chars.peek() != Some(&'\n') && chars.peek().is_some() {
                            chars.next();
                        }
                    } else if chars.peek() == Some(&'*') {
                        // Block comment
                        chars.next();
                        while let Some(&ch) = chars.peek() {
                            chars.next();
                            if ch == '*' && chars.peek() == Some(&'/') {
                                chars.next();
                                break;
                            }
                            if ch == '\n' {
                                line += 1;
                                column = 1;
                            }
                        }
                    } else {
                        tokens.push(Token::new(TokenKind::Slash, "/", line, start_col));
                    }
                }

                // Strings
                '"' | '\'' => {
                    let quote = c;
                    chars.next();
                    let mut s = String::new();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        column += 1;
                        if ch == quote {
                            break;
                        }
                        if ch == '\\' {
                            if let Some(&esc) = chars.peek() {
                                chars.next();
                                column += 1;
                                match esc {
                                    'n' => s.push('\n'),
                                    'r' => s.push('\r'),
                                    't' => s.push('\t'),
                                    '\\' => s.push('\\'),
                                    '"' => s.push('"'),
                                    '\'' => s.push('\''),
                                    _ => s.push(esc),
                                }
                            }
                        } else {
                            s.push(ch);
                        }
                    }
                    tokens.push(Token::new(TokenKind::String(s.clone()), &s, line, start_col));
                }

                // Numbers
                '0'..='9' => {
                    let mut num = String::new();
                    let mut is_float = false;
                    let mut is_hex = false;

                    if c == '0' {
                        chars.next();
                        column += 1;
                        num.push('0');
                        if chars.peek() == Some(&'x') || chars.peek() == Some(&'X') {
                            chars.next();
                            column += 1;
                            num.push('x');
                            is_hex = true;
                        }
                    }

                    while let Some(&ch) = chars.peek() {
                        if is_hex {
                            if ch.is_ascii_hexdigit() {
                                num.push(ch);
                                chars.next();
                                column += 1;
                            } else {
                                break;
                            }
                        } else if ch.is_ascii_digit() {
                            num.push(ch);
                            chars.next();
                            column += 1;
                        } else if ch == '.' && !is_float {
                            num.push(ch);
                            chars.next();
                            column += 1;
                            is_float = true;
                        } else {
                            break;
                        }
                    }

                    if is_float {
                        let val: f64 = num.parse().unwrap_or(0.0);
                        tokens.push(Token::new(TokenKind::Float(val), &num, line, start_col));
                    } else if is_hex {
                        let val = i64::from_str_radix(&num[2..], 16).unwrap_or(0);
                        tokens.push(Token::new(TokenKind::Integer(val), &num, line, start_col));
                    } else {
                        let val: i64 = num.parse().unwrap_or(0);
                        tokens.push(Token::new(TokenKind::Integer(val), &num, line, start_col));
                    }
                }

                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            ident.push(ch);
                            chars.next();
                            column += 1;
                        } else {
                            break;
                        }
                    }

                    let kind = match ident.as_str() {
                        "let" => TokenKind::Let,
                        "const" => TokenKind::Const,
                        "fn" => TokenKind::Fn,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        "while" => TokenKind::While,
                        "for" => TokenKind::For,
                        "in" => TokenKind::In,
                        "return" => TokenKind::Return,
                        "break" => TokenKind::Break,
                        "continue" => TokenKind::Continue,
                        "true" => TokenKind::True,
                        "false" => TokenKind::False,
                        "nil" => TokenKind::Nil,
                        "and" => TokenKind::And,
                        "or" => TokenKind::Or,
                        "not" => TokenKind::Not,
                        _ => TokenKind::Identifier(ident.clone()),
                    };

                    tokens.push(Token::new(kind, &ident, line, start_col));
                }

                // Operators and punctuation
                '+' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::PlusAssign, "+=", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Plus, "+", line, start_col));
                    }
                }
                '-' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::MinusAssign, "-=", line, start_col));
                    } else if chars.peek() == Some(&'>') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::Arrow, "->", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Minus, "-", line, start_col));
                    }
                }
                '*' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::StarAssign, "*=", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Star, "*", line, start_col));
                    }
                }
                '%' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::Percent, "%", line, start_col));
                }
                '=' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::EqualEqual, "==", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Equal, "=", line, start_col));
                    }
                }
                '!' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::BangEqual, "!=", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Bang, "!", line, start_col));
                    }
                }
                '<' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::LessEqual, "<=", line, start_col));
                    } else if chars.peek() == Some(&'<') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::ShiftLeft, "<<", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Less, "<", line, start_col));
                    }
                }
                '>' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::GreaterEqual, ">=", line, start_col));
                    } else if chars.peek() == Some(&'>') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::ShiftRight, ">>", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Greater, ">", line, start_col));
                    }
                }
                '&' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::AndAnd, "&&", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Ampersand, "&", line, start_col));
                    }
                }
                '|' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::PipePipe, "||", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Pipe, "|", line, start_col));
                    }
                }
                '^' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::Caret, "^", line, start_col));
                }
                '~' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::Tilde, "~", line, start_col));
                }
                '(' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::LeftParen, "(", line, start_col));
                }
                ')' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::RightParen, ")", line, start_col));
                }
                '{' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::LeftBrace, "{", line, start_col));
                }
                '}' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::RightBrace, "}", line, start_col));
                }
                '[' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::LeftBracket, "[", line, start_col));
                }
                ']' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::RightBracket, "]", line, start_col));
                }
                ',' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::Comma, ",", line, start_col));
                }
                '.' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&'.') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::DotDot, "..", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Dot, ".", line, start_col));
                    }
                }
                ':' => {
                    chars.next();
                    column += 1;
                    if chars.peek() == Some(&':') {
                        chars.next();
                        column += 1;
                        tokens.push(Token::new(TokenKind::ColonColon, "::", line, start_col));
                    } else {
                        tokens.push(Token::new(TokenKind::Colon, ":", line, start_col));
                    }
                }
                ';' => {
                    chars.next();
                    column += 1;
                    tokens.push(Token::new(TokenKind::Semicolon, ";", line, start_col));
                }
                _ => {
                    return Err(CompileError::UnexpectedCharacter(c, line, column));
                }
            }
        }

        tokens.push(Token::new(TokenKind::Eof, "", line, column));
        Ok(tokens)
    }

    /// Parse tokens into AST
    fn parse(&self, tokens: &[Token]) -> Result<Ast, CompileError> {
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Generate bytecode from AST
    fn generate(&self, ast: &Ast) -> Result<CompiledScript, CompileError> {
        let mut generator = CodeGenerator::new();
        generator.generate(ast)
    }

    /// Optimize bytecode
    fn optimize(&self, script: CompiledScript) -> Result<CompiledScript, CompileError> {
        let mut optimizer = Optimizer::new(self.optimization_level);
        optimizer.optimize(script)
    }
}

impl Default for ScriptCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Token representation
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: &str, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.to_string(),
            line,
            column,
        }
    }
}

/// Token kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Identifier(String),
    True,
    False,
    Nil,

    // Keywords
    Let,
    Const,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Continue,
    And,
    Or,
    Not,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    PlusAssign,
    MinusAssign,
    StarAssign,
    EqualEqual,
    BangEqual,
    Bang,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    AndAnd,
    PipePipe,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    ShiftLeft,
    ShiftRight,
    Arrow,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    DotDot,
    Colon,
    ColonColon,
    Semicolon,

    // End of file
    Eof,
}

/// Abstract Syntax Tree
#[derive(Debug, Clone)]
pub struct Ast {
    pub statements: Vec<Statement>,
}

impl Ast {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    Let { name: String, value: Option<Expression>, type_hint: Option<String> },
    Const { name: String, value: Expression },
    Expression(Expression),
    If { condition: Expression, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    While { condition: Expression, body: Vec<Statement> },
    For { var: String, iterable: Expression, body: Vec<Statement> },
    Function { name: String, params: Vec<(String, Option<String>)>, body: Vec<Statement>, return_type: Option<String> },
    Return(Option<Expression>),
    Break,
    Continue,
    Block(Vec<Statement>),
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary { left: Box<Expression>, op: BinaryOp, right: Box<Expression> },
    Unary { op: UnaryOp, operand: Box<Expression> },
    Call { callee: Box<Expression>, args: Vec<Expression> },
    Index { object: Box<Expression>, index: Box<Expression> },
    Member { object: Box<Expression>, member: String },
    Array(Vec<Expression>),
    Table(Vec<(Expression, Expression)>),
    Assign { target: Box<Expression>, value: Box<Expression> },
    Lambda { params: Vec<String>, body: Box<Expression> },
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Nil,
}

/// Binary operators
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
    Range,
}

/// Unary operators
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg, Not, BitNot,
}

/// Parser for script language
struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(&mut self) -> Result<Ast, CompileError> {
        let mut ast = Ast::new();

        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            ast.statements.push(stmt);
        }

        Ok(ast)
    }

    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        match &self.peek().kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::Const => self.parse_const(),
            TokenKind::Fn => self.parse_function(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => {
                self.advance();
                self.consume_semicolon()?;
                Ok(Statement::Break)
            }
            TokenKind::Continue => {
                self.advance();
                self.consume_semicolon()?;
                Ok(Statement::Continue)
            }
            TokenKind::LeftBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expression()?;
                self.consume_semicolon()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'let'
        
        let name = self.expect_identifier()?;
        
        let type_hint = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        let value = if self.check(&TokenKind::Equal) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume_semicolon()?;
        Ok(Statement::Let { name, value, type_hint })
    }

    fn parse_const(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'const'
        let name = self.expect_identifier()?;
        self.expect(&TokenKind::Equal)?;
        let value = self.parse_expression()?;
        self.consume_semicolon()?;
        Ok(Statement::Const { name, value })
    }

    fn parse_function(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'fn'
        let name = self.expect_identifier()?;
        
        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RightParen)?;

        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(&TokenKind::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::Function { name, params, body, return_type })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Option<String>)>, CompileError> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let name = self.expect_identifier()?;
                let type_hint = if self.check(&TokenKind::Colon) {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                params.push((name, type_hint));

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(params)
    }

    fn parse_if(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        
        self.expect(&TokenKind::LeftBrace)?;
        let then_branch = self.parse_block_body()?;
        self.expect(&TokenKind::RightBrace)?;

        let else_branch = if self.check(&TokenKind::Else) {
            self.advance();
            if self.check(&TokenKind::If) {
                Some(vec![self.parse_if()?])
            } else {
                self.expect(&TokenKind::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&TokenKind::RightBrace)?;
                Some(body)
            }
        } else {
            None
        };

        Ok(Statement::If { condition, then_branch, else_branch })
    }

    fn parse_while(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        
        self.expect(&TokenKind::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'for'
        let var = self.expect_identifier()?;
        self.expect(&TokenKind::In)?;
        let iterable = self.parse_expression()?;
        
        self.expect(&TokenKind::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::For { var, iterable, body })
    }

    fn parse_return(&mut self) -> Result<Statement, CompileError> {
        self.advance(); // consume 'return'
        
        let value = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume_semicolon()?;
        Ok(Statement::Return(value))
    }

    fn parse_block(&mut self) -> Result<Statement, CompileError> {
        self.expect(&TokenKind::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&TokenKind::RightBrace)?;
        Ok(Statement::Block(body))
    }

    fn parse_block_body(&mut self) -> Result<Vec<Statement>, CompileError> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_expression(&mut self) -> Result<Expression, CompileError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, CompileError> {
        let expr = self.parse_or()?;

        if self.check(&TokenKind::Equal) {
            self.advance();
            let value = self.parse_assignment()?;
            return Ok(Expression::Assign {
                target: Box::new(expr),
                value: Box::new(value),
            });
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_and()?;

        while self.check(&TokenKind::Or) || self.check(&TokenKind::PipePipe) {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::And) || self.check(&TokenKind::AndAnd) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::EqualEqual => BinaryOp::Eq,
                TokenKind::BangEqual => BinaryOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_term()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Less => BinaryOp::Lt,
                TokenKind::LessEqual => BinaryOp::Le,
                TokenKind::Greater => BinaryOp::Gt,
                TokenKind::GreaterEqual => BinaryOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_factor()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, CompileError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, CompileError> {
        let op = match self.peek().kind {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Bang | TokenKind::Not => Some(UnaryOp::Not),
            TokenKind::Tilde => Some(UnaryOp::BitNot),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary {
                op,
                operand: Box::new(operand),
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, CompileError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LeftParen) {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&TokenKind::RightParen)?;
                expr = Expression::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.check(&TokenKind::Dot) {
                self.advance();
                let member = self.expect_identifier()?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    member,
                };
            } else if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>, CompileError> {
        let mut args = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression, CompileError> {
        let token = self.peek().clone();

        match &token.kind {
            TokenKind::Integer(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Integer(*n)))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Float(*n)))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(Expression::Literal(Literal::String(s.clone())))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            TokenKind::Nil => {
                self.advance();
                Ok(Expression::Literal(Literal::Nil))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name.clone()))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RightBracket)?;
                Ok(Expression::Array(elements))
            }
            _ => Err(CompileError::UnexpectedToken(token.clone())),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<&Token, CompileError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(CompileError::ExpectedToken(format!("{:?}", kind), self.peek().clone()))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, CompileError> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(CompileError::ExpectedIdentifier(self.peek().clone()))
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), CompileError> {
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        Ok(())
    }
}

/// Code generator
struct CodeGenerator {
    bytecode: Vec<Instruction>,
    constants: Vec<ScriptValue>,
    locals: HashMap<String, usize>,
}

impl CodeGenerator {
    fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
            locals: HashMap::new(),
        }
    }

    fn generate(&mut self, ast: &Ast) -> Result<CompiledScript, CompileError> {
        for stmt in &ast.statements {
            self.compile_statement(stmt)?;
        }

        Ok(CompiledScript {
            bytecode: self.bytecode.clone(),
            constants: self.constants.clone(),
            source_map: Vec::new(),
        })
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Let { name, value, .. } => {
                let slot = self.locals.len();
                self.locals.insert(name.clone(), slot);
                
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                } else {
                    self.emit(Instruction::LoadNil);
                }
                self.emit(Instruction::SetLocal(slot));
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Instruction::Pop);
            }
            Statement::Return(value) => {
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                } else {
                    self.emit(Instruction::LoadNil);
                }
                self.emit(Instruction::Return);
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.compile_expression(condition)?;
                let jump_if_false = self.emit_jump(Instruction::JumpIfFalse(0));
                
                for s in then_branch {
                    self.compile_statement(s)?;
                }

                if let Some(else_body) = else_branch {
                    let jump_over = self.emit_jump(Instruction::Jump(0));
                    self.patch_jump(jump_if_false);
                    
                    for s in else_body {
                        self.compile_statement(s)?;
                    }
                    self.patch_jump(jump_over);
                } else {
                    self.patch_jump(jump_if_false);
                }
            }
            Statement::While { condition, body } => {
                let loop_start = self.bytecode.len();
                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump(Instruction::JumpIfFalse(0));
                
                for s in body {
                    self.compile_statement(s)?;
                }
                
                self.emit(Instruction::Jump(loop_start));
                self.patch_jump(exit_jump);
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), CompileError> {
        match expr {
            Expression::Literal(lit) => {
                let idx = self.add_constant(match lit {
                    Literal::Integer(n) => ScriptValue::Integer(*n),
                    Literal::Float(n) => ScriptValue::Float(*n),
                    Literal::String(s) => ScriptValue::String(s.clone()),
                    Literal::Boolean(b) => ScriptValue::Boolean(*b),
                    Literal::Nil => ScriptValue::Nil,
                });
                self.emit(Instruction::LoadConst(idx));
            }
            Expression::Identifier(name) => {
                if let Some(&slot) = self.locals.get(name) {
                    self.emit(Instruction::GetLocal(slot));
                } else {
                    self.emit(Instruction::GetGlobal(name.clone()));
                }
            }
            Expression::Binary { left, op, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                let instr = match op {
                    BinaryOp::Add => Instruction::Add,
                    BinaryOp::Sub => Instruction::Sub,
                    BinaryOp::Mul => Instruction::Mul,
                    BinaryOp::Div => Instruction::Div,
                    BinaryOp::Mod => Instruction::Mod,
                    BinaryOp::Eq => Instruction::Eq,
                    BinaryOp::Ne => Instruction::Ne,
                    BinaryOp::Lt => Instruction::Lt,
                    BinaryOp::Le => Instruction::Le,
                    BinaryOp::Gt => Instruction::Gt,
                    BinaryOp::Ge => Instruction::Ge,
                    BinaryOp::And => Instruction::And,
                    BinaryOp::Or => Instruction::Or,
                    BinaryOp::BitAnd => Instruction::BitAnd,
                    BinaryOp::BitOr => Instruction::BitOr,
                    BinaryOp::BitXor => Instruction::BitXor,
                    BinaryOp::Shl => Instruction::Shl,
                    BinaryOp::Shr => Instruction::Shr,
                    BinaryOp::Range => Instruction::Range,
                };
                self.emit(instr);
            }
            Expression::Unary { op, operand } => {
                self.compile_expression(operand)?;
                let instr = match op {
                    UnaryOp::Neg => Instruction::Neg,
                    UnaryOp::Not => Instruction::Not,
                    UnaryOp::BitNot => Instruction::BitNot,
                };
                self.emit(instr);
            }
            Expression::Call { callee, args } => {
                for arg in args {
                    self.compile_expression(arg)?;
                }
                self.compile_expression(callee)?;
                self.emit(Instruction::Call(args.len()));
            }
            Expression::Array(elements) => {
                for elem in elements {
                    self.compile_expression(elem)?;
                }
                self.emit(Instruction::NewArray(elements.len()));
            }
            Expression::Index { object, index } => {
                self.compile_expression(object)?;
                self.compile_expression(index)?;
                self.emit(Instruction::GetIndex);
            }
            Expression::Member { object, member } => {
                self.compile_expression(object)?;
                self.emit(Instruction::GetMember(member.clone()));
            }
            Expression::Assign { target, value } => {
                self.compile_expression(value)?;
                match target.as_ref() {
                    Expression::Identifier(name) => {
                        if let Some(&slot) = self.locals.get(name) {
                            self.emit(Instruction::SetLocal(slot));
                        } else {
                            self.emit(Instruction::SetGlobal(name.clone()));
                        }
                    }
                    _ => return Err(CompileError::InvalidAssignment),
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn emit(&mut self, instr: Instruction) {
        self.bytecode.push(instr);
    }

    fn emit_jump(&mut self, instr: Instruction) -> usize {
        self.bytecode.push(instr);
        self.bytecode.len() - 1
    }

    fn patch_jump(&mut self, idx: usize) {
        let target = self.bytecode.len();
        match &mut self.bytecode[idx] {
            Instruction::Jump(t) | Instruction::JumpIfFalse(t) => *t = target,
            _ => {}
        }
    }

    fn add_constant(&mut self, value: ScriptValue) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}

/// Bytecode optimizer
struct Optimizer {
    level: u8,
}

impl Optimizer {
    fn new(level: u8) -> Self {
        Self { level }
    }

    fn optimize(&mut self, script: CompiledScript) -> Result<CompiledScript, CompileError> {
        // Simple constant folding for level 1+
        if self.level >= 1 {
            // TODO: Implement optimizations
        }
        Ok(script)
    }
}

/// Bytecode instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(usize),
    LoadNil,
    LoadTrue,
    LoadFalse,
    GetLocal(usize),
    SetLocal(usize),
    GetGlobal(String),
    SetGlobal(String),
    GetIndex,
    SetIndex,
    GetMember(String),
    SetMember(String),
    Pop,
    Dup,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    Range,
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Call(usize),
    Return,
    NewArray(usize),
    NewTable(usize),
    Nop,
}

/// Compiled script
#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub bytecode: Vec<Instruction>,
    pub constants: Vec<ScriptValue>,
    pub source_map: Vec<(usize, usize)>,
}

impl CompiledScript {
    pub fn instruction_count(&self) -> usize {
        self.bytecode.len()
    }

    pub fn constant_count(&self) -> usize {
        self.constants.len()
    }
}

/// Compile error types
#[derive(Debug, Clone)]
pub enum CompileError {
    UnexpectedCharacter(char, usize, usize),
    UnexpectedToken(Token),
    ExpectedToken(String, Token),
    ExpectedIdentifier(Token),
    InvalidAssignment,
    TooManyLocals,
    TooManyConstants,
    Custom(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UnexpectedCharacter(c, line, col) => {
                write!(f, "Unexpected character '{}' at {}:{}", c, line, col)
            }
            CompileError::UnexpectedToken(t) => {
                write!(f, "Unexpected token {:?} at {}:{}", t.kind, t.line, t.column)
            }
            CompileError::ExpectedToken(expected, got) => {
                write!(f, "Expected {} but got {:?} at {}:{}", expected, got.kind, got.line, got.column)
            }
            CompileError::ExpectedIdentifier(t) => {
                write!(f, "Expected identifier at {}:{}", t.line, t.column)
            }
            CompileError::InvalidAssignment => write!(f, "Invalid assignment target"),
            CompileError::TooManyLocals => write!(f, "Too many local variables"),
            CompileError::TooManyConstants => write!(f, "Too many constants"),
            CompileError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let compiler = ScriptCompiler::new();
        let tokens = compiler.tokenize("let x = 42;").unwrap();
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_compile_expression() {
        let compiler = ScriptCompiler::new();
        let result = compiler.compile("let x = 1 + 2;");
        assert!(result.is_ok());
    }
}
