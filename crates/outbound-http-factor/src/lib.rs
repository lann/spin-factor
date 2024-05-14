use spin_factors::{Factor, Linker, Result, SpinFactors};
use wasi_factor::WasiFactor;
use wasmtime_wasi::{ResourceTable, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

pub struct OutboundHttpFactor;

impl Factor for OutboundHttpFactor {
    type Builder = ();
    type Data = Data;

    fn add_to_linker<Factors: SpinFactors>(
        linker: &mut Linker<Factors>,
        _get: fn(&mut Factors::Data) -> &mut Self::Data,
    ) -> Result<()> {
        let get_wasi_and_outbound_http =
            Factors::data_getter2::<WasiFactor, OutboundHttpFactor>().unwrap();
        let host_getter = lifetime_hint(move |data| {
            let (wasi, outbound_http) = get_wasi_and_outbound_http.get_mut(data);
            HostView {
                ctx: &mut outbound_http.http_ctx,
                table: wasi.table(),
            }
        });
        wasmtime_wasi_http::bindings::http::outgoing_handler::add_to_linker_get_host(
            linker,
            host_getter,
        )
    }
}

// Until closure_lifetime_binder is stabilized this helps Rust
// infer the HRTB lifetime of the closure
fn lifetime_hint<T, F>(f: F) -> F
where
    F: Fn(&mut T) -> HostView,
{
    f
}

pub struct Data {
    http_ctx: WasiHttpCtx,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            http_ctx: WasiHttpCtx::new(),
        }
    }
}

pub struct HostView<'a> {
    ctx: &'a mut WasiHttpCtx,
    table: &'a mut ResourceTable,
}

impl<'a> WasiHttpView for HostView<'a> {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        self.table
    }
}
