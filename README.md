# Aion-Reactor

My Modularised System Framework

My goal is to monopolise SaaS, and all frameworks- incl. web frameworks and game engines.

Selling to programmers:
Imagine you could upload units of code and receive commission

Selling to companies:
Imagine you could have access to a data pipeline in the cloud that could handle arbitrary logic with massively parallel distributed systems. 
Imagine SaaS done easy
(parallel computers, where each computer uses all resources it has to the maximum)

Taken from `Aion` meaning unbound time, and `Reactor` representing the state machine as a nuclear reactor, how events cause chain reactions


Aion-Reactor is centered around `StateMachine` with a minimal API

`resolve`: Query `Memory`

`insert`: Insert to `Memory`

`transition`: "Tick", Run loaded Kernel Systems

Kernel Systems:

Background Processor
- Can spawn systems to run in the background

Blocker Manager
- Can "block" systems, for the current tick: disregard the trigger function of the system

Delay Manager
- On an event existing, spawn an event and optionally delay an event

Event Manager
- Transitions `Next` Events into `Current` Events

Executable Manager
- Parses a string to sequence systems, passing resources between stages

Processor
- Can spawn systems to run in parallel, or asynchronous, where the main thread blocks until everything is complete
- Looks at what resources the system uses to ensure no aliasing
- Adheres to a scheduling parameter to order systems

Read Only Processor
- Uses the invariants that all systems are ReadOnly to maximise throughput
- Can be used as Conditionals, returning true spawns events which can be observed (i.e number == 3)





TODO:

allow systems to be run directly from `MemoryDomain`

make blocker manager remove the event from current events && remove it from Processing
schedule blocker manager after DelayManager

Make my own threadpool implementation

MemoryDomain injection parameter

fix kernel systems to better elogance and move load default into Default implementation 

schedule executable manager before DelayManager