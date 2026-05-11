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
