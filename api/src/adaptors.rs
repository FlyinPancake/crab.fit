use std::env;

use ambassador::Delegate;
use chrono::{DateTime, Utc};
use common::{Adaptor, AdaptorError, Event, Person, Stats, ambassador_impl_Adaptor};
use datastore_adaptor::DatastoreAdaptor;
use memory_adaptor::MemoryAdaptor;
use sql_adaptor::SqlAdaptor;

#[derive(Delegate)]
#[delegate(Adaptor)]
pub enum AnyAdaptor {
    Sql(SqlAdaptor),
    Memory(MemoryAdaptor),
    Datastore(DatastoreAdaptor),
}

impl From<SqlAdaptor> for AnyAdaptor {
    fn from(a: SqlAdaptor) -> Self {
        Self::Sql(a)
    }
}
impl From<MemoryAdaptor> for AnyAdaptor {
    fn from(a: MemoryAdaptor) -> Self {
        Self::Memory(a)
    }
}
impl From<DatastoreAdaptor> for AnyAdaptor {
    fn from(a: DatastoreAdaptor) -> Self {
        Self::Datastore(a)
    }
}

pub async fn create_adaptor() -> AnyAdaptor {
    match env::var("ADAPTOR").as_deref() {
        Ok("sql") => sql_adaptor::SqlAdaptor::new().await.into(),
        Ok("datastore") => datastore_adaptor::DatastoreAdaptor::new().await.into(),
        Ok("memory") => memory_adaptor::MemoryAdaptor::new().await.into(),
        _ => {
            tracing::warn!("defaulting to in-memory datastore");
            memory_adaptor::MemoryAdaptor::new().await.into()
        }
    }
}
