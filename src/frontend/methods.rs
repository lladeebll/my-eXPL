use crate::{ast::*, frontend::PARSER, symbol::*, type_table::*};
use lrlex::DefaultLexeme;
use lrpar::{Lexeme, NonStreamingLexer, Span};
use std::collections::LinkedList;

pub fn parse_int(
    lexer: &dyn NonStreamingLexer<DefaultLexeme, u32>,
    token: &DefaultLexeme,
) -> Result<u32, (Option<Span>, &'static str)> {
    match lexer.span_str(token.span()).parse::<u32>() {
        Ok(val) => Ok(val),
        Err(_) => Err((Some(token.span()), "Can't parse to u32")),
    }
}

fn insert(
    mut var_list: LinkedList<SymbolBuilder>,
    inner_type: Type,
    lexer: &dyn NonStreamingLexer<DefaultLexeme, u32>,
    s_table: &mut SymbolTable,
    base: i16,
) -> Result<(), (Option<Span>, &'static str)> {
    while let Some(mut var) = var_list.pop_front() {
        var.dtype(inner_type);
        let span = var.get_name();

        s_table
            .insert_builder(var, base, lexer)
            .map_err(|msg| (Some(span), msg))?;
    }
    Ok(())
}

pub fn insert_gst(
    var_list: LinkedList<SymbolBuilder>,
    inner_type: Type,
    lexer: &dyn NonStreamingLexer<DefaultLexeme, u32>,
) -> Result<(), (Option<Span>, &'static str)> {
    let mut parser = PARSER.lock().unwrap();
    insert(var_list, inner_type, lexer, parser.gst(), 4099)
}

pub fn insert_lst(
    var_list: LinkedList<SymbolBuilder>,
    inner_type: Type,
    lexer: &dyn NonStreamingLexer<DefaultLexeme, u32>,
) -> Result<(), (Option<Span>, &'static str)> {
    let mut parser = PARSER.lock().unwrap();
    insert(var_list, inner_type, lexer, parser.lst(), 1)
}

pub fn get_variable(
    lexer: &dyn NonStreamingLexer<DefaultLexeme, u32>,
    token: &DefaultLexeme,
    access: Vec<Box<Tnode>>,
    ref_type: RefType,
) -> Result<Tnode, (Option<Span>, &'static str)> {
    let name = lexer.span_str(token.span());
    let entry = PARSER
        .lock()
        .unwrap()
        .get_var(name)
        .map_err(|msg| (Some(token.span()), msg))?;

    if entry.get_dim() != access.len() as i16 {
        return Err((
            Some(token.span()),
            "Array access dimension does not match the declared dimension",
        ));
    }

    Ok(Tnode::Var {
        span: token.span(),
        symbol: entry,
        ref_type,
        access,
    })
}

pub fn check_access_vec(
    exp: Vec<Box<Tnode>>,
) -> Result<Vec<Box<Tnode>>, (Option<Span>, &'static str)> {
    for e in &exp {
        match e.get_type() {
            Type::Int => {}
            _ => return Err((e.get_span(), "Index should be of type integer")),
        }
    }
    Ok(exp)
}

pub fn insert_args(
    params: LinkedList<(Type, String)>,
    span: Span,
) -> Result<(), (Option<Span>, &'static str)> {
    let mut parser = PARSER.lock().unwrap();

    let mut i = parser.cfn_params().into_iter();
    let mut j = params.iter();
    let mut counter = params.len() as i16 + 2;

    loop {
        match (&i.next(), j.next()) {
            (Some((t1, n1)), Some((t2, n2))) if t1 == t2 && n1 == n2 => {
                parser
                    .lst()
                    .insert_arg(Symbol::Variable {
                        name: n1.to_string(),
                        binding: -1 * counter,
                        dtype: *t1,
                        is_static: false,
                    })
                    .map_err(|msg| (Some(span), msg))?;
                counter -= 1;
            }
            (None, None) => break,
            _ => {
                return Err((
                    Some(span),
                    "Arguments of function don't match with definition",
                ));
            }
        }
    }
    Ok(())
}
