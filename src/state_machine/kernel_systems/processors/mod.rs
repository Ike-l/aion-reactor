use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock, atomic::Ordering}, task::{Context, Poll, Waker}};

use crate::prelude::{CurrentBlockers, CurrentEvents, DummyWaker, ExecutionGraph, FinishedGraphTracker, Memory, Shared, StateMachine, StoredSystem, System, SystemId, SystemMetadata, SystemRegistry, SystemResult, SystemStatus, Unique, Unwinder};

use pollster::FutureExt;
use tracing::{Instrument, Level, event, span};

pub mod system_event_registry;
pub mod tasks;
pub mod unwinder;
pub mod finished_graphs;

pub mod non_blocking_processor;
pub mod blocking_processor;
pub mod read_only_processor;
pub mod system;

pub struct Processor;

impl Processor {
    pub fn insert_system(
        state_machine: &StateMachine, 
        system_registry: &mut SystemRegistry, 
        system_id: SystemId, 
        system_metadata: SystemMetadata, 
        stored_system: StoredSystem
    ) -> Option<Option<SystemMetadata>> {
        let resource_id = system_metadata.stored_system_metadata().resource_id();
        if !matches!(state_machine.memory.insert(None, Some(resource_id.clone()), None, stored_system), Some(Ok(_))) {
            return None;
        }
    
        Some(system_registry.insert(system_id, system_metadata))
    }

    pub fn get_systems<'a>(
        memory: &Memory, 
        system_registry: &'a SystemRegistry,
    ) -> HashMap<&'a SystemId, &'a SystemMetadata> {
        let current_blockers = if let Ok(current_blockers) = memory.resolve::<Shared<CurrentBlockers>>(None, None, None, None).unwrap() {
            &*current_blockers
        } else {
            event!(Level::WARN, "Failed to get CurrentBlockers");
            &CurrentBlockers::default()
        };

        let current_events = if let Ok(current_events) = memory.resolve::<Shared<CurrentEvents>>(None, None, None, None).unwrap() {
            &*current_events
        } else {
            event!(Level::WARN, "Failed to get CurrentEvents");
            &CurrentEvents::default()
        };
    
        let events = current_events.read().collect::<HashSet<_>>();

        let span = span!(Level::DEBUG, "Get Systems");
        let _enter = span.enter();
        system_registry.read()
            // Blocking Stage
            .filter(|&(id, _)| !current_blockers.blocks(&id.clone().into()))
            .inspect(|(id, _)| 
                event!(
                    Level::TRACE, 
                    system_id = ?id, 
                    "Passed Blocking Stage"
                ) 
            )
            // Event Stage
            .filter(|(_, system_metadata)| system_metadata.test(&events))
            .inspect(|(id, _)| 
                event!(
                    Level::TRACE, 
                    system_id = ?id, 
                    "Passed Event Stage"
                ) 
            )
            // Resource Stage
            .filter(|(id, system_metadata)| {
                let resource_id = system_metadata.stored_system_metadata().resource_id();
                let program_id = system_metadata.stored_system_metadata().program_id();
                let key = system_metadata.stored_system_metadata().key();

                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(&resource_id), None, None).unwrap().unwrap();

                system.ok_resources(&memory, program_id.as_ref(), Some(&SystemId::from((*id).clone())), key.as_ref())
                    .is_ok_and(|t| t.is_some_and(|t| t))
            })
            .inspect(|(id, _)| 
                event!(
                    Level::TRACE, 
                    system_id = ?id, 
                    "Passed Resource Stage"
                ) 
            )
            // Reason for commented out: Maybe Access is given later? 
            // Reason for uncommented out: If Access *isnt* given, then will block the whole process- maybe intended behaviour?
            // Access Stage
            .filter(|(id, system_metadata)| {
                let resource_id = system_metadata.stored_system_metadata().resource_id();
                let program_id = system_metadata.stored_system_metadata().program_id();
                let key = system_metadata.stored_system_metadata().key();

                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(&resource_id), None, None).unwrap().unwrap();

                system.ok_accesses(&memory, program_id.as_ref(), Some(&SystemId::from((*id).clone())), key.as_ref())
                    .is_ok_and(|t| t.is_some_and(|t| t))
            })
            .inspect(|(id, _)| 
                event!(
                    Level::TRACE, 
                    system_id = ?id, 
                    "Passed Access Stage"
                ) 
            )
            .inspect(|(id, _)|
                event!(Level::TRACE, system_id = ?id, "Passed All Checks")
            )
            .collect()
    }

    pub fn divide_independent_by_aliasing<'a>(
        mut systems: HashMap<&'a SystemId, &'a SystemMetadata>
    ) -> impl Iterator<Item = HashMap<SystemId, &'a SystemMetadata>> {
        let mut independent: Vec<HashSet<SystemId>> = Vec::new();
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

        event!(Level::DEBUG, groups=new_systems.len(), "Independent System Groups");

        new_systems.into_iter()
    }

    pub async fn execute(
        memory: &Arc<Memory>,
        execution_graphs: Arc<Vec<RwLock<ExecutionGraph<SystemId>>>>, 
        system_registry: &SystemRegistry,
        threadpool: &threadpool::ThreadPool,
        async_runtime: &Arc<tokio::runtime::Runtime>,
    ) -> Vec<(SystemId, SystemResult)> {
        let threads = (threadpool.max_count() - threadpool.active_count()).max(1);
        event!(Level::DEBUG, thread_count=threads, "Thread Count");
        
        let graph_count = execution_graphs.len();
        let chunk_size = graph_count / threads;
        let finished_graphs = Arc::new(FinishedGraphTracker::new(graph_count));
        
        let system_map = Arc::new(system_registry.read()
            .map(|(system_id, system_metadata)| (system_id.clone(), system_metadata.stored_system_metadata().clone()))
            .collect::<HashMap<_, _>>());
        let system_cell_mapping = Arc::new(system_registry.into_system_cell_map(&memory));
        
        let (results_tx, results_rx) = std::sync::mpsc::channel();
        let (unwinder_tx, unwinder_rx) = std::sync::mpsc::channel();
        
        let span = span!(Level::DEBUG, "Execute");
        let _enter = span.enter();

        for current_thread in 0..threads {
            let start_graph = current_thread * chunk_size;

            let memory = Arc::clone(&memory);

            let async_runtime = Arc::clone(&async_runtime);

            let execution_graphs = Arc::clone(&execution_graphs);
            let finished_graphs = Arc::clone(&finished_graphs);
            let unwinder = Unwinder::new(unwinder_tx.clone(), current_thread);
            
            let system_map = Arc::clone(&system_map);
            let system_cell_mapping = Arc::clone(&system_cell_mapping);
            
            let results_tx = results_tx.clone();

            let thread_span = span!(Level::TRACE, "Thread", thread_id=current_thread);

            threadpool.execute(move || {
                async_runtime.block_on(async {
                    let waker = Waker::from(Arc::new(DummyWaker));
                    let mut context = Context::from_waker(&waker);
                    let mut tasks: Vec<(usize, SystemId, std::pin::Pin<Box<dyn Future<Output = Option<SystemResult>> + Send>>)> = Vec::new();

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
                                    if status.load(Ordering::Acquire) != ExecutionGraph::<SystemId>::PENDING {
                                        Some(id.clone())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                drop(current_graph_read);

                                if let Some(system_id) = nth_leaf {
                                    let system_metadata = system_map.get(&system_id).unwrap();
                                    
                                    let stored_system = memory.resolve::<Shared<StoredSystem>>(
                                        system_metadata.program_id().as_ref(), 
                                        Some(system_metadata.resource_id()), 
                                        None, 
                                        system_metadata.key().as_ref()
                                    ).unwrap().unwrap();
                                    
                                    match stored_system.status().try_lock() {
                                        Ok(mut status) => {
                                            match *status {
                                                SystemStatus::Ready => {
                                                    // Safety:
                                                    // Only 1 reference because:
                                                    // System is locked indirectly via status
                                                    // unwrap wont panic because "Ready" branch only happens once
                                                    let inner = unsafe {
                                                        system_cell_mapping.get(&system_id).unwrap().get()
                                                    };

                                                    match inner.reserve_accesses(
                                                        &memory, 
                                                        system_metadata.program_id().as_ref(), 
                                                        system_id.clone(), 
                                                        system_metadata.key().as_ref()
                                                    ) {
                                                        Some(Ok(_)) => (),
                                                        Some(Err(err)) => {
                                                            event!(Level::TRACE, system_id=?system_id, error=?err, "Failed to reserve accesses");
                                                            chain += 1;
                                                            continue 'graphs_walk;
                                                        }
                                                        None => {
                                                            event!(Level::TRACE, system_id=?system_id, "Failed to reserve accesses");
                                                            chain += 1;
                                                            continue 'graphs_walk;
                                                        },
                                                    } 
                                                    
                                                    *status = SystemStatus::Executing;
                                                    chain = 0;
                                                    
                                                    match inner {
                                                        System::Sync(sync_system) => {
                                                            event!(
                                                                Level::TRACE, 
                                                                system_id = ?system_id,
                                                                status = ?status,
                                                                "Sync System Running"
                                                            );

                                                            let result = sync_system.run(
                                                                &memory,
                                                                system_metadata.program_id().as_ref(),
                                                                Some(&system_id),
                                                                system_metadata.key().as_ref()
                                                            );

                                                            if let Some(result) = result {
                                                                let _ = results_tx.send((system_id.clone(), result));
                                                            }

                                                            current_graph.write().unwrap().mark_as_complete(&system_id);

                                                            *status = SystemStatus::Executed;
                                                            
                                                            event!(
                                                                Level::TRACE, 
                                                                system_id = ?system_id,
                                                                status = ?status,
                                                                "Sync System Finished"
                                                            );
                                                        },
                                                        System::Async(async_system) => {
                                                            event!(
                                                                Level::TRACE, 
                                                                system_id = ?system_id,
                                                                status = ?status,
                                                                "Async System Running"
                                                            );

                                                            let mut task = async_system.run(
                                                                Arc::clone(&memory),
                                                                system_metadata.program_id().clone(),
                                                                Some(system_id.clone()),
                                                                system_metadata.key().clone()
                                                            );

                                                            match task.as_mut().poll(&mut context) {
                                                                Poll::Pending => {
                                                                    current_graph.write().unwrap().mark_as_pending(&system_id);
                                                                    *status = SystemStatus::Pending;

                                                                    event!(
                                                                        Level::TRACE, 
                                                                        system_id = ?system_id,
                                                                        status = ?status,
                                                                        "Async System Pending"
                                                                    );

                                                                    tasks.push((
                                                                        current_graph_index,
                                                                        system_id,
                                                                        task
                                                                    ));
                                                                },
                                                                Poll::Ready(result) => {
                                                                    if let Some(result) = result {
                                                                        let _ = results_tx.send((system_id.clone(), result));
                                                                    }

                                                                    current_graph.write().unwrap().mark_as_complete(&system_id);
                                                                    *status = SystemStatus::Executed;
                                                                    
                                                                    event!(
                                                                        Level::TRACE, 
                                                                        system_id = ?system_id,
                                                                        status = ?status,
                                                                        "Async System Finished"
                                                                    );
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

                                // if false before then do event
                                if !finished_graphs.complete(current_graph_index) {
                                    event!(Level::DEBUG, current_graph_index=current_graph_index, "Finished Current Graph");
                                }
                            }

                            let mut not_done = Vec::new();
                            for (graph_number, system_id, mut fut) in tasks.drain(..) {
                                match fut.as_mut().poll(&mut context) {
                                    Poll::Pending => {
                                        // event!(Level::TRACE, system_id=?system_id, "Async System Pending");

                                        not_done.push((graph_number, system_id, fut));
                                    },
                                    Poll::Ready(result) => {
                                        if let Some(result) = result {
                                            let _ = results_tx.send((system_id.clone(), result));
                                        }

                                        let system_metadata = system_map.get(&system_id).unwrap();
                                        let stored_system = memory.resolve::<Unique<StoredSystem>>(
                                            system_metadata.program_id().as_ref(), 
                                            Some(system_metadata.resource_id()), 
                                            None, 
                                            system_metadata.key().as_ref()
                                        ).unwrap().unwrap();

                                        // referenced 1
                                        *stored_system.status().lock().unwrap() = SystemStatus::Executed;
                                        execution_graphs.get(graph_number).unwrap().write().unwrap().mark_as_complete(&system_id);
                                        
                                        event!(
                                            Level::TRACE, 
                                            system_id = ?system_id,
                                            // referenced 1
                                            status = ?SystemStatus::Executed,
                                            "Async System Finished",
                                        );
                                    }
                                }
                            }

                            tasks.extend(not_done);
                        }
                    
                        current_graph_index = (current_graph_index + 1 ) % graph_count;
                    }

                    drop(unwinder);
                }.instrument(thread_span));
            });
        }
        
        drop(results_tx);
        
        let span = span!(Level::DEBUG, "Thread Panic Checks");
        span.in_scope(|| {
            for _ in 0..threads {
                let (panicked, thread_id) = unwinder_rx.recv().unwrap();
                event!(Level::TRACE, panicked=panicked, thread_id=thread_id, "Thread Exit");
                assert!(!panicked, "Panicked!")
            }
        });

        threadpool.join();

        event!(Level::DEBUG, "All Threads Finished");

        let mut system_cells = Arc::try_unwrap(system_cell_mapping).unwrap();
        for (id, mut stored_system) in system_map.iter().filter_map(|(id, system_metadata)| {
            let stored_system = memory.resolve::<Unique<StoredSystem>>(
                system_metadata.program_id().as_ref(), 
                Some(system_metadata.resource_id()), 
                None, 
                system_metadata.key().as_ref()
            )?.ok()?;

            Some((id, stored_system))
        }) {
            *stored_system.status().lock().unwrap() = SystemStatus::Ready;
            stored_system.insert_system(system_cells.remove(id).unwrap().consume());
        }

        assert_eq!(system_cells.len(), 0);

        results_rx.iter().collect()
    }

    /// Only runs "read-only" systems so can optimise out the aliasing checks
    pub async fn execute_fast(
        memory: &Arc<Memory>,
        systems: Vec<SystemId>, 
        system_registry: &SystemRegistry,
        threadpool: &threadpool::ThreadPool,
        async_runtime: &Arc<tokio::runtime::Runtime>,
    ) -> Vec<(SystemId, SystemResult)> {
        let threads = (threadpool.max_count() - threadpool.active_count()).max(1);
        event!(Level::DEBUG, thread_count=threads, "Thread Count");

        let chunk_size = systems.len() / threads;
        let systems = Arc::new(systems);

        let system_map = Arc::new(system_registry.read()
            .map(|(system_id, system_metadata)| (system_id.clone(), system_metadata.stored_system_metadata().clone()))
            .collect::<HashMap<_, _>>());
        let system_cell_mapping = Arc::new(system_registry.into_system_cell_map(&memory));

        let (results_tx, results_rx) = std::sync::mpsc::channel();
        let (unwinder_tx, unwinder_rx) = std::sync::mpsc::channel();

        let span = span!(Level::DEBUG, "Execute Fast");
        let _enter = span.enter();
        let outer_span = span.clone();

        for current_thread in 0..threads {
            let start_chunk = current_thread * chunk_size;
            let current_thread_system_ids = start_chunk..((start_chunk + chunk_size).min(systems.len()));


            let memory = Arc::clone(&memory);

            let async_runtime = Arc::clone(&async_runtime);

            let systems = Arc::clone(&systems);   
            let unwinder = Unwinder::new(unwinder_tx.clone(), current_thread);

            let system_map = Arc::clone(&system_map);
            let system_cell_mapping = Arc::clone(&system_cell_mapping);             

            let results_tx = results_tx.clone();
            
            let outer_span = outer_span.clone();

            threadpool.execute(move || {
                async_runtime.block_on(async {
                    let _enter = outer_span.enter();

                    let span = span!(Level::TRACE, "Thread", thread_id=current_thread);
                    let _enter = span.enter();

                    let mut current_system_index = start_chunk;
                    // ?
                    while current_thread_system_ids.contains(&current_system_index) {
                        let system_id = &systems.get(current_system_index).unwrap();

                        let inner = unsafe {
                            system_cell_mapping.get(*system_id).unwrap().get()
                        };

                        let system_metadata = system_map.get(*system_id).unwrap();

                        match inner {
                            System::Sync(sync_system) => {
                                if !sync_system.check_read_only(Some(&system_id)) {
                                    event!(
                                        Level::TRACE, 
                                        system_id = ?system_id,
                                        "Sync System Not ReadOnly"
                                    );
                                    continue;
                                }

                                event!(
                                    Level::TRACE, 
                                    system_id = ?system_id,
                                    "Sync System Running"
                                );

                                let result = sync_system.run(
                                    &memory, 
                                    system_metadata.program_id().as_ref(), 
                                    Some(&system_id), 
                                    system_metadata.key().as_ref()
                                );
                                
                                if let Some(result) = result {
                                    let _ = results_tx.send(((*system_id).clone(), result));
                                }

                                event!(
                                    Level::TRACE, 
                                    system_id = ?system_id,
                                    "Sync System Finished"
                                );
                            },
                            System::Async(async_system) => {
                                if !async_system.check_read_only(Some(&system_id)) {
                                    event!(
                                        Level::TRACE, 
                                        system_id = ?system_id,
                                        "Async System Not ReadOnly"
                                    );
                                    continue;
                                }

                                event!(
                                    Level::TRACE, 
                                    system_id = ?system_id,
                                    "Async System Running"
                                );

                                // todo better async handling
                                let result = async_system.run(
                                    Arc::clone(&memory), 
                                    system_metadata.program_id().clone(), 
                                    Some((*system_id).clone()), 
                                    system_metadata.key().clone()
                                ).block_on();

                                if let Some(result) = result {
                                    let _ = results_tx.send(((*system_id).clone(), result));
                                }

                                event!(
                                    Level::TRACE, 
                                    system_id = ?system_id,
                                    "Async System Finished"
                                );
                            },
                        }
                    
                        current_system_index += 1;
                    }

                    drop(unwinder);
                });                    
            });
        }
        
        drop(results_tx);

        let span = span!(Level::DEBUG, "Thread Panic Checks");
        span.in_scope(|| {
            for _ in 0..threads {
                let (panicked, thread_id) = unwinder_rx.recv().unwrap();
                event!(Level::TRACE, panicked=panicked, thread_id=thread_id, "Thread Exit");
                assert!(!panicked, "Panicked!")
            }
        });

        threadpool.join();
        
        event!(Level::DEBUG, "All Threads Finished");

        // let mut system_mapping = Arc::try_unwrap(system_mapping).unwrap();
        // for (id, mut stored_system) in system_map.iter().map(|(id, (resource_id, program_id, key))| {
        //         (id, memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap())
        // }) {
        //     stored_system.insert_system(system_mapping.remove(id).unwrap().consume());
        // }

        let mut system_cells = Arc::try_unwrap(system_cell_mapping).unwrap();
        for (id, mut stored_system) in system_map.iter().filter_map(|(id, system_metadata)| {
            let stored_system = memory.resolve::<Unique<StoredSystem>>(
                system_metadata.program_id().as_ref(), 
                Some(system_metadata.resource_id()), 
                None, 
                system_metadata.key().as_ref()
            )?.ok()?;

            Some((id, stored_system))
        }) {
            *stored_system.status().lock().unwrap() = SystemStatus::Ready;
            stored_system.insert_system(system_cells.remove(id).unwrap().consume());
        }

        assert_eq!(system_cells.len(), 0);

        results_rx.iter().collect()
    }

    pub async fn execute_non_blocking(
        memory: &Arc<Memory>,
        systems: HashMap<&SystemId, &SystemMetadata>,
        async_runtime: &Arc<tokio::runtime::Runtime>,
    ) -> (Vec<(SystemId, tokio::task::JoinHandle<System>)>, Vec<(SystemId, std::thread::JoinHandle<System>)>) {
        let mut new_async_join_handles = Vec::new();
        let mut new_sync_join_handles = Vec::new();

        let span = span!(Level::DEBUG, "Execute NonBlocking");
        let _enter = span.enter();

        for (system_id, system_metadata) in systems {
                let program_id = system_metadata.stored_system_metadata().program_id();
                let resource_id = system_metadata.stored_system_metadata().resource_id();
    
                let mut system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, None).unwrap().unwrap();

                match system.reserve_accesses(
                    &memory, 
                    system_metadata.stored_system_metadata().program_id().as_ref(), 
                    system_id.clone(), 
                    system_metadata.stored_system_metadata().key().as_ref()
                ) {
                    Ok(Some(Ok(_))) => {},
                    Ok(Some(Err(err))) => {
                        event!(Level::TRACE, system_id=?system_id, error=?err, "Failed to reserve accesses");
                        continue;
                    },
                    Ok(None) => {
                        event!(Level::TRACE, system_id=?system_id, "Failed to reserve accesses");
                        continue
                    },
                    Err(err) => {
                        event!(Level::TRACE, system_id=?system_id, error=?err, "Failed to reserve accesses");
                        continue;
                    },
                }
                
                let source = system_id.clone();

                let memory_clone = Arc::clone(&memory);
                let program_id = program_id.clone();
                let source = source.clone();
                let key = system_metadata.stored_system_metadata().key().clone();

                if let Some(system) = system.take_system() {
                    match system {
                        System::Sync(mut sync_system) => {
                            let thread_span = span!(Level::TRACE, "Sync Thread", system_id=?system_id);
                            
                            let join_handle = std::thread::spawn(move || {
                                let _enter = thread_span.enter();

                                event!(
                                    Level::TRACE, 
                                    status="Executing",
                                    "NonBlocking Sync System",
                                );

                                sync_system.run(&memory_clone, program_id.as_ref(), Some(&source), key.as_ref());
                                
                                event!(
                                    Level::TRACE, 
                                    status="Executed",
                                    "NonBlocking Sync System"
                                );

                                System::Sync(sync_system)
                            });

                            new_sync_join_handles.push((system_id.clone(), join_handle));
                        }
                        System::Async(mut async_system) => {
                            let thread_span = span!(Level::TRACE, "Async Thread", system_id=?system_id);

                            let join_handle = async_runtime.spawn(async move {
                                event!(
                                    Level::TRACE, 
                                    status="Executing",
                                    "NonBlocking Async System"
                                );

                                async_system.run(memory_clone, program_id, Some(source.clone()), key).await;
                                
                                event!(
                                    Level::TRACE, 
                                    status="Executed",
                                    "NonBlocking Async System"
                                );

                                System::Async(async_system)
                            }.instrument(thread_span));

                            new_async_join_handles.push((system_id.clone(), join_handle));
                        }
                    }
                } else {
                    unreachable!("Matches StoredSystemError should catch this");
                }
            }
    
        (new_async_join_handles, new_sync_join_handles)
    }
}