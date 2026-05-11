use std::error::Error;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use common::{Adaptor, AdaptorError, Event, Person, Stats};
use entity::{event, person, stats};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
    TransactionError, TransactionTrait,
};
use serde_json::json;
use strum::Display;

mod entity;

pub struct SqlAdaptor {
    db: DatabaseConnection,
}

#[async_trait]
impl Adaptor for SqlAdaptor {
    async fn get_stats(&self) -> Result<Stats, AdaptorError> {
        let stats_row = get_stats_row(&self.db).await?;
        Ok(Stats {
            event_count: stats_row.event_count.unwrap() as i64,
            person_count: stats_row.person_count.unwrap() as i64,
        })
    }

    async fn increment_stat_event_count(&self) -> Result<i64, AdaptorError> {
        let mut current_stats = get_stats_row(&self.db).await?;
        current_stats.event_count = Set(current_stats.event_count.unwrap() + 1);

        Ok(current_stats
            .save(&self.db)
            .await
            .map_err(AdaptorError::internal)?
            .event_count
            .unwrap() as i64)
    }

    async fn increment_stat_person_count(&self) -> Result<i64, AdaptorError> {
        let mut current_stats = get_stats_row(&self.db).await?;
        current_stats.person_count = Set(current_stats.person_count.unwrap() + 1);

        Ok(current_stats
            .save(&self.db)
            .await
            .map_err(AdaptorError::internal)?
            .person_count
            .unwrap() as i64)
    }

    async fn get_people(&self, event_id: String) -> Result<Option<Vec<Person>>, AdaptorError> {
        // TODO: optimize into one query
        let event_row = event::Entity::find_by_id(event_id)
            .one(&self.db)
            .await
            .map_err(AdaptorError::internal)?;

        Ok(match event_row {
            Some(event) => Some(
                event
                    .find_related(person::Entity)
                    .all(&self.db)
                    .await
                    .map_err(AdaptorError::internal)?
                    .into_iter()
                    .map(|model| model.into())
                    .collect(),
            ),
            None => None,
        })
    }

    async fn upsert_person(
        &self,
        event_id: String,
        person: Person,
    ) -> Result<Option<Person>, AdaptorError> {
        let data = person::ActiveModel {
            name: Set(person.name.clone()),
            password_hash: Set(person.password_hash),
            created_at: Set(person.created_at.naive_utc()),
            availability: Set(serde_json::to_value(person.availability).unwrap_or(json!([]))),
            event_id: Set(event_id.clone()),
        };

        // Check if the event exists
        if event::Entity::find_by_id(event_id.clone())
            .one(&self.db)
            .await
            .map_err(AdaptorError::internal)?
            .is_none()
        {
            return Ok(None);
        }

        Ok(Some(
            match person::Entity::find_by_id((person.name, event_id))
                .one(&self.db)
                .await
                .map_err(AdaptorError::internal)?
            {
                Some(_) => data
                    .update(&self.db)
                    .await
                    .map_err(AdaptorError::internal)?
                    .into(),
                None => data
                    .insert(&self.db)
                    .await
                    .map_err(AdaptorError::internal)?
                    .into(),
            },
        ))
    }

    async fn get_event(&self, id: String) -> Result<Option<Event>, AdaptorError> {
        let existing_event = event::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AdaptorError::internal)?;

        // Mark as visited
        if let Some(event) = existing_event.clone() {
            let mut event: event::ActiveModel = event.into();
            event.visited_at = Set(Utc::now().naive_utc());
            event.save(&self.db).await.map_err(AdaptorError::internal)?;
        }

        Ok(existing_event.map(|model| model.into()))
    }

    async fn create_event(&self, event: Event) -> Result<Event, AdaptorError> {
        Ok(event::ActiveModel {
            id: Set(event.id),
            name: Set(event.name),
            created_at: Set(event.created_at.naive_utc()),
            visited_at: Set(event.visited_at.naive_utc()),
            times: Set(serde_json::to_value(event.times).unwrap_or(json!([]))),
            timezone: Set(event.timezone),
            event_type: Set(serde_json::to_value(event.event_type).unwrap_or(json!([]))),
        }
        .insert(&self.db)
        .await
        .map_err(AdaptorError::internal)?
        .into())
    }

    async fn delete_events(&self, cutoff: DateTime<Utc>) -> Result<Stats, AdaptorError> {
        // TODO: use normal transaction
        let (event_count, person_count) = self
            .db
            .transaction::<_, (i64, i64), DbErr>(|t| {
                Box::pin(async move {
                    // Get events older than the cutoff date
                    let old_events = event::Entity::find()
                        .filter(event::Column::VisitedAt.lt(cutoff.naive_utc()))
                        .all(t)
                        .await?;

                    // Delete people
                    let mut people_deleted: i64 = 0;
                    // TODO: run concurrently
                    for e in old_events.iter() {
                        let people_delete_result = person::Entity::delete_many()
                            .filter(person::Column::EventId.eq(&e.id))
                            .exec(t)
                            .await?;
                        people_deleted += people_delete_result.rows_affected as i64;
                    }

                    // Delete events
                    let event_delete_result = event::Entity::delete_many()
                        .filter(event::Column::VisitedAt.lt(cutoff.naive_utc()))
                        .exec(t)
                        .await?;

                    Ok((event_delete_result.rows_affected as i64, people_deleted))
                })
            })
            .await
            .map_err(AdaptorError::internal)?;

        Ok(Stats {
            event_count,
            person_count,
        })
    }
}

// Get the current stats as an ActiveModel
async fn get_stats_row(db: &DatabaseConnection) -> Result<stats::ActiveModel, AdaptorError> {
    let current_stats = stats::Entity::find()
        .one(db)
        .await
        .map_err(AdaptorError::internal)?;

    Ok(match current_stats {
        Some(model) => model.into(),
        None => stats::ActiveModel {
            id: NotSet,
            event_count: Set(0),
            person_count: Set(0),
        },
    })
}

impl SqlAdaptor {
    pub async fn new(database_url: &str) -> Self {
        // Connect to the database
        let db = Database::connect(database_url)
            .await
            .expect("Failed to connect to SQL database");

        tracing::info!(connection_string = ?database_url, "Connected to database");

        // Setup tables
        Migrator::up(&db, None)
            .await
            .expect("Failed to set up tables in the database");

        Self { db }
    }
}

impl From<event::Model> for Event {
    fn from(value: event::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_at: Utc.from_utc_datetime(&value.created_at),
            visited_at: Utc.from_utc_datetime(&value.visited_at),
            times: serde_json::from_value(value.times).unwrap_or(vec![]),
            timezone: value.timezone,
            event_type: serde_json::from_value(value.event_type).unwrap_or_default(),
        }
    }
}

impl From<person::Model> for Person {
    fn from(value: person::Model) -> Self {
        Self {
            name: value.name,
            password_hash: value.password_hash,
            created_at: Utc.from_utc_datetime(&value.created_at),
            availability: serde_json::from_value(value.availability).unwrap_or(vec![]),
        }
    }
}

#[derive(Display, Debug)]
pub enum SqlAdaptorError {
    DbErr(DbErr),
    TransactionError(TransactionError<DbErr>),
}

impl Error for SqlAdaptorError {}

impl From<DbErr> for SqlAdaptorError {
    fn from(value: DbErr) -> Self {
        Self::DbErr(value)
    }
}
impl From<TransactionError<DbErr>> for SqlAdaptorError {
    fn from(value: TransactionError<DbErr>) -> Self {
        Self::TransactionError(value)
    }
}
