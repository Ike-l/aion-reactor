use std::{collections::{HashMap, HashSet}, pin::Pin, sync::{atomic::{AtomicUsize, Ordering}, Arc}, task::{Context, Poll, Waker}};

use threadpool::ThreadPool;

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::Memory, state_machine::kernel_systems::{blocker_manager::blocker::CurrentBlockers, event_manager::event::{CurrentEvents, Event, NextEvents}, processor::{blacklist::Blacklist, scheduler::{execution_graph::ExecutionGraph, panic_catching_execution_graph::PanicCatchingExecutionGraphs, panic_catching_execution_graph_future::PanicCatchingExecutionGraphsFuture}, tasks::DummyWaker}, KernelSystem}, system::{stored_system::StoredSystem, system_cell::SystemCell, system_metadata::{SystemMetadata, SystemRegistry}, system_result::{SystemEvent, SystemResult}, system_status::SystemStatus, System}};

pub mod scheduler;
pub mod tasks;
pub mod blacklist;

#[derive(Debug)]
pub struct Processor {
    threadpool: ThreadPool,
    runtime: Arc<tokio::runtime::Runtime>
}

impl Processor {
    pub fn new(threads: usize) -> Self {
        Self {
            threadpool: ThreadPool::new(threads),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap())
        }
    }

    fn get_systems<'a>(&self, memory: &Memory, system_registry: &'a SystemRegistry) -> HashMap<&'a Id, &'a SystemMetadata> {
        let current_blockers = memory.resolve::<Shared<CurrentBlockers>>(None, None, None).unwrap().unwrap();
        let current_events = memory.resolve::<Shared<CurrentEvents>>(None, None, None).unwrap().unwrap();
    
        let blacklist = memory.resolve::<Shared<Blacklist>>(None, None, None).unwrap().unwrap();
    
        let events = current_events.read().collect::<HashSet<_>>();
    
        system_registry.read()
            .filter(|&(id, _)| !current_blockers.blocks(id.clone()))
            .filter(|(id, _)| !blacklist.blocks(id))
            .filter(|(_, system_metadata)| system_metadata.test(&events))
            .filter(|(_, system_metadata)| {
                let (source, program_id) = system_metadata.ids();
                let system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), None, Some(source)).unwrap().unwrap();
                system.ok_resources(&memory, program_id.as_ref(), Some(source)).is_some_and(|t| t)
            }).collect()
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
            memory.resolve::<Shared<SystemRegistry>>(None, None, None)
                .unwrap().unwrap()
                .into_map()
                .collect::<HashMap<_, _>>()
            );
            
        let system_mapping: Arc<HashMap<Id, SystemCell>> = Arc::new(
           system_map.iter().map(|(id, (source, program_id))| {
                (id.clone(), SystemCell::new(memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), None, Some(source)).unwrap().unwrap().take_system().unwrap()))
           }).collect() 
        );

        let results = Arc::new(tokio::sync::Mutex::new(Vec::new()));
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
                        let waker = Waker::from(Arc::new(DummyWaker));
                        let mut context = Context::from_waker(&waker);
                        let mut tasks = Vec::new();

                        let mut current_graph_index = start_graph;

                        while finished_graphs.load(Ordering::Acquire) > 0 {
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
                                        let (source, program_id) = system_map.get(&id).unwrap();
                                        let stored_system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), None, Some(source)).unwrap().unwrap();
                                        
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
                                                        
                                                        if !inner.reserve_accesses(&memory, program_id.as_ref(), source.clone()).is_some_and(|t| t) {
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
                                                                    Some(source),
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
                                            Err(err) => {
                                                assert!(matches!(err, std::sync::TryLockError::WouldBlock), "How poison?");
                                                
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
                                    }).unwrap();
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

                                            let (source, program_id) = system_map.get(&id).unwrap();
                                            let stored_system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), None, Some(source)).unwrap().unwrap();

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
        for (id, mut stored_system) in system_map.iter().map(|(id, (source, program_id))| {
                (id, memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), None, Some(source)).unwrap().unwrap())
        }) {
            *stored_system.status().lock().unwrap() = SystemStatus::Ready;
            stored_system.insert_system(system_mapping.remove(id).unwrap().consume());
        }

        Arc::try_unwrap(results).unwrap().into_inner()
    }
}

impl KernelSystem for Processor {
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<SystemRegistry>>(None, None, None).unwrap().unwrap();
            let systems = self.get_systems(&memory, &system_registry);
            
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None).unwrap().unwrap();
            for &id in systems.keys() {
                next_events.insert(id.clone());
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

            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None).unwrap().unwrap();
            for (id, result) in results {
                match result {
                    SystemResult::Event(system_event) => match system_event {
                        SystemEvent::NoEvent => { next_events.remove(&Event::from(id)); },
                    },
                    SystemResult::Error(error) => println!("{id:?}: {error}"),
                }
            }
        })        
    }
}