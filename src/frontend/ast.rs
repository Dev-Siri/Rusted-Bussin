#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    // STATEMENTS
    Program(Program),
    VarDeclaration(VarDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    IfStatement(IfStatement),
    /// `ForStatement(init, test, update, body)`
    ForStatement(ForStatement),
    /// TryCatchStatement(body, alternate)
    TryCatchStatement(TryCatchStatement),

    // EXPRESSIONS
    AssignmentExpr(AssignmentExpr),
    MemberExpr(MemberExpr),
    CallExpr(CallExpr),

    // LITERALS
    Property(Property),
    ObjectLiteral(ObjectLiteral),
    NumericLiteral(NumericLiteral),
    Identifier(Identifier),
    StringLiteral(StringLiteral),
    BinaryExpr(BinaryExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<NodeType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclaration {
    pub constant: bool,
    pub identifier: String,
    pub value: Option<Box<NodeType>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub parameters: Vec<String>,
    pub name: String,
    pub body: Vec<NodeType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub test: Box<NodeType>,
    pub body: Vec<NodeType>,
    pub alternate: Option<Vec<NodeType>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub init: Box<NodeType>,
    pub test: Box<NodeType>,
    pub update: Box<NodeType>,
    pub body: Vec<NodeType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryCatchStatement {
    pub body: Vec<NodeType>,
    pub alternate: Vec<NodeType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<NodeType>,
    pub right: Box<NodeType>,
    pub operator: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub args: Vec<NodeType>,
    pub caller: Box<NodeType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpr {
    pub object: Box<NodeType>,
    pub property: Box<NodeType>,
    pub computed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub assign: Box<NodeType>,
    pub value: Box<NodeType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: Option<Box<NodeType>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLiteral {
    pub properties: Vec<NodeType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumericLiteral {
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    pub value: String,
}
