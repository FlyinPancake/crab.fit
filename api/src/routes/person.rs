use axum::{extract::Path, Extension, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use base64::{engine::general_purpose, Engine};
use common::{Adaptor, Person};
use utoipa_axum::routes;

use crate::{
    errors::ApiError,
    payloads::{ApiResult, PersonInput, PersonResponse},
    AdaptorExtension, Router,
};

pub fn router() -> Router {
    Router::new()
        .routes(routes!(get_person, update_person))
        .routes(routes!(get_people))
}

#[utoipa::path(
    get,
    path = "/{event_id}/people",
    params(
        ("event_id", description = "The ID of the event"),
    ),
    responses(
        (status = 200, description = "Ok", body = [PersonResponse]),
        (status = 404, description = "Event not found"),
        (status = 429, description = "Too many requests"),
    ),
    tag = "person",
)]
/// Get availabilities for an event
pub async fn get_people(
    Extension(adaptor): AdaptorExtension,
    Path(event_id): Path<String>,
) -> ApiResult<Vec<PersonResponse>> {
    let people = adaptor
        .get_people(event_id)
        .await
        .map_err(ApiError::AdaptorError)?;

    match people {
        Some(people) => Ok(Json(
            people
                .into_iter()
                .filter_map(|p| {
                    if !p.availability.is_empty() {
                        Some(p.into())
                    } else {
                        None
                    }
                })
                .collect(),
        )),
        None => Err(ApiError::NotFound),
    }
}

#[utoipa::path(
    get,
    path = "/{event_id}/people/{person_name}",
    params(
        ("event_id", description = "The ID of the event"),
        ("person_name", description = "The name of the person"),
    ),
    security((), ("password" = [])),
    responses(
        (status = 200, description = "Ok", body = PersonResponse),
        (status = 401, description = "Incorrect password"),
        (status = 404, description = "Event not found"),
        (status = 415, description = "Unsupported input format"),
        (status = 422, description = "Invalid input provided"),
        (status = 429, description = "Too many requests"),
    ),
    tag = "person",
)]
/// Login or create a person for an event
pub async fn get_person(
    Extension(adaptor): AdaptorExtension,
    Path((event_id, person_name)): Path<(String, String)>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> ApiResult<PersonResponse> {
    // Get inputted password
    let password = parse_password(bearer);

    let existing_people = adaptor
        .get_people(event_id.clone())
        .await
        .map_err(ApiError::AdaptorError)?;

    // Event not found
    if existing_people.is_none() {
        return Err(ApiError::NotFound);
    }

    // Check if the user already exists
    let existing_person = existing_people
        .unwrap()
        .into_iter()
        .find(|p| p.name.to_lowercase() == person_name.to_lowercase());

    match existing_person {
        // Login
        Some(p) => {
            // Verify password (if set)
            if verify_password(&p, password) {
                Ok(Json(p.into()))
            } else {
                Err(ApiError::NotAuthorized)
            }
        }
        // Signup
        None => {
            // Update stats
            adaptor
                .increment_stat_person_count()
                .await
                .map_err(ApiError::AdaptorError)?;

            Ok(Json(
                adaptor
                    .upsert_person(
                        event_id,
                        Person {
                            name: person_name,
                            password_hash: password
                                .map(|raw| bcrypt::hash(raw, 10).unwrap_or(String::from(""))),
                            created_at: chrono::offset::Utc::now(),
                            availability: vec![],
                        },
                    )
                    .await
                    .map_err(ApiError::AdaptorError)?
                    .unwrap()
                    .into(),
            ))
        }
    }
}

#[utoipa::path(
    patch,
    path = "/{event_id}/people/{person_name}",
    params(
        ("event_id", description = "The ID of the event"),
        ("person_name", description = "The name of the person"),
    ),
    security((), ("password" = [])),
    request_body(content = PersonInput, description = "Person details"),
    responses(
        (status = 200, description = "Ok", body = PersonResponse),
        (status = 401, description = "Incorrect password"),
        (status = 404, description = "Event or person not found"),
        (status = 415, description = "Unsupported input format"),
        (status = 422, description = "Invalid input provided"),
        (status = 429, description = "Too many requests"),
    ),
    tag = "person",
)]
/// Update a person's availabilities
pub async fn update_person(
    Extension(adaptor): AdaptorExtension,
    Path((event_id, person_name)): Path<(String, String)>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    Json(input): Json<PersonInput>,
) -> ApiResult<PersonResponse> {
    let existing_people = adaptor
        .get_people(event_id.clone())
        .await
        .map_err(ApiError::AdaptorError)?;

    // Event not found
    if existing_people.is_none() {
        return Err(ApiError::NotFound);
    }

    // Check if the user exists
    let existing_person = existing_people
        .unwrap()
        .into_iter()
        .find(|p| p.name.to_lowercase() == person_name.to_lowercase())
        .ok_or(ApiError::NotFound)?;

    // Verify password (if set)
    if !verify_password(&existing_person, parse_password(bearer)) {
        return Err(ApiError::NotAuthorized);
    }

    Ok(Json(
        adaptor
            .upsert_person(
                event_id,
                Person {
                    name: existing_person.name,
                    password_hash: existing_person.password_hash,
                    created_at: existing_person.created_at,
                    availability: input.availability,
                },
            )
            .await
            .map_err(ApiError::AdaptorError)?
            .unwrap()
            .into(),
    ))
}

pub fn parse_password(bearer: Option<TypedHeader<Authorization<Bearer>>>) -> Option<String> {
    bearer.map(|TypedHeader(Authorization(b))| {
        String::from_utf8(
            general_purpose::STANDARD
                .decode(b.token().trim())
                .unwrap_or(vec![]),
        )
        .unwrap_or("".to_owned())
    })
}

pub fn verify_password(person: &Person, raw: Option<String>) -> bool {
    match &person.password_hash {
        Some(hash) => bcrypt::verify(raw.unwrap_or("".to_owned()), hash).unwrap_or(false),
        // Specifically allow a user who doesn't have a password
        // set to log in with or without any password input
        None => true,
    }
}
