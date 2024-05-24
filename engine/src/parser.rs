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
        = n:$(['0'..='9']+) {? n.parse().or(Err("i64")) }

    /// An identifier
    rule ident() -> &'input IdentStr
        = i:$(
            (['a'..='z'|'A'..='Z'] / ['_']+ ['0'..='9'|'a'..='z'|'A'..='Z'])
            ['0'..='9'|'a'..='z'|'A'..='Z'|'_']*
        ) {? IdentStr::new(i).ok_or("identifier") }


    /// Parse an expression
    rule expr() -> Expr
        = precedence!{
            f:@ _ "(" _ p:(expr() ** (_ "," _)) _ ")" {
                Expr::Call { fun: Box::new(f), params: p }
            }
            --
            "|" _ p:((i: ident() { Rc::<IdentStr>::from(i) }) ** (_ "," _)) _ "|" _ e:@ {
                Expr::Function { params: p.into(), body: [Statement::Expr(e)].into() }
            }
            --
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

            "|" _ p:((i: ident() { Rc::<IdentStr>::from(i) }) ** (_ "," _)) _ "|" _ b:scope()
            {
                Expr::Function { params: p.into(), body: b.into() }
            }
        }
        / expected!("expression")

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
        / expected!("statement")

    /// Parse a command - a statetement with optional space
    pub rule command() -> Statement
        = _ stm:statement() _ { stm }

    /// Parse whitespace and comments, discarding them
    rule _ -> ()
        = quiet!{
            (
                [' ' | '\t' | '\r' | '\n']       // Whitespace
                / "//" [^'\n']* (['\n'] / ![_])  // C-style line comment
                / "/*" (!"*/" [_])* "*/"         // C-style block comment
            )* { () }
        }
  }
}

pub fn parse_statement(input: &str) -> Result<Statement, peg::error::ParseError<LineCol>> {
    grammar::command(input)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::expr::{
        Expr::*,
        Statement::{self, *},
    };

    use super::parse_statement;

    fn parse_test(src: &str, res: Statement) {
        match parse_statement(src) {
            Ok(parsed) => assert_eq!(parsed, res),
            Err(err) => panic!("{err:#}"),
        }
    }

    macro_rules! parse_tests {
        (
            $(
            $name:ident : $src:literal => $res:expr
            );* $(;)?
        ) => {
            $(
                #[test]
                fn $name() {
                    parse_test($src, $res)
                }
            )*
        };
    }

    parse_tests! {
        empty: "" => Expr(Null);
        line_comment: "// comment" => Expr(Null);
        block_comment: "/* comment */" => Expr(Null);
        number: "3" => Expr(Number(3));
        null:"null"=> Expr(Null);
        bool_t:"true"=> Expr(Bool(true));
        bool_f:"false"=> Expr(Bool(false));

        list: "[3, 2 ,false , null]" => Expr(List(vec![Number(3), Number(2), Bool(false), Null]));
        list_empty: "[]" => Expr(List(vec![]));
        list_nested: "[[]]" => Expr(List(vec![List(vec![])]));

        map: "<|a:3, b:2, __0:false , o:null|>" => Expr(Map(From::from([("a".into(),Number(3)), ("b".into(),Number(2)), ("__0".into(),Bool(false)), ("o".into(),Null)])));
        map_empty: "<||>" => Expr(Map(From::from([])));

        // TODO: TEST MORE
    }
}
