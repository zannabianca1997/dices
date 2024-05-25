use std::{borrow::Cow, rc::Rc};

use either::Either::{Left, Right};
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


        /// A string literal
        rule str_lit() -> std::borrow::Cow<'input,str>
            = ['"'] parts: (
                lit: $( [^ '"' | '\\']+ ) { Left(lit) }
                / "\\" escape:(
                      "n" {'\n'}
                      / "r" {'\r'}
                      / "t" {'\t'}
                      / "0" {'\0'}
                      / "\\" {'\\'}
                      / "\'" {'\''}
                      / "\"" {'"'}
                      / hex:(
                        "x" hex: $(['0'..='7']['a'..='f'|'A'..='F'|'0'..='9']) {hex}
                        / "u{" hex: $(['a'..='f'|'A'..='F'|'0'..='9']*<,6>) "}" {hex}) {?
                             char::from_u32(u32::from_str_radix(hex, 16).unwrap())
                                .ok_or("unicode codepoint")
                        }
                      / expected!("escape code")

                ) {Right(escape)}
             )* ['"'] {
                if let [Left(part)] = &*parts {
                    Cow::Borrowed(*part)
                } else {
                    // some escapes remains. We need to build a string
                    let mut buf = std::string::String::with_capacity(
                        parts.iter().map(|p| match p {
                            Left(s) => s.len(),
                            Right(c) => c.len_utf8(),
                        }).sum()
                    );
                    for p in parts {
                        match p {
                            Left(s) => buf.push_str(s),
                            Right(ch) => buf.push(ch),
                        }
                    }
                    Cow::Owned(buf)
                }
             }


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
            "null"      { Null }
            "true"      { Bool(true) }
            "false"     { Bool(false) }
            n:number()  { Number(n) }
            i:ident()   { Reference(i.into()) }
            s:str_lit() { String(s.into()) }

            "[" _ l:(expr() ** (_ "," _)) _ "]" {
                List(l)
            }

            "<|" _
                elems:(
                    (
                        n:(n:ident() {n.as_ref().into()} / s:str_lit() {s.into()}) _ ":" _ v:expr() {
                            (n,v)
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
