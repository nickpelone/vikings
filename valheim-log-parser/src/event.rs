use chrono::NaiveDateTime;

/// placeholder struct until I spec out more events and their relevant data
#[derive(Debug, Clone)]
pub struct EventData;

#[derive(Debug, Clone)]
pub struct SpawnData {
    pub timestamp: NaiveDateTime,
    pub character: String,
    pub location: (i64, i64),
}

#[derive(Debug, Clone, Copy)]
pub struct SaveData {
    pub timestamp: NaiveDateTime,
    pub time_spent: f64
}

#[derive(Debug, Clone)]
pub enum Event {
    UserConnected(EventData),
    UserDisconnected(EventData),
    WorldSaved(SaveData),
    CharacterDied(SpawnData),
    CharacterSpawned(SpawnData)
}