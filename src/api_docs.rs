use crate::{handler, models};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handler::model::calculate_volume
    ),
    components(
        schemas(
            // calculate_volume
            models::mdl::CalculateVolumeReq,
            models::mdl::CalculateVolumeRes,

            // generic error response
            models::error::ResponseError,
        )
    ),
    tags(
        (name = "3D model slicer for PolyVoxel", description = "API for calculating properties of 3D models" )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth", // This name must match the one in #[utoipa::path]
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
