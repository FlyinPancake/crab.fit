use ambassador::delegatable_trait;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum AdaptorError {
    #[error("conflict")]
    Conflict, // unique violation, optimistic-lock failure, etc.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl AdaptorError {
    pub fn internal<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self::Internal(Box::new(e))
    }
}

/// Data storage adaptor, all methods on an adaptor can return an error if
/// something goes wrong, or potentially None if the data requested was not found.
#[async_trait]
#[delegatable_trait]
pub trait Adaptor: Send + Sync {
    async fn get_stats(&self) -> Result<Stats, AdaptorError>;
    async fn increment_stat_event_count(&self) -> Result<i64, AdaptorError>;
    async fn increment_stat_person_count(&self) -> Result<i64, AdaptorError>;

    async fn get_people(&self, event_id: String) -> Result<Option<Vec<Person>>, AdaptorError>;
    async fn upsert_person(
        &self,
        event_id: String,
        person: Person,
    ) -> Result<Option<Person>, AdaptorError>;

    /// Get an event and update visited date to current time
    async fn get_event(&self, id: String) -> Result<Option<Event>, AdaptorError>;
    async fn create_event(&self, event: Event) -> Result<Event, AdaptorError>;

    /// Delete events older than a cutoff date, as well as any associated people
    /// Returns the amount of events and people deleted
    async fn delete_events(&self, cutoff: DateTime<Utc>) -> Result<Stats, AdaptorError>;
}

#[derive(Clone)]
pub struct Stats {
    pub event_count: i64,
    pub person_count: i64,
}

#[derive(Clone)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub visited_at: DateTime<Utc>,
    pub times: Vec<String>,
    pub timezone: String,
}

#[derive(Clone)]
pub struct Person {
    pub name: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub availability: Vec<String>,
}
