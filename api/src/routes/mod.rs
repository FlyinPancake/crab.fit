use crate::Router;

pub mod event;
pub mod person;
pub mod stats;
pub mod tasks;

pub(super) fn router() -> Router {
    Router::new()
        .nest("/stats", stats::router())
        .nest("/event", event::router().merge(person::router()))
        .nest("/tasks", tasks::router())
}
