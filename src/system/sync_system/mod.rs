use crate::{id::Id, injection::injection_trait::Injection, memory::{Memory, errors::ReservationError, program_memory_map::inner_program_memory_map::Key}, system::{FunctionSystem, system_metadata::Source, system_result::SystemResult}};


pub mod into_sync_system;

pub type StoredSyncSystem = Box<dyn SyncSystem>;

pub trait SyncSystem: Send + Sync {
    fn run(
        &mut self,
        memory: &Memory,
        program_id: Option<&Id>, 
        source: Option<&Source>,
        key: Option<&Key>
    ) -> Option<SystemResult>;

    fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool>;
    fn ok_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool>;

    fn check_read_only(&self, source: Option<&Source>) -> bool;

    fn reserve_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Source, key: Option<&Key>) -> Option<Result<(), ReservationError>>;
}

macro_rules! impl_sync_system {
    (
        $($params:ident),*
    ) => {

        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<F, $($params: Injection),*> SyncSystem for FunctionSystem<($($params,)*), F>
            where F: Send + Sync,
            for <'a, 'b> &'a mut F:
                FnMut($($params),*) -> Option<SystemResult> +
                FnMut($(<$params as Injection>::Item<'b>),*) -> Option<SystemResult>
        {
            fn run(
                &mut self,
                memory: &Memory,
                program_id: Option<&Id>,
                source: Option<&Source>,
                key: Option<&Key>
            ) -> Option<SystemResult> {
                fn call_inner<$($params),*>(
                    mut f: impl FnMut($($params),*) -> Option<SystemResult>,
                    $($params: $params),*
                ) -> Option<SystemResult> {
                    f($($params),*)
                }

                $(
                    let $params = memory.resolve::<$params>(
                        program_id,
                        None,
                        source,
                        key
                    )?.ok()?;
                )*

                // (&mut self.f)($($params),*)
                call_inner(&mut self.f, $($params),*)
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

            fn check_read_only(&self, source: Option<&Source>) -> bool {
                true $(&& { 
                    let mut access_map = $params::create_access_map();
                    $params::resolve_accesses(&mut access_map, source, None);
                    access_map.is_read_only()
                 })*
            }

            fn reserve_accesses(
                &self,
                memory: &Memory,
                program_id: Option<&Id>,
                source: Source,
                key: Option<&Key>
            ) -> Option<Result<(), ReservationError>> {
                // println!("Here 1");
                // std::thread::sleep(std::time::Duration::from_secs(1));
                let other_memory = Memory::new();
                // simulate reservations together in a separate memory, exclude no resource as an error.
                $( {
                    let result = other_memory.reserve_current_accesses::<$params>(program_id, None, source.clone(), key); 
                    // check if all reservations work and if any fail then return the error
                    // println!("Result: {:?}", result);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    match result {
                        None => return None,
                        Some(Err(err)) => return Some(Err(err)),
                        Some(Ok(_)) => {}
                    }
                } )*

                // println!("Here");

                // then if all are ok together try to integrate them atomically
                return match memory.try_integrate_reservations(other_memory, source) {
                    None => Some(Ok(())),
                    Some(err) => Some(Err(err)),
                }
            }
        }
    };
}

macro_rules! impl_all_sync_system {
    () => {
        impl_sync_system!();
    };

    ($first:ident $(, $rest:ident)*) => {
        impl_sync_system!($first $(, $rest)*);
        impl_all_sync_system!($($rest),*);
    };
}

impl_all_sync_system!(T1, T2, T3, T4, T5, T6, T7, T8, T9);