use std::{collections::{HashMap, HashSet}, pin::Pin, sync::{atomic::{AtomicUsize, Ordering}, Arc}, task::{Context, Poll, Waker}};

use threadpool::ThreadPool;

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{access_checked_heap::heap::HeapId, Memory, ResourceId}, state_machine::{kernel_systems::{blocker_manager::blocker::{CurrentBlockers, NextBlockers}, event_manager::event::{CurrentEvents, Event, NextEvents}, processor::{processor_system_registry::ProcessorSystemRegistry, scheduler::{execution_graph::ExecutionGraph, panic_catching_execution_graph::PanicCatchingExecutionGraphs, panic_catching_execution_graph_future::PanicCatchingExecutionGraphsFuture}, tasks::DummyWaker}, KernelSystem}, StateMachine}, system::{stored_system::StoredSystem, system_cell::SystemCell, system_metadata::{Source, SystemMetadata, SystemRegistry}, system_result::{SystemEvent, SystemResult}, system_status::SystemStatus, System}};

pub mod scheduler;
pub mod tasks;
pub mod processor_system_registry;

#[derive(Debug)]
pub struct Processor {
    threadpool: ThreadPool,
    runtime: Arc<tokio::runtime::Runtime>
}

impl Processor {
    // remember weirdly used in kernel/state machine
    pub fn new(threads: usize) -> Self {
        Self {
            threadpool: ThreadPool::new(threads),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap())
        }
    }

    pub fn insert_system2(state_machine: &StateMachine, id: Id, system_metadata: SystemMetadata, system: StoredSystem) -> Option<SystemMetadata> {
        let mut system_registry = state_machine.state.resolve::<Unique<ProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
        Self::insert_system(state_machine, &mut system_registry.0, id, system_metadata, system)
    }


    pub fn insert_system(state_machine: &StateMachine, system_registry: &mut SystemRegistry, id: Id, system_metadata: SystemMetadata, system: StoredSystem) -> Option<SystemMetadata> {
        let resource_id = system_metadata.resource_id();
        state_machine.state.insert(None, Some(resource_id.clone()), None, system);

        system_registry.insert(id, system_metadata)
    }

    pub fn get_systems<'a>(memory: &Memory, system_registry: &'a SystemRegistry) -> HashMap<&'a Id, &'a SystemMetadata> {
        let current_blockers = memory.resolve::<Shared<CurrentBlockers>>(None, None, None, None).unwrap().unwrap();
        let current_events = memory.resolve::<Shared<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
    
        let events = current_events.read().collect::<HashSet<_>>();
    
        system_registry.read()
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|&(id, _)| !current_blockers.blocks(id.clone()))
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|(_, system_metadata)| system_metadata.test(&events))
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|(id, system_metadata)| {
                let resource_id = system_metadata.resource_id();
                let program_id = system_metadata.program_id();
                let key = system_metadata.key();
                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(&resource_id), None, None).unwrap().unwrap();
                system.ok_resources(&memory, program_id.as_ref(), Some(&Source((*id).clone())), key.as_ref()).is_some_and(|t| t)
            })
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|(id, system_metadata)| {
                let resource_id = system_metadata.resource_id();
                let program_id = system_metadata.program_id();
                let key = system_metadata.key();
                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(&resource_id), None, None).unwrap().unwrap();
                system.ok_accesses(&memory, program_id.as_ref(), Some(&Source((*id).clone())), key.as_ref()).is_some_and(|t| t)
            })
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .collect()
    }

    fn divide_independent<'a>(&self, mut systems: HashMap<&'a Id, &'a SystemMetadata>) -> impl Iterator<Item = HashMap<Id, &'a SystemMetadata>> {
        let mut independent: Vec<HashSet<Id>> = Vec::new();
        for (&id, system) in &systems {
            let mut current_set = HashSet::new();

            current_set.insert(id.clone());

            current_set.extend(system.ordering().before().clone());
            current_set.extend(system.ordering().after().clone());

            // Find all sets which overlap with the current set
            let mut dependent_sets = Vec::new();
            for (i, set) in independent.iter().enumerate() {
                if set.intersection(&current_set).next().is_some() {
                    dependent_sets.push(i)
                }
            }

            // Merge all sets found previously
            for i in dependent_sets.into_iter().rev() {
                let set = independent.remove(i);
                current_set.extend(set);
            }

            independent.push(current_set);
        }

        let mut new_systems = Vec::new();
        for ids in independent {
            let mut current_systems = HashMap::new();
            for id in ids {
                // Since currently the "ids" samples from all ids, not just the systems that are currently running
                // if there is a bug in the future can debug by limiting the ids, then panic if systems doesnt contain the id
                if let Some(system) = systems.remove(&id) {
                    current_systems.insert(id, system);
                }
            }

            new_systems.push(current_systems);
        }

        new_systems.into_iter()
    }

    pub async fn execute(&self, execution_graphs: PanicCatchingExecutionGraphs<Id>, memory: &Arc<Memory>) -> Vec<(Id, SystemResult)> {
        let graph_count = execution_graphs.graphs.len();
        let finished_graphs = Arc::new(AtomicUsize::new(graph_count));

        let system_map = Arc::new(
            memory.resolve::<Shared<ProcessorSystemRegistry>>(None, None, None, None)
                .unwrap().unwrap().0
                .into_map()
                .collect::<HashMap<_, _>>()
            );
            
        let system_mapping: Arc<HashMap<Id, SystemCell>> = Arc::new(
           system_map.iter().map(|(id, (resource_id, program_id, key))| {
                (id.clone(), SystemCell::new(memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap().take_system().unwrap()))
           }).collect() 
        );

        let results = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        println!("Starting");
        for i in 0..self.threadpool.max_count() {
            let start_graph = (i * graph_count) / self.threadpool.max_count();

            let runtime = Arc::clone(&self.runtime);
            let execution_graphs = execution_graphs.arc_clone();
            let memory = Arc::clone(&memory);
            let system_map = Arc::clone(&system_map);
            let system_mapping = Arc::clone(&system_mapping);
            let results = Arc::clone(&results);
            let finished_graphs = Arc::clone(&finished_graphs);

            self.threadpool.execute(
                move || {
                    runtime.block_on(async move {
                    // runtime.spawn_blocking(async move {
                        let waker = Waker::from(Arc::new(DummyWaker));
                        let mut context = Context::from_waker(&waker);
                        let mut tasks = Vec::new();

                        let mut current_graph_index = start_graph;

                        while finished_graphs.load(Ordering::Acquire) > 0 {
                            // println!("Looping");
                            let current_graph = execution_graphs.graphs.get(current_graph_index).unwrap();

                            let mut chain = 0;

                            'graphs_walk: while !current_graph.read().await.finished().load(Ordering::Acquire) && chain <= ( 2 * current_graph.read().await.leaves().count()) {
                                let current_graph_read = current_graph.read().await;
                                let leaf_count = current_graph_read.leaves().count();

                                if leaf_count > 0 {
                                    let nth_leaf = if let Some((id, status)) = current_graph_read.leaves().nth(chain % leaf_count) {
                                        // Not `finished` invariant upheld by .leaves()
                                        if status.load(Ordering::Acquire) != ExecutionGraph::<Id>::PENDING {
                                            Some(id.clone())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                    drop(current_graph_read);

                                    if let Some(id) = nth_leaf {
                                        let (resource_id, program_id, key) = system_map.get(&id).unwrap();
                                        let stored_system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, None).unwrap().unwrap();
                                        
                                        match stored_system.status().try_lock() {
                                            Ok(mut status) => {
                                                match *status {
                                                    SystemStatus::Ready => {
                                                        // Safety:
                                                        // Only 1 reference because:
                                                        // System is locked indirectly via status
                                                        // unwrap wont panic because "Ready" branch only happens once
                                                        let inner = unsafe {
                                                            system_mapping.get(&id).unwrap().get()
                                                        };
                                                        
                                                        let source = Source(id.clone());
                                                        // if !inner.reserve_accesses(&memory, program_id.as_ref(), source.clone(), key.as_ref()).is_some_and(|t| t) {
                                                        //     chain += 1;
                                                        //     continue 'graphs_walk;
                                                        // }

                                                        if let Some(Ok(())) = inner.reserve_accesses(&memory, program_id.as_ref(), source.clone(), key.as_ref()) {} 
                                                        else {
                                                            chain += 1;
                                                            continue 'graphs_walk;
                                                        }
                                                        
                                                        *status = SystemStatus::Executing;
                                                        chain = 0;
                                                        
                                                        match inner {
                                                            System::Sync(sync_system) => {
                                                                println!("----> System Running: {:?}", id);
                                                                let result = sync_system.run(
                                                                    &memory,
                                                                    program_id.as_ref(),
                                                                    Some(&source),
                                                                    key.as_ref()
                                                                );

                                                                if let Some(result) = result {
                                                                    results.lock().await.push((id.clone(), result));
                                                                }

                                                                current_graph.write().await.mark_as_complete(&id);
                                                                *status = SystemStatus::Executed;
                                                                println!("<---- System Running: {:?}", id);
                                                            },
                                                            System::Async(async_system) => {
                                                                println!("----> System Running: {:?}", id);
                                                                let mut task = async_system.run(
                                                                    Arc::clone(&memory),
                                                                    program_id.clone(),
                                                                    Some(source.clone()),
                                                                    key.clone()
                                                                );

                                                                match task.as_mut().poll(&mut context) {
                                                                    Poll::Pending => {
                                                                        current_graph.write().await.mark_as_pending(&id);
                                                                        *status = SystemStatus::Pending;

                                                                        tasks.push((
                                                                            current_graph_index,
                                                                            id,
                                                                            task
                                                                        ));
                                                                    },
                                                                    Poll::Ready(result) => {
                                                                        if let Some(result) = result {
                                                                            results.lock().await.push((id.clone(), result));
                                                                        }

                                                                        current_graph.write().await.mark_as_complete(&id);
                                                                        *status = SystemStatus::Executed;
                                                                        println!("<---- System Running: {:?}", id);
                                                                    }
                                                                }
                                                            },
                                                        }
                                                        assert_ne!(*status, SystemStatus::Executing);
                                                    },
                                                    SystemStatus::Pending => chain += 1,
                                                    SystemStatus::Executed => { /* Somehow possible but is benign :) */ },
                                                    SystemStatus::Executing => { unreachable!("Somehow got a lock while another thread should be holding it (possible if another thread panics)") }
                                                }
                                            },
                                            Err(_) => {
                                                // assert!(matches!(err, std::sync::TryLockError::WouldBlock), "How poison?");
                                                
                                                chain += 1;
                                                continue 'graphs_walk;
                                            }
                                        }
                                    }
                                } else {
                                    drop(current_graph_read);

                                    current_graph.write().await.finished().store(true, Ordering::Release);
                                    let _ = finished_graphs.fetch_update(Ordering::SeqCst, Ordering::Relaxed, |finished| {
                                        if finished == 0 {
                                            None
                                        } else {
                                            Some(finished - 1)
                                        }
                                    });
                                }

                                let mut not_done = Vec::new();
                                for (graph_number, id, mut fut) in tasks.drain(..) {
                                    match fut.as_mut().poll(&mut context) {
                                        Poll::Pending => {
                                            not_done.push((graph_number, id, fut));
                                        },
                                        Poll::Ready(result) => {
                                            if let Some(result) = result {
                                                results.lock().await.push((id.clone(), result));
                                            }

                                            let (resource_id, program_id, key) = system_map.get(&id).unwrap();
                                            let stored_system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap();

                                            *stored_system.status().lock().unwrap() = SystemStatus::Executed;
                                            execution_graphs.graphs.get(graph_number).unwrap().write().await.mark_as_complete(&id);

                                            println!("<---- System Finished: {:?}", id);
                                        }
                                    }
                                }

                                tasks.extend(not_done);
                            }
                        
                            current_graph_index = (current_graph_index + 1 ) % graph_count;
                        }
                    })
                }
            );
        }

        PanicCatchingExecutionGraphsFuture::new(&self.threadpool, &execution_graphs).await;

        let mut system_mapping = Arc::try_unwrap(system_mapping).unwrap();
        for (id, mut stored_system) in system_map.iter().map(|(id, (resource_id, program_id, key))| {
                (id, memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap())
        }) {
            *stored_system.status().lock().unwrap() = SystemStatus::Ready;
            stored_system.insert_system(system_mapping.remove(id).unwrap().consume());
        }

        Arc::try_unwrap(results).unwrap().into_inner()
    }
}

impl KernelSystem for Processor {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, ProcessorSystemRegistry::default()).unwrap().is_ok());

        ResourceId::Heap(HeapId::Label(Id("KernelProcessor".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>, ) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<ProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
            
            let systems = Self::get_systems(&memory, &system_registry.0);
            
            {
                let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
                for &id in systems.keys() {
                    next_events.insert(id.clone());
                }
            }
    
            let independent_systems = self.divide_independent(systems);
    
            let execution_graphs = independent_systems
                .map(|systems| {
                    systems.into_iter().map(|(id, system_metadata)| {
                        (id, system_metadata.ordering())
                    }).collect::<Vec<_>>()
                })
                .map(|systems| {
                    tokio::sync::RwLock::new(ExecutionGraph::new(&systems))
                })
                .collect::<Vec<_>>();

            let results = self.execute(
                PanicCatchingExecutionGraphs::new(Arc::new(execution_graphs)),
                &memory
            ).await;

            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let mut next_blockers = memory.resolve::<Unique<NextBlockers>>(None, None, None, None).unwrap().unwrap();

            for (id, result) in results {
                match result {
                    SystemResult::Events(system_events) => {
                        for system_event in system_events {
                            match system_event {
                                SystemEvent::NoEvent => { next_events.remove(&Event::from(id.clone())); },
                                SystemEvent::WithEvent(event) => { next_events.insert(event); },
                                SystemEvent::WithBlocker(blocker) => { next_blockers.insert(blocker); },
                            }
                        }
                    },
                    SystemResult::Error(error) => println!("{id:?}: {error}"),
                    SystemResult::Conditional(_bool) => {
                        // match bool {
                        //     true => {
                        //         todo!()
                        //     },
                        //     false => {
                        //         todo!()
                        //     }
                        // }
                    }
                }
            }
        })        
    }
}