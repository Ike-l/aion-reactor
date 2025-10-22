# Aion-Reactor

Taken from `Aion` meaning unbound time, and `Reactor` representing the transition of states

Aion-Reactor is centered around `StateMachine` with a minimal API

`resolve`: Query internal state

`insert`: Insert to internal state

`transition`: "Tick" the internal state- as the name implies

Still in very early development (so extremely unstable!) but it should be usable for all your abstract computational needs!

I personally made this to be the pillar of a game engine i want to make


TODO:

allow systems to be run directly from `MemoryDomain`

better Blacklist
current bug: Need the kernel system to run 1. but also need to blacklist it for other kernel systems