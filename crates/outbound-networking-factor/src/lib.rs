use std::sync::Arc;

use spin_factors::{Factor, FactorBuilder, Result, SpinFactors};

pub struct OutboundNetworkingFactor;

impl Factor for OutboundNetworkingFactor {
    type Builder = Builder;
    type Data = ();
}

pub struct Builder {
    allowed_hosts: Arc<Vec<String>>,
}

impl Builder {
    pub fn allowed_hosts(&self) -> &Arc<Vec<String>> {
        &self.allowed_hosts
    }
}

impl FactorBuilder<OutboundNetworkingFactor> for Builder {
    fn prepare<Factors: SpinFactors>(
        _factor: &OutboundNetworkingFactor,
        _ctx: spin_factors::PrepareContext<Factors>,
    ) -> Result<Self> {
        // TODO: feed Component info into this method
        Ok(Builder {
            allowed_hosts: Arc::new(vec!["example.com".into()]),
        })
    }

    fn build(self) -> Result<<OutboundNetworkingFactor as Factor>::Data> {
        Ok(())
    }
}
