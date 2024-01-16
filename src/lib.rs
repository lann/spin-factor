use std::any::Any;

pub use spin_factor_derive::SpinFactors;

pub use wasmtime;
pub type Error = wasmtime::Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait Factor: Any + Sized {
    type InstanceBuilder: InstanceBuilder<Self>;
    type InstanceData;

    fn add_to_linker<Factors: SpinFactors>(
        linker: &mut wasmtime::component::Linker<Factors::InstanceData>,
        get: fn(&mut Factors::InstanceData) -> &mut Self::InstanceData,
    ) -> Result<()> {
        (_, _) = (linker, get);
        Ok(())
    }

    fn add_to_module_linker<Factors: SpinFactors>(
        linker: &mut wasmtime::Linker<Factors::InstanceData>,
        get: fn(&mut Factors::InstanceData) -> &mut Self::InstanceData,
    ) -> Result<()> {
        (_, _) = (linker, get);
        Ok(())
    }
}

impl<'a, Factors: SpinFactors> PrepareContext<'a, Factors> {
    pub fn builder_mut<T: Factor>(&mut self) -> Result<&mut T::InstanceBuilder> {
        let err_msg = match Factors::builder_mut::<T>(self.builders) {
            Some(Some(builder)) => return Ok(builder),
            Some(None) => "builder not yet prepared",
            None => "no such factor",
        };
        Err(Error::msg(format!(
            "could not get builder for {ty}: {err_msg}",
            ty = std::any::type_name::<T>()
        )))
    }
}

pub trait SpinFactors {
    type InstanceBuilders;
    type InstanceData: Send + 'static;

    #[allow(clippy::type_complexity)]
    fn data_getter<T: Factor>(
    ) -> Option<for<'a> fn(&'a mut Self::InstanceData) -> &'a mut T::InstanceData>;

    fn builder_mut<T: Factor>(
        builders: &mut Self::InstanceBuilders,
    ) -> Option<Option<&mut T::InstanceBuilder>>;
}

pub trait InstanceBuilder<T: Factor>: Sized {
    fn prepare<Factors: SpinFactors>(factor: &T, ctx: PrepareContext<Factors>) -> Result<Self>;

    fn build(self) -> Result<T::InstanceData>;
}

pub struct PrepareContext<'a, Factors: SpinFactors> {
    builders: &'a mut Factors::InstanceBuilders,
    // TODO: component: &'a AppComponent,
}

impl<'a, Factors: SpinFactors> PrepareContext<'a, Factors> {
    #[doc(hidden)]
    pub fn new(builders: &'a mut Factors::InstanceBuilders) -> Self {
        Self { builders }
    }
}

pub type DefaultBuilder = ();

impl<T: Factor> InstanceBuilder<T> for DefaultBuilder
where
    T::InstanceData: Default,
{
    fn prepare<Factors: SpinFactors>(factor: &T, ctx: PrepareContext<Factors>) -> Result<Self> {
        (_, _) = (factor, ctx);
        Ok(())
    }

    fn build(self) -> Result<T::InstanceData> {
        Ok(Default::default())
    }
}
