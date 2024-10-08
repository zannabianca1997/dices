use std::borrow::Cow;

use either::Either::{Left, Right};
use nunny::NonEmpty;
use peg::{error::ParseError, str::LineCol};
use set::MemberReceiver;

use crate::{
    expression::{bin_ops::BinOp, un_ops::UnOp, *},
    ident::IdentStr,
    value::*,
};

peg::parser! {
    /**
        # `dices` expressions

        This is the full grammar for a `dices` expression.
    */
    pub grammar expression() for str {

        rule expr<InjectedIntrisic>() -> Expression<InjectedIntrisic>
            = precedence!{
                receiver:receiver() _ "=" _ value:@ { ExpressionSet{ receiver, value: Box::new(value) }.into()}
                --
                "|" _ p:( ident()  ** ( _ "," _ ) ) _ "|" _ body:@ {
                    ExpressionClosure::new(p.into_iter().map(|p| p.to_owned()).collect(), body).into()
                }
                --
                a:(@) _ "+" _ b:@ { ExpressionBinOp::new(BinOp::Add, a,b).into() }
                a:(@) _ "-" _ b:@ { ExpressionBinOp::new(BinOp::Sub, a,b).into() }
                --
                a:(@) _ "~" _ b:@ { ExpressionBinOp::new(BinOp::Join, a,b).into() }
                --
                a:(@) _ "*" _ b:@ { ExpressionBinOp::new(BinOp::Mult, a,b).into() }
                a:(@) _ "/" _ b:@ { ExpressionBinOp::new(BinOp::Div, a,b).into() }
                a:(@) _ "%" _ b:@ { ExpressionBinOp::new(BinOp::Rem, a,b).into() }
                --
                a:(@) _ "^" _ b:@  { ExpressionBinOp::new(BinOp::Repeat, a,b).into() }
                a:(@) _ "kh" !ident() _ b:@ { ExpressionBinOp::new(BinOp::KeepHigh, a,b).into() }
                a:(@) _ "kl" !ident() _ b:@ { ExpressionBinOp::new(BinOp::KeepLow, a,b).into() }
                a:(@) _ "rh" !ident() _ b:@ { ExpressionBinOp::new(BinOp::RemoveHigh, a,b).into() }
                a:(@) _ "rl" !ident() _ b:@ { ExpressionBinOp::new(BinOp::RemoveLow, a,b).into() }
                 --
                "+" _ a:@ { ExpressionUnOp::new(UnOp::Plus, a).into() }
                "-" _ a:@ { ExpressionUnOp::new(UnOp::Neg, a).into() }
                --
                "d" !ident() _ f:@ { ExpressionUnOp::new(UnOp::Dice, f).into() }
                n:@ _ "d" !ident() _ f:(@) { ExpressionBinOp::new(BinOp::Repeat, ExpressionUnOp::new(UnOp::Dice, f).into(), n).into() }
                --
                f:@ _ "(" _ p:(expr() ** (_ "," _)) _ ")" {
                    ExpressionCall::new(f,p.into_boxed_slice()).into()
                }
                accessed:@ _ "[" _ index:expr() _ "]" {
                    ExpressionMemberAccess { accessed: Box::new(accessed), index: Box::new(index) }.into()
                }
                accessed:@ _ "." _ index:(
                    i:ident()      { Expression::Const(Value::String((&**i).into())) }
                    / s: string()  { Expression::Const(s.into()) }
                    / n: number()  { Expression::Const(n.into()) }
                ) {
                    ExpressionMemberAccess { accessed: Box::new(accessed), index: Box::new(index) }.into()
                }
                --
                v:null()      { Expression::Const(v.into()) }
                v:boolean()   { Expression::Const(v.into()) }
                v:number()    { Expression::Const(v.into()) }
                v:string()    { Expression::Const(v.into()) }

                name:ident()     { ExpressionRef { name:name.to_owned() }.into() }

                "[" _ l:(expr() ** (_ "," _)) _ "]" {
                    ExpressionList::from_iter(l).into()
                }

                "<|" _
                    elems:(
                        (
                            k:ident_or_quoted_string() _ ":" _ v:expr() {
                                (ValueString::from(k.into_owned().into_boxed_str()),v)
                            }
                        ) ** (_ "," _)
                    )
                _ "|>" {
                    ExpressionMap::from_iter(elems).into()
                }

                "(" _ e:expr() _ ")" { e }

                "{" inner:scope_inner() "}" { Expression::Scope(inner.into()) }
            }
            / expected!("expression")

        // -- LHS
        rule receiver<InjectedIntrisic>() -> Receiver<InjectedIntrisic>
         = "_"               { Receiver::Ignore }
         / "let" _ i:ident() { Receiver::Let(i.to_owned()) }
         / i:ident() indices:(
            _ "." _ e:(
                i:ident()    { Value::String((**i).into())}
                / s:string() { Value::String(s) }
                / n:number() { Value::Number(n) }
            ) { Expression::Const(e) }
            /  _ "[" _ e:expr() _ "]" { e }
         ) *        { Receiver::Set(MemberReceiver::new(i.to_owned(), indices)) }

        // --- SCALARS ---

        /// A null value
        rule null() -> ValueNull
        = "null"  { ValueNull }

        /// A boolean value
        rule boolean() -> ValueBool
            = "true"  { ValueBool::TRUE  }
            / "false" { ValueBool::FALSE }

        /// An unsigned number
        rule number() -> ValueNumber
            = n:$(['0'..='9']+) {? n.parse().or(Err("number")) }

        /// A quoted string value
        rule string() -> ValueString
            = s:quoted_string() { ValueString::from(s.into_owned().into_boxed_str()) }

        // --- STRING QUOTING AND ESCAPING ---

        /// The inner part of a string literal
        rule escaped_string_inner() -> std::borrow::Cow<'input,str>
            = parts: (
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
                        / "u{" hex: $(['a'..='f'|'A'..='F'|'0'..='9']*<1,6>) "}" {hex}) {?
                                char::from_u32(u32::from_str_radix(hex, 16).unwrap())
                                .ok_or("unicode codepoint")
                        }
                        / expected!("escape code")

                ) {Right(escape)}
                )*  {
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

        /// A string literal
        rule quoted_string() -> std::borrow::Cow<'input,str>
            = "\"" s:escaped_string_inner() "\"" { s }


        /// An identifier
        rule ident() -> &'input IdentStr
            = i:$(
                (['a'..='z'|'A'..='Z'] / ['_']+ ['0'..='9'|'a'..='z'|'A'..='Z'])
                ['0'..='9'|'a'..='z'|'A'..='Z'|'_']*
            ) {? IdentStr::new(i).ok_or("identifier") }


        /// Either a identifier or a wuoted string literal
        rule ident_or_quoted_string() -> std::borrow::Cow<'input,str>
            = i: ident()         { Cow::Borrowed(&**i) }
            / s: quoted_string() { s }

        // --- Inner of a scope `{}`. Also the content of a file
        pub rule scope_inner<InjectedIntrisic>() -> Box<NonEmpty<[Expression<InjectedIntrisic>]>>
            = _ exprs: ( e:expr() {e} / { Value::Null(ValueNull).into() } ) ** (_ ";" _) _ {
                exprs.into_boxed_slice()
                    .try_into()
                    .unwrap_or_else(|_| nunny::vec![Value::Null(ValueNull).into()].into())
            }


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

pub type Error = ParseError<LineCol>;

pub fn parse_file<InjectedIntrisic>(
    src: &str,
) -> Result<Box<NonEmpty<[Expression<InjectedIntrisic>]>>, Error> {
    expression::scope_inner(src)
}
