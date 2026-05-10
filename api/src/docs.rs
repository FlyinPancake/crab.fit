use utoipa::openapi::security::ApiKey;
use utoipa::openapi::security::ApiKeyValue;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(title = "Crab Fit API"),
    tags(
        (name = "info"),
        (name = "event"),
        (name = "person"),
        (name = "tasks"),
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

// Add password auth spec
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().add_security_scheme(
            "password",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("base64")
                    .build(),
            ),
        );
        openapi.components.as_mut().unwrap().add_security_scheme(
            "cron-key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Cron-Key"))),
        );
    }
}
