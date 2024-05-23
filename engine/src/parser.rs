use peg::str::LineCol;

use crate::expr::Expr::{self, *};
use crate::expr::Statement;
use crate::identifier::DIdentifier;

peg::parser! {
  grammar grammar() for str {
    /// A unsigned number literal
    rule number() -> i64
        = n:$(['0'..='9']+) {? n.parse().or(Err("Literal too long for i64")) }

    /// An identifier
    rule ident() -> DIdentifier
        = i:$(
            (['a'..='z'|'A'..='Z'] / ['_']+ ['0'..='9'|'a'..='z'|'A'..='Z'])
            ['0'..='9'|'a'..='z'|'A'..='Z'|'_']*
        ) {? DIdentifier::new(i).ok_or("Invalid identifier") }


    /// Parse an expression
    rule expr() -> Expr
        = precedence!{
            "null"     { Null }
            "true"     { Bool(true) }
            "false"    { Bool(false) }
            n:number() { Number(n) }
            i:ident()  { Reference(i) }

            "[" _ l:(expr() ** (_ "," _)) _ "]" {
                List(l)
            }

            "{" _
                elems:(
                    (
                        n:ident() _ ":" _ v:expr() {
                            (n.into(),v)
                        }
                    ) ** (_ "," _)
                )
            _ "}" {
                Map(elems.into_iter().collect())
            }

            "(" e:expr() ")" { e }

            "|" _ p:(ident() ** (_ "," _)) _ "|" _ b:(
                e:expr() { vec![Statement::Expr(e)] }
                / scope()
            ) {
                Expr::Function { params: p.into(), body: b.into() }
            }

            f:@ _ "(" _ p:(expr() ** (_ "," _)) _ ")" {
                Expr::Call { fun: Box::new(f), params: p }
            }
        }

    /// Scoped statement
    rule scope() -> Vec<Statement>
        = "{" _ stms:((statement()) ** (_ ";" _)) _ "}" { stms }

    /// Parse any statement
    rule statement() -> Statement
        = e:expr() { Statement::Expr(e) }
        / ""       { Statement::Expr(Expr::Null) }

    /// Parse a command - a statetement with optional space
    pub rule command() -> Statement
        = _ stm:statement() _ { stm }

    /// Parse whitespace and comments, discarding them
    rule _ -> ()
        = ([' ' | '\t' | '\r' | '\n'] / line_comment() / block_comment())* { () }

    /// C-style line comment
    rule line_comment() -> ()
        = "//" [^'\n']* ['\n']
    /// C-style block comment
    rule block_comment() -> ()
        = "/*" (!"*/" [_])* "*/"
  }
}

pub fn parse_statement(input: &str) -> Result<Statement, peg::error::ParseError<LineCol>> {
    grammar::command(input)
}
