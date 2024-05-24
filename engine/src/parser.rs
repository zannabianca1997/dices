use std::rc::Rc;

use peg::str::LineCol;

use crate::{
    expr::{
        Expr::{self, *},
        Statement,
    },
    identifier::IdentStr,
};

peg::parser! {
  grammar grammar() for str {
    /// A unsigned number literal
    rule number() -> i64
        = n:$(['0'..='9']+) {? n.parse().or(Err("Literal too long for i64")) }

    /// An identifier
    rule ident() -> &'input IdentStr
        = i:$(
            (['a'..='z'|'A'..='Z'] / ['_']+ ['0'..='9'|'a'..='z'|'A'..='Z'])
            ['0'..='9'|'a'..='z'|'A'..='Z'|'_']*
        ) {? IdentStr::new(i).ok_or("Invalid identifier") }


    /// Parse an expression
    rule expr() -> Expr
        = precedence!{
            "null"     { Null }
            "true"     { Bool(true) }
            "false"    { Bool(false) }
            n:number() { Number(n) }
            i:ident()  { Reference(i.into()) }

            "[" _ l:(expr() ** (_ "," _)) _ "]" {
                List(l)
            }

            "<|" _
                elems:(
                    (
                        n:ident() _ ":" _ v:expr() {
                            (n.as_ref().into(),v)
                        }
                    ) ** (_ "," _)
                )
            _ "|>" {
                Map(elems.into_iter().collect())
            }

            "(" _ e:expr() _ ")" { e }

            "|" _ p:((i: ident() { Rc::<IdentStr>::from(i) }) ** (_ "," _)) _ "|" _ b:(
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
        = v:ident() _ "=" _ e:expr() {
            Statement::Set(v.into(), e)
        }
        / "let" _ v:ident() e:(_ "=" _ e:expr() {e})? {
            Statement::Let(v.into(), e)
        }
        / e:expr()  { Statement::Expr(e)          }
        / b:scope() { Statement::Scope(b.into())  }
        / ""        { Statement::Expr(Expr::Null) }

    /// Parse a command - a statetement with optional space
    pub rule command() -> Statement
        = _ stm:statement() _ { stm }

    /// Parse whitespace and comments, discarding them
    rule _ -> ()
        = quiet!{
            (
                [' ' | '\t' | '\r' | '\n']
                / line_comment()
                / block_comment()
            )* { () }
        }

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
