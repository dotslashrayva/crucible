use crate::ast::*;
use crate::token::Token;

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// Main parse function that starts the parsing process
pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        return Parser { tokens, current: 0 };
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.current];
        self.current += 1;
        return token;
    }

    fn expect(&mut self, expected: Token, error_msg: &str) -> Result<(), String> {
        match self.advance() {
            token if token == &expected => Ok(()),
            _ => Err(error_msg.to_string()),
        }
    }

    fn expect_identifier(&mut self, error_msg: &str) -> Result<String, String> {
        match self.advance() {
            Token::Identifier(name) => Ok(name.clone()),
            _ => Err(error_msg.to_string()),
        }
    }

    fn get_precedence(token: &Token) -> Option<u8> {
        match token {
            Token::Star | Token::Slash | Token::Percent => Some(50),
            Token::Plus | Token::Minus => Some(45),
            Token::LessLess | Token::GreaterGreater => Some(40),
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => Some(35),
            Token::EqualEqual | Token::ExclaimEqual => Some(30),
            Token::Ampersand => Some(25),
            Token::Caret => Some(20),
            Token::Pipe => Some(15),
            Token::AmpAmp => Some(10),
            Token::PipePipe => Some(5),
            Token::Question => Some(3),
            Token::Equal
            | Token::PlusEqual
            | Token::MinusEqual
            | Token::StarEqual
            | Token::SlashEqual
            | Token::PercentEqual
            | Token::AmpEqual
            | Token::PipeEqual
            | Token::CaretEqual
            | Token::LessLessEqual
            | Token::GreaterGreaterEqual => Some(1),
            _ => None,
        }
    }

    fn is_binary_op(token: &Token) -> bool {
        Self::get_precedence(token).is_some()
    }

    fn token_to_binary_op(&mut self) -> Result<BinaryOperator, String> {
        match self.advance() {
            Token::Plus => Ok(BinaryOperator::Add),
            Token::Minus => Ok(BinaryOperator::Subtract),
            Token::Star => Ok(BinaryOperator::Multiply),
            Token::Slash => Ok(BinaryOperator::Divide),
            Token::Percent => Ok(BinaryOperator::Modulo),
            Token::Ampersand => Ok(BinaryOperator::BitwiseAnd),
            Token::Pipe => Ok(BinaryOperator::BitwiseOr),
            Token::Caret => Ok(BinaryOperator::BitwiseXor),
            Token::LessLess => Ok(BinaryOperator::LeftShift),
            Token::GreaterGreater => Ok(BinaryOperator::RightShift),
            Token::AmpAmp => Ok(BinaryOperator::LogicalAnd),
            Token::PipePipe => Ok(BinaryOperator::LogicalOr),
            Token::EqualEqual => Ok(BinaryOperator::Equal),
            Token::ExclaimEqual => Ok(BinaryOperator::NotEqual),
            Token::Less => Ok(BinaryOperator::LessThan),
            Token::LessEqual => Ok(BinaryOperator::LessOrEqual),
            Token::Greater => Ok(BinaryOperator::GreaterThan),
            Token::GreaterEqual => Ok(BinaryOperator::GreaterOrEqual),
            _ => Err("Expected binary operator".to_string()),
        }
    }

    fn token_to_unary_op(token: &Token) -> Option<UnaryOperator> {
        match token {
            Token::Tilde => Some(UnaryOperator::Complement),
            Token::Minus => Some(UnaryOperator::Negate),
            Token::Exclaim => Some(UnaryOperator::LogicalNot),
            Token::PlusPlus => Some(UnaryOperator::PrefixIncrement),
            Token::MinusMinus => Some(UnaryOperator::PrefixDecrement),
            _ => None,
        }
    }

    fn compound_to_binop(token: &Token) -> Option<BinaryOperator> {
        match token {
            Token::PlusEqual => Some(BinaryOperator::Add),
            Token::MinusEqual => Some(BinaryOperator::Subtract),
            Token::StarEqual => Some(BinaryOperator::Multiply),
            Token::SlashEqual => Some(BinaryOperator::Divide),
            Token::PercentEqual => Some(BinaryOperator::Modulo),
            Token::AmpEqual => Some(BinaryOperator::BitwiseAnd),
            Token::PipeEqual => Some(BinaryOperator::BitwiseOr),
            Token::CaretEqual => Some(BinaryOperator::BitwiseXor),
            Token::LessLessEqual => Some(BinaryOperator::LeftShift),
            Token::GreaterGreaterEqual => Some(BinaryOperator::RightShift),
            _ => None,
        }
    }
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program, String> {
        let function = self.parse_function()?;
        self.expect(Token::EOF, "Expected end of file")?;
        return Ok(Program { function });
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Int, "Expected 'int' keyword")?;
        let name = self.expect_identifier("Expected function name")?;

        self.expect(Token::OpenParen, "Expected '('")?;
        self.expect(Token::Void, "Expected 'void'")?;
        self.expect(Token::CloseParen, "Expected ')'")?;

        let body = self.parse_block()?;
        return Ok(Function { name, body });
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(Token::OpenBrace, "Expected '{'")?;

        let mut items = Vec::new();

        while self.peek() != &Token::CloseBrace {
            let block_item = self.parse_block_item()?;
            items.push(block_item);
        }

        self.expect(Token::CloseBrace, "Expected '}'")?;

        return Ok(Block { items });
    }

    fn parse_block_item(&mut self) -> Result<BlockItem, String> {
        if self.peek() == &Token::Int {
            Ok(BlockItem::Declaration(self.parse_declaration()?))
        } else {
            Ok(BlockItem::Statement(self.parse_statement()?))
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration, String> {
        self.expect(Token::Int, "Expected 'int' keyword")?;
        let name = self.expect_identifier("Expected variable name")?;

        // optional initializer: "=" <exp>
        let init = if self.peek() == &Token::Equal {
            self.advance();
            Some(self.parse_exp(0)?)
        } else {
            None
        };

        self.expect(Token::Semicolon, "Expected ';'")?;
        return Ok(Declaration { name, init });
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek() {
            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            Token::OpenBrace => self.parse_compound(),
            Token::Break => self.parse_break(),
            Token::Continue => self.parse_continue(),
            Token::While => self.parse_while(),
            Token::Do => self.parse_do_while(),
            Token::For => self.parse_for(),
            Token::Goto => self.parse_goto(),
            Token::Semicolon => {
                self.advance();
                Ok(Statement::Null)
            }
            _ => {
                // Check for labeled statement: <identifier> ":"
                if let Token::Identifier(name) = self.peek() {
                    if self.tokens.get(self.current + 1) == Some(&Token::Colon) {
                        let name = name.clone();

                        self.advance();
                        self.advance();

                        let stmt = self.parse_statement()?;
                        return Ok(Statement::Labeled(name, Box::new(stmt)));
                    }
                }

                // Otherwise it's an expression statement
                let exp = self.parse_exp(0)?;
                self.expect(Token::Semicolon, "Expected ';'")?;
                Ok(Statement::Expression(exp))
            }
        }
    }

    // "return" <exp> ";"
    fn parse_return(&mut self) -> Result<Statement, String> {
        self.advance();
        let exp = self.parse_exp(0)?;
        self.expect(Token::Semicolon, "Expected ';'")?;
        Ok(Statement::Return(exp))
    }

    // "if" "(" <exp> ")" <statement>, Optional: "else" <statement>
    fn parse_if(&mut self) -> Result<Statement, String> {
        self.advance();
        self.expect(Token::OpenParen, "Expected '('")?;
        let condition = self.parse_exp(0)?;
        self.expect(Token::CloseParen, "Expected ')'")?;

        let then_branch = self.parse_statement()?;

        let else_branch = if self.peek() == &Token::Else {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If(condition, Box::new(then_branch), else_branch))
    }

    // "{" <block_item>* "}"
    fn parse_compound(&mut self) -> Result<Statement, String> {
        let block = self.parse_block()?;
        Ok(Statement::Compound(block))
    }

    // "break" ";"
    fn parse_break(&mut self) -> Result<Statement, String> {
        self.advance();
        self.expect(Token::Semicolon, "Expected ';'")?;
        Ok(Statement::Break(String::new()))
    }

    // "continue" ";"
    fn parse_continue(&mut self) -> Result<Statement, String> {
        self.advance();
        self.expect(Token::Semicolon, "Expected ';'")?;
        Ok(Statement::Continue(String::new()))
    }

    // "while" "(" <exp> ")" <statement>
    fn parse_while(&mut self) -> Result<Statement, String> {
        self.advance();
        self.expect(Token::OpenParen, "Expected '('")?;
        let condition = self.parse_exp(0)?;
        self.expect(Token::CloseParen, "Expected ')'")?;
        let body = self.parse_statement()?;

        Ok(Statement::While(condition, Box::new(body), String::new()))
    }

    // "do" <statement> "while" "(" <exp> ")" ";"
    fn parse_do_while(&mut self) -> Result<Statement, String> {
        self.advance();
        let body = self.parse_statement()?;

        self.expect(Token::While, "Expected 'while'")?;
        self.expect(Token::OpenParen, "Expected '('")?;
        let condition = self.parse_exp(0)?;
        self.expect(Token::CloseParen, "Expected ')'")?;
        self.expect(Token::Semicolon, "Expected ';'")?;

        Ok(Statement::DoWhile(Box::new(body), condition, String::new()))
    }

    // "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
    fn parse_for(&mut self) -> Result<Statement, String> {
        self.advance();
        self.expect(Token::OpenParen, "Expected '('")?;

        let init = self.parse_for_init()?;

        // Optional condition
        let condition = if self.peek() == &Token::Semicolon {
            None
        } else {
            Some(self.parse_exp(0)?)
        };
        self.expect(Token::Semicolon, "Expected ';'")?;

        // Optional post expression
        let post = if self.peek() == &Token::CloseParen {
            None
        } else {
            Some(self.parse_exp(0)?)
        };
        self.expect(Token::CloseParen, "Expected ')'")?;

        let body = self.parse_statement()?;
        Ok(Statement::For(
            init,
            condition,
            post,
            Box::new(body),
            String::new(),
        ))
    }

    // Parse the initialize in the for loop
    fn parse_for_init(&mut self) -> Result<ForInit, String> {
        match self.peek() {
            // If we see 'int', it's a declaration (which consumes its own semicolon)
            Token::Int => {
                let decl = self.parse_declaration()?;
                Ok(ForInit::InitDecl(decl))
            }

            // Otherwise it's an optional expression followed by ";"
            Token::Semicolon => {
                self.advance(); // consume ';'
                Ok(ForInit::InitExpr(None))
            }

            _ => {
                let exp = self.parse_exp(0)?;
                self.expect(Token::Semicolon, "Expected ';'")?;
                Ok(ForInit::InitExpr(Some(exp)))
            }
        }
    }

    // "goto" <identifier> ";"
    fn parse_goto(&mut self) -> Result<Statement, String> {
        self.advance();
        let label = self.expect_identifier("Expected label name after 'goto'")?;
        self.expect(Token::Semicolon, "Expected ';'")?;
        Ok(Statement::Goto(label))
    }

    fn parse_exp(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;

        loop {
            let token = self.peek();
            if !Self::is_binary_op(token) {
                break;
            }

            let token_prec = Self::get_precedence(token).unwrap();
            if token_prec < min_prec {
                break;
            }

            // Assignment as right-associative
            if token == &Token::Equal {
                self.advance();
                let right = self.parse_exp(token_prec)?;
                left = Expr::Assignment(Box::new(left), Box::new(right));
            }
            // Ternary
            else if token == &Token::Question {
                self.advance();
                let middle = self.parse_exp(0)?;
                self.expect(Token::Colon, "Expected ':'")?;
                let right = self.parse_exp(token_prec)?;
                left = Expr::Conditional(Box::new(left), Box::new(middle), Box::new(right));
            }
            // Compound Assignment
            else if let Some(binary_op) = Self::compound_to_binop(token) {
                self.advance();
                let right = self.parse_exp(token_prec)?;
                left = Expr::CompoundAssignment(Box::new(left), binary_op, Box::new(right));
            }
            // Binary Expression as left-associative
            else {
                let operator = self.token_to_binary_op()?;
                let right = self.parse_exp(token_prec + 1)?;
                left = Expr::Binary(operator, Box::new(left), Box::new(right));
            }
        }

        return Ok(left);
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        // Unary operators: <op> <factor>
        if let Some(op) = Self::token_to_unary_op(self.peek()) {
            self.advance();
            let inner = self.parse_factor()?;
            return Ok(Expr::Unary(op, Box::new(inner)));
        }

        // Primary expressions
        let mut expr = match self.peek() {
            Token::Constant(_) => self.parse_constant()?,
            Token::Identifier(_) => self.parse_variable()?,
            Token::OpenParen => self.parse_paren_expr()?,
            _ => return Err("Expected number, unary operator, or '('".to_string()),
        };

        // Postfix ++ and --
        loop {
            match self.peek() {
                Token::PlusPlus => {
                    self.advance();
                    expr = Expr::PostfixIncrement(Box::new(expr));
                }
                Token::MinusMinus => {
                    self.advance();
                    expr = Expr::PostfixDecrement(Box::new(expr));
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn parse_constant(&mut self) -> Result<Expr, String> {
        let value = match self.advance() {
            Token::Constant(v) => v.clone(),
            tok => return Err(format!("Expected constant, got {:?}", tok)),
        };

        match value.parse::<i32>() {
            Ok(n) => Ok(Expr::Constant(n)),
            Err(_) => Err(format!("Invalid number: {}", value)),
        }
    }

    fn parse_variable(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Identifier(name) => Ok(Expr::Variable(name.clone())),
            tok => Err(format!("Expected identifier, got {:?}", tok)),
        }
    }

    fn parse_paren_expr(&mut self) -> Result<Expr, String> {
        self.advance(); // consume '('

        let inner = self.parse_exp(0)?;
        self.expect(Token::CloseParen, "Expected ')'")?;
        return Ok(inner);
    }
}
