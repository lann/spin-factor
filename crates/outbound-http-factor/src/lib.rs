use std::sync::Arc;

use outbound_networking_factor::OutboundNetworkingFactor;
use spin_factors::{Factor, FactorBuilder, Result, SpinFactors};
use wasi_factor::WasiFactor;
use wasmtime_wasi::{ResourceTable, WasiView};
use wasmtime_wasi_http::{bindings::http::outgoing_handler::ErrorCode, WasiHttpCtx, WasiHttpView};

pub struct OutboundHttpFactor;

impl Factor for OutboundHttpFactor {
    type Builder = Builder;
    type Data = Data;

    fn init<Factors: SpinFactors>(mut ctx: spin_factors::InitContext<Factors, Self>) -> Result<()> {
        let get_wasi_and_outbound_http =
            Factors::data_getter2::<WasiFactor, OutboundHttpFactor>().unwrap();
        let host_getter = lifetime_hint(move |data| {
            let (wasi, outbound_http) = get_wasi_and_outbound_http.get_mut(data);
            DataRefs {
                outbound_http,
                wasi,
            }
        });
        wasmtime_wasi_http::bindings::http::outgoing_handler::add_to_linker_get_host(
            ctx.linker(),
            host_getter,
        )
    }
}

// Until closure_lifetime_binder is stabilized this helps Rust
// infer the HRTB lifetime of the closure
fn lifetime_hint<T, F>(f: F) -> F
where
    F: Fn(&mut T) -> DataRefs,
{
    f
}

pub struct Builder {
    allowed_hosts: Arc<Vec<String>>,
}

impl FactorBuilder<OutboundHttpFactor> for Builder {
    fn prepare<Factors: SpinFactors>(
        _factor: &OutboundHttpFactor,
        mut ctx: spin_factors::PrepareContext<Factors>,
    ) -> Result<Self> {
        let allowed_hosts = ctx
            .builder_mut::<OutboundNetworkingFactor>()?
            .allowed_hosts()
            .clone();
        Ok(Self { allowed_hosts })
    }

    fn build(self) -> Result<<OutboundHttpFactor as Factor>::Data> {
        Ok(Data {
            http_ctx: WasiHttpCtx::new(),
            allowed_hosts: self.allowed_hosts,
        })
    }
}

pub struct Data {
    http_ctx: WasiHttpCtx,
    allowed_hosts: Arc<Vec<String>>,
}

pub struct DataRefs<'a> {
    outbound_http: &'a mut Data,
    wasi: &'a mut <wasi_factor::WasiFactor as spin_factors::Factor>::Data,
}

impl<'a> WasiHttpView for DataRefs<'a> {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.outbound_http.http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        self.wasi.table()
    }

    fn send_request(
        &mut self,
        request: hyper::Request<wasmtime_wasi_http::body::HyperOutgoingBody>,
        config: wasmtime_wasi_http::types::OutgoingRequestConfig,
    ) -> wasmtime_wasi_http::HttpResult<wasmtime_wasi_http::types::HostFutureIncomingResponse> {
        // TODO: real impl
        let host = request
            .uri()
            .authority()
            .map(|authority| authority.host().to_string())
            .unwrap_or_default();
        if !self.outbound_http.allowed_hosts.contains(&host) {
            return Err(ErrorCode::InternalError(Some("host not allowed".into())).into());
        }
        Ok(wasmtime_wasi_http::types::default_send_request(
            request, config,
        ))
    }
}
