use crate::board::OutputId;

#[derive(Clone, Debug)]
pub struct Mission<'a> {
    pub channels: &'a [Channel],
    pub states: &'a [State<'a>],
    pub initial_state: StateId,
    pub transitions: &'a [Transition<'a>],
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StateId(pub u8);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ChannelId(pub u8);

#[derive(Clone, Debug)]
pub struct Channel {
    pub output: OutputId,
    pub drive: Drive,
}

#[derive(Clone, Debug)]
pub enum Drive {
    Pulse { ms: u32 },
    Hold,
}

#[derive(Clone, Debug)]
pub enum Action {
    Fire(ChannelId),
}

#[derive(Clone, Debug)]
pub struct State<'a> {
    pub on_enter: &'a [Action],
    pub hold: &'a [ChannelId],
}

#[derive(Clone, Copy, Debug)]
pub enum Signal {
    AltitudeAgl,
    VerticalSpeed,
    AccelZ,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Apogee,
}

#[derive(Clone, Debug)]
pub enum Condition<'a> {
    All(&'a [Condition<'a>]),
    Any(&'a [Condition<'a>]),
    TimeInState {
        above_ms: u32,
    },
    Always,
    Above {
        signal: Signal,
        value: f32,
        for_ms: u32,
    },
    Below {
        signal: Signal,
        value: f32,
        for_ms: u32,
    },
    Between {
        signal: Signal,
        lower_inclusive: f32,
        upper_inclusive: f32,
        for_ms: u32,
    },
    Event(Event),
}

#[derive(Clone, Debug)]
pub struct Transition<'a> {
    pub from: StateId,
    pub to: StateId,
    pub when: Condition<'a>,
}
