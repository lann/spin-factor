use spin_factors::{Factor, FactorBuilder, ModuleLinker, PrepareContext, Result, SpinFactors};
use wasmtime_wasi::{preview1::WasiP1Ctx, WasiCtxBuilder};

pub struct WasiPreview1Factor;

impl Factor for WasiPreview1Factor {
    type Builder = Builder;
    type Data = WasiP1Ctx;

    fn add_to_module_linker<Factors: SpinFactors>(
        linker: &mut ModuleLinker<Factors>,
        get: fn(&mut Factors::Data) -> &mut Self::Data,
    ) -> Result<()> {
        wasmtime_wasi::preview1::add_to_linker_async(linker, get)
    }
}

pub struct Builder {
    wasi_ctx: WasiCtxBuilder,
}

impl FactorBuilder<WasiPreview1Factor> for Builder {
    fn prepare<Factors: SpinFactors>(
        _factor: &WasiPreview1Factor,
        _ctx: PrepareContext<Factors>,
    ) -> Result<Self> {
        Ok(Self {
            wasi_ctx: WasiCtxBuilder::new(),
        })
    }

    fn build(mut self) -> Result<<WasiPreview1Factor as Factor>::Data> {
        Ok(self.wasi_ctx.build_p1())
    }
}
