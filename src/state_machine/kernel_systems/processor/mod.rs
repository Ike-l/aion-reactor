use std::{collections::{HashMap, HashSet}, pin::Pin, sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering}}, task::{Context, Poll, Waker}};

use threadpool::ThreadPool;

use crate::{id::Id, injection::injection_primitives::{cloned::Cloned, shared::Shared, unique::Unique}, memory::{Memory, ResourceId, access_checked_heap::heap::HeapId}, state_machine::{StateMachine, kernel_systems::{KernelSystem, blocker_manager::prelude::{NextBlockers, CurrentBlockers}, event_manager::prelude::{CurrentEvents, Event, NextEvents}, processor::{processor_system_registry::ProcessorSystemRegistry, scheduler::execution_graph::ExecutionGraph, tasks::DummyWaker}}}, system::{System, stored_system::StoredSystem, system_cell::SystemCell, system_metadata::{Source, SystemMetadata, SystemRegistry}, system_result::{SystemEvent, SystemResult}, system_status::SystemStatus}};

pub mod scheduler;
pub mod tasks;
pub mod processor_system_registry;

#[derive(Debug)]
pub struct Processor {
    runtime: Arc<tokio::runtime::Runtime>,
    threads: usize,
    threadpool: ThreadPool
}

pub struct Unwinder {
    results: std::sync::mpsc::Sender<bool>
}

impl Drop for Unwinder {
    fn drop(&mut self) {
        let _ = self.results.send(std::thread::panicking());
    }
}

impl Processor {
    pub fn new(threads: usize) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Self {
            runtime: Arc::new(rt),
            threads,
            threadpool: ThreadPool::new(threads)
        }
    }

    pub fn insert_system(state_machine: &StateMachine, system_id: Id, system_metadata: SystemMetadata, stored_system: StoredSystem) -> Option<Option<SystemMetadata>> {
        let mut system_registry = state_machine.state.resolve::<Unique<ProcessorSystemRegistry>>(None, None, None, None)?.ok()?;
        Self::insert_system2(state_machine, &mut system_registry.0, system_id, system_metadata, stored_system)
    }


    pub fn insert_system2(state_machine: &StateMachine, system_registry: &mut SystemRegistry, system_id: Id, system_metadata: SystemMetadata, stored_system: StoredSystem) -> Option<Option<SystemMetadata>> {
        let resource_id = system_metadata.resource_id();
        if !matches!(state_machine.state.insert(None, Some(resource_id.clone()), None, stored_system), Some(Ok(_))) {
            return None;
        }

        Some(system_registry.insert(system_id, system_metadata))
    }

    // `Id` == `Source` 👍😁
    pub fn get_systems<'a>(memory: &Memory, system_registry: &'a SystemRegistry) -> HashMap<&'a Id, &'a SystemMetadata> {
        let current_blockers = memory.resolve::<Shared<CurrentBlockers>>(None, None, None, None).unwrap().unwrap();
        let current_events = memory.resolve::<Shared<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
    
        let events = current_events.read().collect::<HashSet<_>>();
    
        system_registry.read()
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|&(id, _)| !current_blockers.blocks(id.clone().into()))
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|(_, system_metadata)| system_metadata.test(&events))
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|(id, system_metadata)| {
                let resource_id = system_metadata.resource_id();
                let program_id = system_metadata.program_id();
                let key = system_metadata.key();
                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(&resource_id), None, None).unwrap().unwrap();
                system.ok_resources(&memory, program_id.as_ref(), Some(&Source((*id).clone())), key.as_ref())
                    .is_ok_and(|t| t.is_some_and(|t| t))
            })
            // .inspect(|(id, _)| println!("id: {id:?}") )
            .filter(|(id, system_metadata)| {
                let resource_id = system_metadata.resource_id();
                let program_id = system_metadata.program_id();
                let key = system_metadata.key();
                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(&resource_id), None, None).unwrap().unwrap();
                system.ok_accesses(&memory, program_id.as_ref(), Some(&Source((*id).clone())), key.as_ref())
                    .is_ok_and(|t| t.is_some_and(|t| t))
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

    pub async fn execute(&self, execution_graphs: Arc<Vec<RwLock<ExecutionGraph<Id>>>>, memory: &Arc<Memory>, trace: Arc<Trace>) -> Vec<(Id, SystemResult)> {
        let graph_count = execution_graphs.len();
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

        let (results_tx, results_rx) = std::sync::mpsc::channel();
        let (unwinder_tx, unwinder_rx) = std::sync::mpsc::channel();

        for i in 0..self.threads {
            let start_graph = (i * graph_count) / self.threads;

            let memory = Arc::clone(&memory);
            let runtime = self.runtime.clone();
            let execution_graphs = Arc::clone(&execution_graphs);
            let system_map = Arc::clone(&system_map);
            let system_mapping = Arc::clone(&system_mapping);
            let results = results_tx.clone();
            let finished_graphs = Arc::clone(&finished_graphs);
            let trace = Arc::clone(&trace);

            let unwinder = Unwinder {
                results: unwinder_tx.clone()
            };
            
            self.threadpool.execute(move || {
                runtime.block_on(async {
                    let waker = Waker::from(Arc::new(DummyWaker));
                    let mut context = Context::from_waker(&waker);
                    let mut tasks = Vec::new();

                    let mut current_graph_index = start_graph;
                    while finished_graphs.load(Ordering::Acquire) > 0 {
                        let current_graph = execution_graphs.get(current_graph_index).unwrap();

                        let mut chain = 0;

                        'graphs_walk: while {
                            let current_graph = current_graph.read().unwrap();
                            let finished = !current_graph.finished().load(Ordering::Acquire);
                            let leaves_count = current_graph.leaves().count();
                            finished && chain <= ( 2 * leaves_count) 
                        } {
                            let current_graph_read = current_graph.read().unwrap();
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

                                                    if let Some(Ok(())) = inner.reserve_accesses(&memory, program_id.as_ref(), source.clone(), key.as_ref()) {} 
                                                    else {
                                                        chain += 1;
                                                        continue 'graphs_walk;
                                                    }
                                                    
                                                    *status = SystemStatus::Executing;
                                                    chain = 0;
                                                    
                                                    match inner {
                                                        System::Sync(sync_system) => {
                                                            if trace.enabled_system() {
                                                                println!("----> Thread: {i}. System Running: {:?}", id);
                                                            }

                                                            let result = sync_system.run(
                                                                &memory,
                                                                program_id.as_ref(),
                                                                Some(&source),
                                                                key.as_ref()
                                                            );

                                                            if let Some(result) = result {
                                                                let _ = results.send((id.clone(), result));
                                                            }

                                                            current_graph.write().unwrap().mark_as_complete(&id);

                                                            *status = SystemStatus::Executed;
                                                            
                                                            if trace.enabled_system() {
                                                                println!("<---- Thread: {i}. System Running: {:?}", id);
                                                            }
                                                        },
                                                        System::Async(async_system) => {
                                                            if trace.enabled_system() {
                                                                println!("----> Thread: {i}. System Running: {:?}", id);
                                                            }

                                                            let mut task = async_system.run(
                                                                Arc::clone(&memory),
                                                                program_id.clone(),
                                                                Some(source.clone()),
                                                                key.clone()
                                                            );

                                                            match task.as_mut().poll(&mut context) {
                                                                Poll::Pending => {
                                                                    current_graph.write().unwrap().mark_as_pending(&id);
                                                                    *status = SystemStatus::Pending;

                                                                    tasks.push((
                                                                        current_graph_index,
                                                                        id,
                                                                        task
                                                                    ));
                                                                },
                                                                Poll::Ready(result) => {
                                                                    if let Some(result) = result {
                                                                        let _ = results.send((id.clone(), result));
                                                                    }

                                                                    current_graph.write().unwrap().mark_as_complete(&id);
                                                                    *status = SystemStatus::Executed;
                                                                    
                                                                    if trace.enabled_system() {
                                                                        println!("<---- Thread: {i}. System Running: {:?}", id);
                                                                    }
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

                                current_graph.write().unwrap().finished().store(true, Ordering::Release);
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
                                            let _ = results.send((id.clone(), result));
                                        }

                                        let (resource_id, program_id, key) = system_map.get(&id).unwrap();
                                        let stored_system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap();

                                        *stored_system.status().lock().unwrap() = SystemStatus::Executed;
                                        execution_graphs.get(graph_number).unwrap().write().unwrap().mark_as_complete(&id);
                                        
                                        if trace.enabled_system() {
                                            println!("<---- Thread: {i}. System Finished: {:?}", id);
                                        }
                                    }
                                }
                            }

                            tasks.extend(not_done);
                        }
                    
                        current_graph_index = (current_graph_index + 1 ) % graph_count;
                    }
                    drop(unwinder);
                });
            });
        }
        
        drop(results_tx);
        
        for _ in 0..self.threads {
            let panicked = unwinder_rx.recv().unwrap();
            assert!(!panicked, "Panicked!")
        }

        self.threadpool.join();

        let mut system_mapping = Arc::try_unwrap(system_mapping).unwrap();
        for (id, mut stored_system) in system_map.iter().map(|(id, (resource_id, program_id, key))| {
                (id, memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap())
        }) {
            *stored_system.status().lock().unwrap() = SystemStatus::Ready;
            stored_system.insert_system(system_mapping.remove(id).unwrap().consume());
        }

        results_rx.iter().collect()
    }
}

#[derive(Clone)]
pub struct Trace {
    enable_all: bool
}

impl Trace {
    pub fn new() -> Self {
        Self {
            enable_all: false
        }
    }

    pub fn enable_all(&mut self) {
        self.enable_all = true;
    }

    pub fn disable_all(&mut self) {
        self.enable_all = false;
    }

    pub fn enabled_system(&self) -> bool {
        self.enable_all
    }
}


impl KernelSystem for Processor {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, ProcessorSystemRegistry::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, Trace::new()).unwrap().is_ok());

        ResourceId::Heap(HeapId::Label(Id("KernelProcessor".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
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
                    std::sync::RwLock::new(ExecutionGraph::new(&systems))
                })
                .collect::<Vec<_>>();

            let trace = {
                memory.resolve::<Cloned<Trace>>(None, None, None, None).unwrap().unwrap().take()
            };

            let results = self.execute(
                Arc::new(execution_graphs),
                &memory,
                Arc::new(trace)
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