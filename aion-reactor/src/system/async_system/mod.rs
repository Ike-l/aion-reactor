use crate::{id::Id, injection::injection_trait::Injection, memory::{Memory, errors::ReservationError, program_memory_map::inner_program_memory_map::Key}, system::{FunctionSystem, system_metadata::Source, system_result::SystemResult}};

pub mod into_async_system;

use std::{pin::Pin, sync::Arc};

pub type StoredAsyncSystem = Box<dyn AsyncSystem>;

pub trait AsyncSystem: Send + Sync {
    fn run<'a>(
        &'a mut self,
        memory: Arc<Memory>,
        program_id: Option<Id>, 
        source: Option<Source>, 
        key: Option<Key>
    ) -> Pin<Box<dyn Future<Output = Option<SystemResult>> + 'a + Send>>;

    fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool>;
    fn ok_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool>;

    fn reserve_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Source, key: Option<&Key>) -> Option<Result<(), ReservationError>>;
}
macro_rules! impl_async_system {
    (
        $($params:ident),*
    ) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<F, Fut, $($params: Injection),*> AsyncSystem for FunctionSystem<($($params,)*), F>
            where 
                Fut: Future<Output = Option<SystemResult>> + Send + 'static,
                F: Send + Sync,
            for <'b> F:
                FnMut($($params),*) -> Fut +
                FnMut($(<$params as Injection>::Item<'b>),*) -> Fut
        {
            fn run<'a>(
                &'a mut self,
                memory: Arc<Memory>,
                program_id: Option<Id>,
                source: Option<Source>,
                key: Option<Key>
            ) -> Pin<Box<dyn Future<Output = Option<SystemResult>> + 'a + Send>> {
                Box::pin(async move {
                    $(
                        let $params = memory.resolve::<$params>(
                            program_id.as_ref(),
                            None,
                            source.as_ref(),
                            key.as_ref()
                        )?.ok()?;
                    )*

                    (self.f)($($params),*).await
                })
            }
        
            fn ok_resources(
                &self,
                memory: &Memory,
                program_id: Option<&Id>,
                source: Option<&Source>,
                key: Option<&Key>
            ) -> Option<bool> {
                Some(true $(&& memory.ok_resources::<$params>(program_id, source, None, key)?)*)
            }

            fn ok_accesses(
                &self,
                memory: &Memory,
                program_id: Option<&Id>,
                source: Option<&Source>,
                key: Option<&Key>
            ) -> Option<bool> {
                Some(true $(&& memory.ok_accesses::<$params>(program_id, source, None, key)?)*)
            }

            fn reserve_accesses(
                &self,
                memory: &Memory,
                program_id: Option<&Id>,
                source: Source,
                key: Option<&Key>
            ) -> Option<Result<(), ReservationError>> {
                let other_memory = Memory::new();
                // simulate reservations together in a separate memory, exclude no resource as an error.
                $( {
                    let result = other_memory.reserve_current_accesses::<$params>(program_id, None, source.clone(), key); 
                    // check if all reservations work and if any fail then return the error
                    match result {
                        None => return None,
                        Some(Err(err)) => return Some(Err(err)),
                        Some(Ok(_)) => {}
                    }
                } )*

                // then if all are ok together try to integrate them atomically
                return match memory.try_integrate_reservations(other_memory, source) {
                    None => Some(Ok(())),
                    Some(err) => Some(Err(err)),
                }
            }
        }
    };
}

macro_rules! impl_all_async_system {
    () => {
        impl_async_system!();
    };

    ($first:ident $(, $rest:ident)*) => {
        impl_async_system!($first $(, $rest)*);
        impl_all_async_system!($($rest),*);
    };
}

impl_all_async_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9);

// use crate::injection::injection_primitives::cloned::Cloned;
// async fn foo(f: Cloned<i32>) -> Option<SystemResult> {
//     todo!()
// }

// use super::async_system::into_async_system::IntoAsyncSystem;
// async fn bar(memory: Arc<Memory>, program_id: Option<Id>, source: Option<Source>) {
//     let mut b = foo.into_system();
//     let c = b.run(memory, program_id, source).await;
// }