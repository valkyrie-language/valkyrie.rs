use jvm_backend::{class::JvmFieldRef, JvmInstruction, JvmMethodDescriptor, JvmMethodRef, JvmTypeDescriptor};
use miette::{miette, Result};

use crate::{lir::LirOperand, mir::MirValueRef};

use super::{
    jvm_lowering::FunctionLoweringContext,
    jvm_operation_lowering::{lower_operand, store_required_output},
};

pub(super) fn try_lower_host_bridge_call(
    symbol: &str,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<bool> {
    match symbol {
        "__console_write" => {
            emit_print_stream_call("out", "print", arguments, context, instructions)?;
            return Ok(true);
        }
        "__console_write_line" => {
            emit_print_stream_call("out", "println", arguments, context, instructions)?;
            return Ok(true);
        }
        "__console_error_line" => {
            emit_print_stream_call("err", "println", arguments, context, instructions)?;
            return Ok(true);
        }
        "__console_read" => {
            instructions.push(JvmInstruction::GetStatic(JvmFieldRef {
                owner: "java/lang/System".to_string(),
                name: "in".to_string(),
                descriptor: JvmTypeDescriptor::Object("java/io/InputStream".to_string()),
            }));
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/io/InputStream".to_string(),
                name: "read".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Int),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__system_get_property" => {
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/lang/System".to_string(),
                name: "getProperty".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/lang/String".to_string())],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__path_of" => {
            lower_operand(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/lang/String", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Path".to_string(),
                name: "of".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/lang/String".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
                    ],
                    JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/nio/file/Path".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__file_exists" => {
            emit_path_of_argument(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/nio/file/LinkOption", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "exists".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/LinkOption".to_string()))),
                    ],
                    JvmTypeDescriptor::Int,
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__is_directory" => {
            emit_path_of_argument(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/nio/file/LinkOption", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "isDirectory".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/LinkOption".to_string()))),
                    ],
                    JvmTypeDescriptor::Int,
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__mkdirs" => {
            emit_path_of_argument(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/nio/file/attribute/FileAttribute", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "createDirectories".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/attribute/FileAttribute".to_string()))),
                    ],
                    JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                ),
            }));
            instructions.push(JvmInstruction::Pop);
            instructions.push(JvmInstruction::IConst(1));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__file_read_all_text" => {
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::CheckCast("java/nio/file/Path".to_string()));
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "readString".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/nio/file/Path".to_string())],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__file_write_all_text" => {
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::CheckCast("java/nio/file/Path".to_string()));
            lower_operand(&arguments[1], context, instructions)?;
            emit_empty_object_array("java/nio/file/OpenOption", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "writeString".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Object("java/lang/CharSequence".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/OpenOption".to_string()))),
                    ],
                    JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/nio/file/Path".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            else {
                instructions.push(JvmInstruction::Pop);
            }
            return Ok(true);
        }
        _ => {}
    }

    Ok(false)
}

pub(super) fn is_jvm_host_bridge_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "__console_write"
            | "__console_write_line"
            | "__console_error_line"
            | "__console_read"
            | "__console_read_line"
            | "__system_get_property"
            | "__file_exists"
            | "__mkdirs"
            | "__is_directory"
            | "__path_of"
            | "__file_read_all_text"
            | "__file_write_all_text"
    )
}

fn emit_print_stream_call(
    stream_field: &str,
    method_name: &str,
    arguments: &[LirOperand],
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    let Some(first_argument) = arguments.first()
    else {
        return Err(miette!("JVM host bridge `{method_name}` 至少需要一个参数"));
    };
    instructions.push(JvmInstruction::GetStatic(JvmFieldRef {
        owner: "java/lang/System".to_string(),
        name: stream_field.to_string(),
        descriptor: JvmTypeDescriptor::Object("java/io/PrintStream".to_string()),
    }));
    lower_operand(first_argument, context, instructions)?;
    instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
        owner: "java/io/PrintStream".to_string(),
        name: method_name.to_string(),
        descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".to_string())], JvmTypeDescriptor::Void),
    }));
    Ok(())
}

fn emit_path_of_argument(
    argument: &LirOperand,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    lower_operand(argument, context, instructions)?;
    emit_empty_object_array("java/lang/String", instructions);
    instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
        owner: "java/nio/file/Path".to_string(),
        name: "of".to_string(),
        descriptor: JvmMethodDescriptor::new(
            vec![
                JvmTypeDescriptor::Object("java/lang/String".to_string()),
                JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
            ],
            JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
        ),
    }));
    Ok(())
}

fn emit_empty_object_array(class_name: &str, instructions: &mut Vec<JvmInstruction>) {
    instructions.push(JvmInstruction::IConst(0));
    instructions.push(JvmInstruction::ANewArray(class_name.to_string()));
}
