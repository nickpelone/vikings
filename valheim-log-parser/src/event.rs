pub struct EventData {}

pub enum Event {
    UserConnected(EventData),
    UserDisconnected(EventData),
    WorldSaved(EventData),
    CharacterDied(EventData),
    CharacterRespawned(EventData)
}