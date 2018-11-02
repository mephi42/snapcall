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

fn visit_children<'tu, F: FnMut(Entity<'tu>) -> Result>(
    entity: &Entity<'tu>,
    recurse: EntityVisitResult,
    mut f: F,
) -> Result {
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

struct Var<'tu> {
    name: String,
    tpe: Type<'tu>,
}

fn declare_locals(out: &mut Write, locals: &Vec<Var>) -> Result {
    for local in locals {
        write!(out, "    fprintf(stream, \"    {} {};\\n\");\n",
               local.tpe.get_display_name(), local.name)?;
    }
    Ok(())
}

struct Assignment<'tu> {
    lhs: String,
    tpe: Type<'tu>,
    rhs: String,
}

fn assign_all(out: &mut Write, assignments: &Vec<Assignment>) -> Result {
    for a in assignments {
        write!(out, "    fprintf(stream, \"    {} = {};\\n\", {});\n",
               a.lhs, printf_format(&a.tpe), a.rhs)?;
    }
    Ok(())
}

fn printf_format(t: &Type) -> &'static str {
    match t.get_kind() {
        TypeKind::Int => "%d",
        TypeKind::Long => "%ld",
        TypeKind::LongLong => "%lld",
        TypeKind::Float => "%f",
        TypeKind::Double => "%lf",
        TypeKind::UInt => "%u",
        TypeKind::Pointer => "%s",
        _ => panic!("Unsupported type: {:?}", t)
    }
}

fn handle_arg<'tu>(
    locals: &mut Vec<Var<'tu>>,
    assignments: &mut Vec<Assignment<'tu>>,
    arg_type: &Type<'tu>,
    arg_name: &str,
    arg_val: &str,
    need_local: bool,
) {
    if arg_type.get_kind() == TypeKind::Elaborated {
        let val_type = arg_type.get_elaborated_type()
            .expect("Elaborated type without elaboratee");
        return handle_arg(locals, assignments, &val_type, arg_name, arg_val, true);
    }
    if need_local {
        locals.push(Var {
            name: arg_name.clone().to_string(),
            tpe: arg_type.clone(),
        });
    }
    if arg_type.get_kind() == TypeKind::Typedef {
        let val_type = arg_type
            .get_declaration().expect("Typedef without declaration")
            .get_typedef_underlying_type().expect("Typedef without underlying type");
        return handle_arg(locals, assignments, &val_type, arg_name, arg_val, false);
    } else if arg_type.get_kind() == TypeKind::Pointer {
        let val_name = format!("{}_val", &arg_name);
        let val_type = arg_type.get_pointee_type().expect("Pointer without pointee");
        let val = format!("(*{})", &arg_val);
        handle_arg(locals, assignments, &val_type, &val_name, &val, true);
        assignments.push(Assignment {
            lhs: arg_name.clone().to_string(),
            tpe: arg_type.clone(),
            rhs: format!("\"(&{})\"", &val_name),
        });
    } else if arg_type.get_kind() == TypeKind::Record {
        let fields = arg_type.get_fields().expect("Record without fields");
        for field in fields {
            let field_name = field.get_name().expect("Field without a name");
            let field_type = field.get_type().expect("Field without a type");
            let field_expr = format!("{}.{}", &arg_name, &field_name);
            let field_val = format!("{}.{}", &arg_val, &field_name);
            handle_arg(locals, assignments, &field_type, &field_expr, &field_val, false);
        }
    } else {
        assignments.push(Assignment {
            lhs: arg_name.clone().to_string(),
            tpe: arg_type.clone(),
            rhs: arg_val.clone().to_string(),
        });
    }
}

fn handle_args<'tu>(
    locals: &mut Vec<Var<'tu>>,
    assignments: &mut Vec<Assignment<'tu>>,
    args: &Args<'tu>,
) {
    for (arg_type, arg_name) in args {
        handle_arg(locals, assignments, arg_type, arg_name, arg_name, true);
    }
}

fn is_global(entity: &Entity) -> bool {
    entity.get_kind() == EntityKind::VarDecl && entity.get_linkage() == Some(Linkage::External)
}

fn handle_globals<'tu>(assignments: &mut Vec<Assignment<'tu>>, function: &Entity<'tu>) {
    function.visit_children(|entity, _| {
        if entity.get_kind() == EntityKind::DeclRefExpr {
            match entity.get_reference() {
                Some(def) =>
                    if is_global(&def) {
                        let var_name = def.get_name().expect("Global without a name");
                        let var_type = def.get_type().expect("Global without a type");
                        assignments.push(Assignment {
                            lhs: var_name.clone(),
                            tpe: var_type.clone(),
                            rhs: var_name.clone(),
                        });
                    }
                None => {}
            }
        }
        EntityVisitResult::Recurse
    });
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
    let mut locals = Vec::new();
    let mut assignments = Vec::new();
    handle_args(&mut locals, &mut assignments, &args);
    handle_globals(&mut assignments, &function);
    declare_locals(out, &locals)?;
    assign_all(out, &assignments)?;
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
