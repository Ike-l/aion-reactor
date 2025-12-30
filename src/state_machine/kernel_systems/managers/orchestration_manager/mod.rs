// todo: make sure all calculations related to event! are in the event macro so when excluded it doesnt waste cycles

/*

remove EventMapper and SystemEventRegistry
EventMapper is "Becomes"
"Becomes" will spawn all events that an event becomes
SystemEventMapper is "Is"
"Is" will spawn all events that are the "same" when it sees any of them


remove DelayManager
*/


/*
EventMapperManager

IsEventMapper
BecomesEventMapper
// "Is" happens before "Becomes"

TimeEventManager:

Creates Events for particular events
i.e user can specify every 5 ticks spawn these Vec<EventId>
i.e user can specify every 3 seconds spawn these Vec<EventId>

// While Manager:
Delay (while do)
1. When A; While B spawn C; Finally spawn D

Wait (do until)
2. When A; While !B spawn C; Finally spawn D

struct WhileEvent {
    When: EventId,
    While: EventId,
    /// Empty can mean Delay
    Do: Vec<EventId>,
    /// Empty can mean Faucet
    Finally: Vec<EventId>,

    /// maps While EventId -> While !EventId
    /// so becomes an "until" coded "while loop"
    is_until: bool
}

to delay use 1. Have B be the delayer and have C be empty
to wait use 2. Have B be the turn off and have D be empty

Delay is like a kettle (A === turn on; B === boiling in progress; D === use boiled water)
Wait is like a tap (A === turn on; C === water; !B === turn off)

// this manager is essentially just an abstraction level above while loops over the domain of events
*/