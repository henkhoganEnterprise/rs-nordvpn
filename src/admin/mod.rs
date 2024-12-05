#![deny(warnings)]

use std::sync::{Arc, RwLock};



use helper::CurlClient;


#[path = "../benches/support/mod.rs"]
mod support;




use crate::{helper, proxy};


#[derive(Debug, Clone)]
pub struct Admin {
    curl_client: CurlClient,
    proxy:Arc<RwLock<proxy::ProxyState>>,
}


impl Admin {
    pub fn new(curl_client: CurlClient, proxy: Arc<RwLock<proxy::ProxyState>>) -> Result<Self, &'static str> {

        log::info!("Creating new Admin instance");

        return Ok(Self {
            curl_client,
            proxy,
        });
    }

    pub fn get_rotation(&self) -> String {
        log::debug!("Getting rotation...");
        return "rotation".to_string();
    }

    pub fn set_rotation_from_str(&self, rotation_type: &str, rotation_value: &str) -> String {
        log::debug!("Setting rotation to type: {:?}, value: {:?}", rotation_type, rotation_value);
        return "rotation".to_string();
    }

    //pub fn get_status(&self) -> bool {
    //    log::debug!("Getting status...");
    //    let output = self.nordvpn.account();
    //    return output;
    //}

    /* pub fn nord_account(&self) -> () {
        log::debug!("Checking NordVPN account...");
        let output = self.nordvpn.account();
        if output {
            log::info!("Account: {}", output);
        } else {
            log::error!("Failed to fetch account: {}", output);
        }
        return ();
    } */
}


pub mod restapi {
    use std::sync::Arc;
    use axum::{extract::Path, Router};
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    
    
    use utoipa_swagger_ui::SwaggerUi;
    use utoipa::OpenApi;
    


    use crate::proxy::ProxyStatusCompact;

    use super::super::Admin;

    use axum::{
        extract::State,
        Json,
    };

    const IP_URL: &str = "https://api.ipify.org";

    #[derive(OpenApi)]
    #[openapi(
        //modifiers(&SecurityAddon),
        tags(
            (name = "blabla", description = "Todo items management API")
        )
    )]
    struct ApiDoc;


    type AdminState = Arc<Admin>;

    pub(super) fn admin_router(admin_state: AdminState) -> OpenApiRouter {
        return OpenApiRouter::new()
            .routes(routes!(get_rotation))
            .routes(routes!(set_rotation))
            .routes(routes!(get_public_ip))
            .routes(routes!(nordvpn_account))
            .routes(routes!(nordvpn_connect))
            .routes(routes!(nordvpn_connect_with_argument))
            .routes(routes!(nordvpn_disconnect))
            .routes(routes!(nordvpn_logs))
            .routes(routes!(nordvpn_logs_with_argument))
            .routes(routes!(nordvpn_rotate))
            .routes(routes!(nordvpn_sanitize))
            .routes(routes!(nordvpn_status))
            .routes(routes!(nordvpn_daemon_status))
            .routes(routes!(nordvpn_daemon_restart))
            .routes(routes!(nordvpn_daemon_start))
            .routes(routes!(nordvpn_daemon_stop))
            .routes(routes!(proxy_rotate))
            .routes(routes!(proxy_settings))
            .routes(routes!(proxy_settings_rotation))
            .routes(routes!(proxy_settings_rotation_interval))
            .routes(routes!(proxy_status))
            .routes(routes!(proxy_status_purge))
            .routes(routes!(proxy_status_compact))

            //.routes(routes!(search_todos))
            //.routes(routes!(mark_done, delete_todo))
            .with_state(admin_state);
    }

    pub fn router(admin_state: AdminState) -> Router {

        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/admin", admin_router(admin_state))
        .split_for_parts();

        let router = router
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));
            /*
            .merge(Redoc::with_url("/redoc", api.clone()))
            // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
            // via SwaggerUi instead we only make rapidoc to point to the existing doc.
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            // Alternative to above
            // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
            .merge(Scalar::with_url("/scalar", api));
            */

        return router;
    }

    #[utoipa::path(
        get,
        path = "/admin/rotation",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn get_rotation(State(admin_state): State<AdminState>) -> Json<String> {
        Json(admin_state.get_rotation())
    }

    #[utoipa::path(
        post,
        path = "/admin/rotation/{mode}/{value}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        ),
        params(
            ("mode"  = String, Path, description = "Mode of rotation"),
            ("value" = String, Path, description = "Value of rotation")
        )
    )]
    async fn set_rotation(Path(mode): Path<String>, Path(value): Path<String>, State(admin_state): State<AdminState>) -> Json<String> {
        log::info!("mpde: {:?},  value: {:?}", mode, value);
        return Json(admin_state.set_rotation_from_str(&mode, &value));
    }

    /*
    #[utoipa::path(
        get,
        path = "/ip/local",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn get_local_ip(State(admin_state): State<AdminState>) -> Json<String> {
        Json(admin_state.curl_client.get(IP_URL).unwrap())
    }
    */

    #[utoipa::path(
        get,
        path = "/ip/public",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn get_public_ip(State(admin_state): State<AdminState>) -> Json<String> {
        Json(admin_state.curl_client.get(IP_URL).unwrap())
    }

    #[utoipa::path(
        get,
        path = "/nordvpn/account",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_account(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.proxy.read().unwrap().nordvpn.account()))
    }

    #[utoipa::path(
        post,
        path = "/nordvpn/connect",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_connect(State(admin_state): State<AdminState>) -> Json<String> {
        admin_state.proxy.write().unwrap().drain();
        let resp = serde_json::to_string(&admin_state.proxy.read().unwrap().nordvpn.connect(None)).unwrap();
        admin_state.proxy.write().unwrap().activate();
        Json(resp)
    }

    #[utoipa::path(
        post,
        path = "/nordvpn/connect/{argument}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        ),
        params(
            ("argument"  = String, Path, description = "Argument for connection")
        )
    )]
    async fn nordvpn_connect_with_argument(Path(argument): Path<String>, State(admin_state): State<AdminState>) -> Json<String> {
        let output = match admin_state.proxy.read().unwrap().nordvpn.connect(Some(argument.clone())) {
            Ok(output) => {
                log::info!("Connected with argument: {:?}", argument);
                serde_json::to_string(&output).unwrap()
            },
            Err(e) => {
                log::error!("Failed to connect with argument: {:?}", argument);
                format!("Failed to connect with argument: {:?}", e)
            }
        };
        Json(output)
    }

    #[utoipa::path(
        post,
        path = "/nordvpn/disconnect",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_disconnect(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.proxy.read().unwrap().nordvpn.disconnect()))
    }

    #[utoipa::path(
        get,
        path = "/nordvpn/logs",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_logs(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.proxy.read().unwrap().nordvpn.logs(10)).unwrap())
    }

    #[utoipa::path(
        get,
        path = "/nordvpn/logs/{lines}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        ),
        params(
            ("lines"  = i32, Path, description = "Number of lines")
        )
    )]
    async fn nordvpn_logs_with_argument(Path(lines): Path<u16>, State(admin_state): State<AdminState>) -> Json<String> {
            Json(serde_json::to_string(&admin_state.proxy.read().unwrap().nordvpn.logs(lines)).unwrap())
        }

    #[utoipa::path(
        post,
        path = "/nordvpn/rotate",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_rotate(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.proxy.write().unwrap().nordvpn.rotate()).unwrap())
    }

    #[utoipa::path(
        post,
        path = "/nordvpn/sanitize",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_sanitize(State(admin_state): State<AdminState>) -> Json<String> {
        let retention = Some(60);
        admin_state.proxy.write().unwrap().sanitize(retention);
        Json(serde_json::to_string(&admin_state.proxy.read().unwrap().status()).unwrap())
    }

    #[utoipa::path(
        get,
        path = "/nordvpn/status",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_status(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.proxy.read().unwrap().nordvpn.status()).unwrap())
    }

    #[utoipa::path(
        post,
        path = "/nordvpn/daemon/restart",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_restart(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.proxy.read().unwrap().nordvpn.daemon_restart(Some(30))))
    }

    #[utoipa::path(
        get,
        path = "/nordvpn/daemon/status",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_status(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.proxy.read().unwrap().nordvpn.daemon_status().output))
    }

    #[utoipa::path(
        post,
        path = "/nordvpn/daemon/start",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_start(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.proxy.read().unwrap().nordvpn.daemon_start(Some(30))))
    }

    #[utoipa::path(
        post,
        path = "/nordvpn/daemon/stop",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_stop(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.proxy.read().unwrap().nordvpn.daemon_stop()))
    }

    #[utoipa::path(
        post,
        path = "/proxy/rotate",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_rotate(State(admin_state): State<AdminState>) -> Json<String> {
        let resp = serde_json::to_string(&admin_state.proxy.write().unwrap().rotate()).unwrap();
        Json(resp)
    }

    #[utoipa::path(
        get,
        path = "/proxy/settings",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_settings(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.proxy.read().unwrap().settings).unwrap())
    }

    #[utoipa::path(
        post,
        path = "/proxy/settings/rotation/interval/{interval}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        ),
        params(
            ("interval"  = u16, Path, description = "Interval for rotation")
        )
    )]
    async fn proxy_settings_rotation_interval(Path(interval): Path<u16>, State(admin_state): State<AdminState>) -> Json<String> {
        let output = admin_state.proxy.write().unwrap().set_rotation_interval(interval);
        Json(serde_json::to_string(&output).unwrap())
    }

    #[utoipa::path(
        get,
        path = "/proxy/status",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_status(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", serde_json::to_string(&admin_state.proxy.read().unwrap().status()).unwrap()))
    }

    #[utoipa::path(
        post,
        path = "/proxy/status/purge",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_status_purge(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.proxy.write().unwrap().purge(None)).unwrap())
    }

    #[utoipa::path(
        get,
        path = "/proxy/status/compact",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]

    async fn proxy_status_compact(State(admin_state): State<AdminState>) -> Json<ProxyStatusCompact> {
        return Json(admin_state.proxy.read().unwrap().compact_status());
    }



    #[utoipa::path(
        get,
        path = "/proxy/settings/rotation",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_settings_rotation(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.proxy.read().unwrap().settings.rotation_interval).unwrap())
    }

}