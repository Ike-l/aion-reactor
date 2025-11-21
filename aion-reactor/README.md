# Aion-Reactor

Taken from `Aion` meaning unbound time, and `Reactor` representing the transition of states

Aion-Reactor is centered around `StateMachine` with a minimal API

`resolve`: Query internal state

`insert`: Insert to internal state

`transition`: "Tick" the internal state- as the name implies

Still in very early development (so extremely unstable!) but it should be usable for all your abstract computational needs!


Background Processor
- Can spawn systems to run in the background

Processor
- Can spawn systems to run in parallel, or asynchronous, where the main thread blocks until everything is complete
- Looks at what resources the system uses to ensure no aliasing
- Adheres to a scheduling parameter to order systems

Blocker Manager
- Can "block" systems, for the current tick: disregard the trigger function of the system

Delay Manager
- On an event existing, spawn an event and optionally delay an event

Event Manager

Executable Manager
- Parses a string to sequence systems, passing resources between stages






TODO:

allow systems to be run directly from `MemoryDomain`

make blocker manager remove the event from current events && remove it from Processing
schedule blocker manager after DelayManager

Make my own threadpool implementation

MemoryDomain injection parameter

fix kernel systems to better elogance and move load default into Default implementation 

schedule executable manager before DelayManager