use spin_factor::{Factor, InstanceBuilder, Result, SpinFactors};
use wasmtime_wasi::preview2::{WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

struct WasiFactor;

impl Factor for WasiFactor {
    type InstanceBuilder = WasiFactorBuilder;
    type InstanceData = WasiInstanceData;

    fn add_to_linker<Factors: SpinFactors>(
        linker: &mut wasmtime::component::Linker<Factors::InstanceData>,
        get: fn(&mut Factors::InstanceData) -> &mut Self::InstanceData,
    ) -> Result<()> {
        use wasmtime_wasi::preview2::bindings as wasi_preview2;
        wasi_preview2::clocks::wall_clock::add_to_linker(linker, get)?;
        wasi_preview2::clocks::monotonic_clock::add_to_linker(linker, get)?;
        wasi_preview2::sync_io::filesystem::types::add_to_linker(linker, get)?;
        wasi_preview2::filesystem::preopens::add_to_linker(linker, get)?;
        wasi_preview2::io::error::add_to_linker(linker, get)?;
        wasi_preview2::sync_io::io::poll::add_to_linker(linker, get)?;
        wasi_preview2::sync_io::io::streams::add_to_linker(linker, get)?;
        wasi_preview2::random::random::add_to_linker(linker, get)?;
        wasi_preview2::random::insecure::add_to_linker(linker, get)?;
        wasi_preview2::random::insecure_seed::add_to_linker(linker, get)?;
        wasi_preview2::cli::exit::add_to_linker(linker, get)?;
        wasi_preview2::cli::environment::add_to_linker(linker, get)?;
        wasi_preview2::cli::stdin::add_to_linker(linker, get)?;
        wasi_preview2::cli::stdout::add_to_linker(linker, get)?;
        wasi_preview2::cli::stderr::add_to_linker(linker, get)?;
        wasi_preview2::cli::terminal_input::add_to_linker(linker, get)?;
        wasi_preview2::cli::terminal_output::add_to_linker(linker, get)?;
        wasi_preview2::cli::terminal_stdin::add_to_linker(linker, get)?;
        wasi_preview2::cli::terminal_stdout::add_to_linker(linker, get)?;
        wasi_preview2::cli::terminal_stderr::add_to_linker(linker, get)?;
        wasi_preview2::sockets::tcp::add_to_linker(linker, get)?;
        wasi_preview2::sockets::tcp_create_socket::add_to_linker(linker, get)?;
        wasi_preview2::sockets::udp::add_to_linker(linker, get)?;
        wasi_preview2::sockets::udp_create_socket::add_to_linker(linker, get)?;
        wasi_preview2::sockets::instance_network::add_to_linker(linker, get)?;
        wasi_preview2::sockets::network::add_to_linker(linker, get)?;
        wasi_preview2::sockets::ip_name_lookup::add_to_linker(linker, get)?;
        Ok(())
    }
}

struct WasiFactorBuilder {
    wasi_ctx: WasiCtxBuilder,
}

impl InstanceBuilder<WasiFactor> for WasiFactorBuilder {
    fn prepare<Factors: SpinFactors>(
        _factor: &WasiFactor,
        _ctx: spin_factor::PrepareContext<Factors>,
    ) -> Result<Self> {
        Ok(Self {
            wasi_ctx: WasiCtxBuilder::new(),
        })
    }

    fn build(mut self) -> Result<WasiInstanceData> {
        Ok(WasiInstanceData {
            table: Default::default(),
            ctx: self.wasi_ctx.build(),
        })
    }
}

struct WasiInstanceData {
    table: wasmtime::component::ResourceTable,
    ctx: wasmtime_wasi::preview2::WasiCtx,
}

impl wasmtime_wasi::preview2::WasiView for WasiInstanceData {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut wasmtime_wasi::preview2::WasiCtx {
        &mut self.ctx
    }
}

struct WasiHttpFactor;

trait Host {
    fn newx(&mut self);
}

impl Factor for WasiHttpFactor {
    type InstanceBuilder = ();
    type InstanceData = ();

    fn add_to_linker<Factors: SpinFactors>(
        linker: &mut wasmtime::component::Linker<Factors::InstanceData>,
        _get: fn(&mut Factors::InstanceData) -> &mut Self::InstanceData,
    ) -> Result<()> {
        let get = WasiCtxHack::make_getter::<Factors>();
        use wasmtime_wasi_http::bindings::http as wasi_http;
        wasi_http::outgoing_handler::add_to_linker(linker, get)?;
        wasi_http::types::add_to_linker(linker, get)?;
        Ok(())
    }
}

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
struct WasiCtxHack(WasiInstanceData);

impl WasiCtxHack {
    fn make_getter<Factors: SpinFactors>(
    ) -> impl for<'a> Fn(&mut Factors::InstanceData) -> &mut Self + Copy + 'static {
        let wasi_getter = Factors::data_getter::<WasiFactor>().expect("depends on WasiFactor");
        move |data: &mut _| {
            let wasi_data = wasi_getter(data);
            ref_cast::RefCast::ref_cast_mut(wasi_data)
        }
    }
}

impl WasiView for WasiCtxHack {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        self.0.table()
    }

    fn ctx(&mut self) -> &mut wasmtime_wasi::preview2::WasiCtx {
        self.0.ctx()
    }
}

impl WasiHttpView for WasiCtxHack {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        self.0.table()
    }

    fn ctx(&mut self) -> &mut WasiHttpCtx {
        Box::leak(Box::new(WasiHttpCtx))
    }
}

#[derive(SpinFactors)]
pub struct MyFactors {
    wasi: WasiFactor,
    wasi_http: WasiHttpFactor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let engine = wasmtime::Engine::default();
        let mut linker = wasmtime::component::Linker::new(&engine);
        MyFactors::add_to_linker(&mut linker).unwrap();
        let factors = MyFactors::new(WasiFactor, WasiHttpFactor);
        let data = factors.build_data().unwrap();
        let mut store = wasmtime::Store::new(&engine, data);
        let component = wasmtime::component::Component::new(&engine, b"(component)").unwrap();
        let _instance = linker.instantiate(&mut store, &component).unwrap();
    }
}

fn main() {}
