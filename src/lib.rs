extern crate clang;
#[macro_use]
extern crate lazy_static;

use clang::{Clang, Entity, EntityKind, EntityVisitResult, Index, Linkage, Parser, SourceError,
            TranslationUnit, Type, TypeKind};
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug)]
pub enum Error {
    Clang(SourceError),
    Io(io::Error),
    String(String),
}

impl From<SourceError> for Error {
    fn from(error: SourceError) -> Self {
        Error::Clang(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::String(error)
    }
}

type Result = std::result::Result<(), Error>;

fn visit_children<F: FnMut(Entity) -> Result>(entity: &Entity,
                                              recurse: EntityVisitResult,
                                              mut f: F) -> Result {
    let mut result: Result = Ok(());
    entity.visit_children(|x, _| {
        match f(x) {
            Ok(()) => recurse,
            err @ Err(_) => {
                result = err;
                EntityVisitResult::Break
            }
        }
    });
    result
}

type Arg<'tu> = (Type<'tu>, String);
type Args<'tu> = Vec<Arg<'tu>>;

fn printf_format(t: &Type) -> &'static str {
    match t.get_kind() {
        TypeKind::Int => "%d",
        TypeKind::Long => "%ld",
        _ => panic!("Unsupported type")
    }
}

fn generate_args(out: &mut Write, args: &Args) -> Result {
    for (arg_type, arg_name) in args {
        write!(out, "    fprintf(stream, \"    {} {} = {};\\n\", {});\n",
               arg_type.get_display_name(), arg_name, printf_format(&arg_type), arg_name)?;
    }
    Ok(())
}

fn is_global(entity: &Entity) -> bool {
    entity.get_kind() == EntityKind::VarDecl && entity.get_linkage() == Some(Linkage::External)
}

fn generate_globals(out: &mut Write, function: &Entity) -> Result {
    visit_children(function, EntityVisitResult::Recurse, |entity| {
        if entity.get_kind() == EntityKind::DeclRefExpr {
            match entity.get_reference() {
                Some(def) =>
                    if is_global(&def) {
                        let var_name = def.get_name().expect("Global without a name");
                        let var_type = def.get_type().expect("Global without a type");
                        write!(out, "    fprintf(stream, \"    {} = {};\\n\", {});\n",
                               var_name, printf_format(&var_type), var_name)?;
                    }
                None => {}
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn generate_call(out: &mut Write, function_name: &str, args: &Args) -> Result {
    write!(out, "    fprintf(stream, \"    return {}(", function_name)?;
    let mut first = true;
    for (_, arg_name) in args {
        if first {
            first = false;
        } else {
            write!(out, ", ")?;
        }
        write!(out, "{}", arg_name)?;
    }
    write!(out, ");\\n\");\n")?;
    Ok(())
}

fn generate_function(out: &mut Write, function: Entity) -> Result {
    let function_name = function.get_name().expect("Function without a name");
    write!(out, "static inline void snapshot_{}(FILE *stream", function_name)?;
    let raw_args = function.get_arguments().expect("Function without arguments");
    let args: Args = raw_args.iter().map(|arg| {
        let arg_type = arg.get_type().expect("Argument without a type");
        let arg_name = arg.get_name().expect("Argument without a name");
        (arg_type, arg_name)
    }).collect();
    for (arg_type, arg_name) in &args {
        write!(out, ", {} {}", arg_type.get_display_name(), arg_name)?;
    }
    write!(out, ") {{\n")?;
    write!(out, "    static int counter = 0;\n")?;
    let ret = function.get_result_type().expect("Function without result type");
    write!(out, "    fprintf(stream, \"static inline {} replay_{}_%d(void) {{\\n\", ++counter);\n",
           ret.get_display_name(), function_name)?;
    generate_args(out, &args)?;
    generate_globals(out, &function)?;
    generate_call(out, &function_name, &args)?;
    write!(out, "    fprintf(stream, \"}}\\n\");\n")?;
    write!(out, "}}\n")?;
    Ok(())
}

lazy_static! {
    static ref CLANG: Mutex<Clang> = Mutex::new(Clang::new().expect("Clang::new() failed"));
}

pub fn generate(out: &mut Write, path: &Path) -> Result {
    write!(out, "#include <stdio.h>\n\n")?;
    let clang = CLANG.lock().unwrap();
    let index: Index = Index::new(&clang, false, true);
    let parser: Parser = index.parser(path);
    let unit: TranslationUnit = parser.parse()?;
    let unit_entity: Entity = unit.get_entity();
    visit_children(&unit_entity, EntityVisitResult::Continue, |unit_child: Entity| {
        match unit_child.get_kind() {
            EntityKind::FunctionDecl => {
                if unit_child.get_definition() == Some(unit_child) {
                    generate_function(out, unit_child)?;
                }
            }
            _ => {}
        }
        Ok(())
    })
}
