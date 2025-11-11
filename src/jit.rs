use cranelift::prelude::*;
use cranelift_module::{self, Linkage, Module, ModuleError, default_libcall_names};
use cranelift_jit::{JITBuilder, JITModule};

use crate::{io::{input, output}, vm::BFVM};

pub fn jit_compile(vm: BFVM) -> Result<fn(), ModuleError> {
    let mut builder = JITBuilder::new(default_libcall_names())?;
    builder.symbol("output", output as *const u8);
    builder.symbol("input", input as *const u8);
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();
    let mut fn_builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

    let block = fn_builder.create_block();
    fn_builder.append_block_params_for_function_params(block);
    fn_builder.switch_to_block(block);
    fn_builder.seal_block(block);

    let output_ref = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::I8));
        let func_id = module.declare_function("output", Linkage::Import, &sig)?;
        module.declare_func_in_func(func_id, &mut fn_builder.func)
    };

    let input_ref = {
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I8));
        let func_id = module.declare_function("input", Linkage::Import, &sig)?;
        module.declare_func_in_func(func_id, &mut fn_builder.func)
    };

    // x + y を計算
    let x = fn_builder.block_params(block)[0];
    let y = fn_builder.block_params(block)[1];
    let sum = fn_builder.ins().iadd(x, y);
    fn_builder.ins().return_(&[sum]);

    fn_builder.finalize();

    // コンパイルして実行
    let id = module.declare_function("run", Linkage::Export, &ctx.func.signature)?;
    module.define_function(id, &mut ctx)?;
    module.clear_context(&mut ctx);
    module.finalize_definitions()?;

    let code = module.get_finalized_function(id);
    let func = unsafe { std::mem::transmute::<_, fn()>(code) };

    Ok(func)
}
