#![deny(warnings)]

use std::{collections::HashSet, sync::{Arc, RwLock}};
use gethostname::gethostname;
use uuid::Uuid;
use local_ip_address::list_afinet_netifas;

use helper::CurlClient;
use serde::{Deserialize, Serialize};


#[path = "../benches/support/mod.rs"]
mod support;




use crate::{helper, proxy};

type ClusterNodeId = String;
pub type ClusterTouchpoint = String;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, Eq, PartialEq, Hash)]
pub struct ClusterNode {
    id: ClusterNodeId,
    host: String,
    ip_addresses: Vec<String>,
    port: u16,
}
impl ClusterNode {
    pub fn new(id: ClusterNodeId, host: String, ip_addresses: Vec<String>, port: u16) -> Self {
        Self {
            id,
            host: host,
            ip_addresses,
            port,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct Cluster {
    master: Option<ClusterNodeId>,
    touchpoints: HashSet<ClusterTouchpoint>,
    other_nodes: HashSet<ClusterNode>,
    local_node: ClusterNode,
}
impl Default for Cluster {
    fn default() -> Self {
        Self {
            master: None,
            touchpoints: HashSet::new(),
            other_nodes: HashSet::new(),
            local_node: ClusterNode::new(
                "error".to_string(),
                "error".to_string(),
                vec![],
                0,
            ),
        }
    }
}

impl Cluster {
    pub fn new(cluster_touchpoints: HashSet<ClusterTouchpoint>, port: u16) -> Self {

        let network_interfaces = list_afinet_netifas().unwrap();
        let ip_addesess = network_interfaces.iter().map(|(_, ip)| ip.to_string()).collect();

        Self {
            master: None,
            touchpoints: cluster_touchpoints,
            other_nodes: HashSet::new(),
            local_node: ClusterNode::new(
                Uuid::new_v4().to_string(),
                gethostname().to_str().unwrap().to_string(),
                ip_addesess,
                port,
            ),
        }
    }

    pub fn add_node(&mut self, node: ClusterNode) -> bool {
        self.other_nodes.insert(node);
        return true;
        }

    pub fn remove_node(&mut self, node_id: String) -> bool {
        self.other_nodes.retain(|node| node.id != node_id);
        return true;
    }

    pub fn add_touchpoint(&mut self, touchpoint: ClusterTouchpoint) -> bool {
        self.touchpoints.insert(touchpoint);
        return true;
    }

    pub fn remove_touchpoint(&mut self, touchpoint: ClusterTouchpoint) -> bool {
        self.touchpoints.remove(&touchpoint);
        return true;
    }
    
    /*
    pub async fn advertise(&self, touchpoint: ClusterTouchpoint) -> Result<Cluster, Box<dyn std::error::Error>> {

        let client = reqwest::Client::new();
        let resp = client.post(&touchpoint)
            .json(&self.local_node)
            .send()
            .await?
            .json::<Cluster>()
            .await?;
    
        //println!("{resp:#?}");


        return Ok(resp);
    }
    */
    /*

    pub fn discovery(&self) -> Vec<ClusterNode> {
        for touchpoint in self.touchpoints.iter() {
            self.advertise(touchpoint.clone());
        }
    }
    */
}

#[derive(Debug, Clone)]
pub struct Admin {
    curl_client: CurlClient,
    cluster: Cluster,
    proxy:Arc<RwLock<proxy::ProxyState>>,
}


impl Admin {        
    pub fn new(curl_client: CurlClient, proxy: Arc<RwLock<proxy::ProxyState>>, cluster_touchpoints: HashSet<ClusterTouchpoint>, port: u16) -> Result<Self, &'static str> {

        log::info!("Creating new Admin instance");


        return Ok(Self {
            curl_client,
            cluster: Cluster::new(cluster_touchpoints, port),
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
    use std::sync::{Arc, RwLock};
    use axum::{extract::{Path, State}, Router};
    use http::StatusCode;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    
    
    use utoipa_swagger_ui::SwaggerUi;

    use crate::{nordvpn::{NordVpnConnectOutput, NordVpnDisconnectOutput, NordVpnStatusOutput}, proxy::{ProxyRotateResult, ProxyRotationMode, ProxySettingsDrainUpdate, ProxyStatus, ProxyStatusCompact, RequestCountProxyRotation}};

    use super::{Cluster, ClusterNode};
    use super::super::Admin;

    use axum::Json;

    const IP_URL: &str = "https://api.ipify.org";
    type AdminState = Arc<RwLock<Admin>>;

    pub(super) fn admin_router(admin_state: AdminState) -> OpenApiRouter {
        return OpenApiRouter::new()
            .routes(routes!(get_rotation))
            .routes(routes!(set_rotation))
            .routes(routes!(get_public_ip))
            .with_state(admin_state);
    }
    
    pub(super) fn cluster_router(admin_state: AdminState) -> OpenApiRouter {
        return OpenApiRouter::new()
            //.routes(routes!(cluster_advertise))
            .routes(routes!(cluster_node_add))
            .routes(routes!(cluster_node_remove))
            .routes(routes!(cluster_state))
            .routes(routes!(cluster_touchpoint_add))
            .routes(routes!(cluster_touchpoint_remove))
            .with_state(admin_state);
    }


    pub(super) fn nordvpn_router(admin_state: AdminState) -> OpenApiRouter {
        return OpenApiRouter::new()
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
            .with_state(admin_state);
    }

    pub(super) fn proxy_router(admin_state: AdminState) -> OpenApiRouter {
        return OpenApiRouter::new()
            .routes(routes!(proxy_rotate))
            .routes(routes!(proxy_settings))
            .routes(routes!(proxy_settings_drain))
            .routes(routes!(proxy_settings_rotation))
            .routes(routes!(proxy_settings_rotation_interval))
            .routes(routes!(proxy_status))
            .routes(routes!(proxy_status_purge))
            .routes(routes!(proxy_status_compact))
            .routes(routes!(proxy_settings_rotation_set_requestcount))
            .routes(routes!(proxy_health))
            .with_state(admin_state);
    }

    pub fn router(admin_state: AdminState) -> Router {

        let (router, api) = OpenApiRouter::new()
        .nest("/api/admin", admin_router(admin_state.clone()))
        .nest("/api/cluster", cluster_router(admin_state.clone()))
        .nest("/api/nordvpn", nordvpn_router(admin_state.clone()))
        .nest("/api/proxy", proxy_router(admin_state))
        .split_for_parts();

        let router = router
            .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", api.clone()));
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
        Json(admin_state.read().unwrap().get_rotation())
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
        return Json(admin_state.write().unwrap().set_rotation_from_str(&mode, &value));
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
            (status = 200, description = "List all todos successfully", body = String)
        )
    )]
    async fn get_public_ip(State(admin_state): State<AdminState>) -> Json<String> {
        Json(admin_state.read().unwrap().curl_client.get(IP_URL).unwrap())
    }

    /*
    #[utoipa::path(
        post,
        path = "/advertise/{touchpoint}",
        responses(
            (status = 200, description = "Advertise this node towards the touchpoint", body = Cluster),
            (status = 500, description = "Failed to advertise", body = Cluster)
        ),
        params(
            ("touchpoint"  = String, Path, description = "Touchpoint")
        )
    )]
    async fn cluster_advertise(State(admin_state): State<AdminState>, Path(touchpoint): Path<String>) -> (StatusCode, Json<Cluster>) {
        match admin_state.read().unwrap().cluster.advertise(touchpoint.clone()).await {
            Ok(cluster) => (StatusCode::OK, Json(cluster)),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Cluster::default())),
        }
    }
    */

    #[utoipa::path(
        get,
        path = "/state",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "Show cluster state", body = Cluster)
        )
    )]
    async fn cluster_state(State(admin_state): State<AdminState>) -> Json<Cluster> {
        return Json(admin_state.read().unwrap().cluster.clone());
    }

    #[utoipa::path(
        post,
        path = "/node/add",
        responses(
            (status = 200, description = "Add Cluster Node", body = bool)
        ),
        request_body = ClusterNode
    )]
    async fn cluster_node_add(State(admin_state): State<AdminState>, Json(node): Json<ClusterNode>) -> Json<bool> {
        return Json(admin_state.write().unwrap().cluster.add_node(node));
    }

    #[utoipa::path(
        delete,
        path = "/node/remove/{node_id}",
        responses(
            (status = 200, description = "Remove Cluster Node", body = bool)
        ),
        params(
            ("node_id"  = String, Path, description = "Node ID")
        )
    )]
    async fn cluster_node_remove(Path(node_id): Path<String>, State(admin_state): State<AdminState>) -> Json<bool> {
        return Json(admin_state.write().unwrap().cluster.remove_node(node_id));
    }

    #[utoipa::path(
        post,
        path = "/touchpoint/add",
        responses(
            (status = 200, description = "Add Cluster Touchpoint", body = bool)
        ),
        params(
            ("touchpoint"  = String, Path, description = "Touchpoint")
        )
    )]
    async fn cluster_touchpoint_add(Path(touchpoint): Path<String>, State(admin_state): State<AdminState>) -> Json<bool> {
        return Json(admin_state.write().unwrap().cluster.add_touchpoint(touchpoint));
    }

    #[utoipa::path(
        delete,
        path = "/touchpoint/remove/{touchpoint}",
        responses(
            (status = 200, description = "Remove Cluster Touchpoint", body = bool)
        ),
        params(
            ("touchpoint"  = String, Path, description = "Touchpoint")
        )
    )]
    async fn cluster_touchpoint_remove(Path(touchpoint): Path<String>, State(admin_state): State<AdminState>) -> Json<bool> {
        return Json(admin_state.write().unwrap().cluster.remove_touchpoint(touchpoint));
    }

    #[utoipa::path(
        get,
        path = "/account",
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_account(State(admin_state): State<AdminState>) -> Json<String> {
        Json(admin_state.read().unwrap().proxy.read().unwrap().nordvpn.account().unwrap())
    }

    #[utoipa::path(
        post,
        path = "/connect",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = NordVpnConnectOutput)
        )
    )]
    async fn nordvpn_connect(State(admin_state): State<AdminState>) -> Json<NordVpnConnectOutput> {
        admin_state.read().unwrap().proxy.write().unwrap().drain();
        let resp = admin_state.read().unwrap().proxy.read().unwrap().nordvpn.connect(None);
        admin_state.read().unwrap().proxy.write().unwrap().activate();
        match resp {
            Ok(output) => {
                return Json(output)
            },
            Err(e) => {
                return Json(e)
            }
        }
    }

    #[utoipa::path(
        post,
        path = "/connect/{argument}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = NordVpnConnectOutput)
        ),
        params(
            ("argument"  = String, Path, description = "Argument for connection")
        )
    )]
    async fn nordvpn_connect_with_argument(Path(argument): Path<String>, State(admin_state): State<AdminState>) -> Json<NordVpnConnectOutput> {
        match admin_state.read().unwrap().proxy.read().unwrap().nordvpn.connect(Some(argument.clone())) {
            Ok(output) => {
                return Json(output)
            },
            Err(e) => {
                return Json(e)
            }
        }
    }

    #[utoipa::path(
        post,
        path = "/disconnect",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = NordVpnDisconnectOutput)
        )
    )]
    async fn nordvpn_disconnect(State(admin_state): State<AdminState>) -> Json<NordVpnDisconnectOutput> {
        match admin_state.read().unwrap().proxy.read().unwrap().nordvpn.disconnect() {
            Ok(output) => Json(output),
            Err(err) => {
                Json(err)
            }
        }
    }

    #[utoipa::path(
        get,
        path = "/logs",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = Vec<u8>)
        )
    )]
    async fn nordvpn_logs(State(admin_state): State<AdminState>) -> Json<Vec<u8>> {
        Json(admin_state.read().unwrap().proxy.read().unwrap().nordvpn.logs(10))
    }

    #[utoipa::path(
        get,
        path = "/logs/{lines}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        ),
        params(
            ("lines"  = i32, Path, description = "Number of lines")
        )
    )]
    async fn nordvpn_logs_with_argument(Path(lines): Path<u16>, State(admin_state): State<AdminState>) -> Json<String> {
            Json(serde_json::to_string(&admin_state.read().unwrap().proxy.read().unwrap().nordvpn.logs(lines)).unwrap())
        }

    #[utoipa::path(
        post,
        path = "/rotate",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = String)
        )
    )]
    async fn nordvpn_rotate(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.read().unwrap().proxy.write().unwrap().nordvpn.rotate()).unwrap())
    }

    #[utoipa::path(
        post,
        path = "/sanitize",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = String)
        )
    )]
    async fn nordvpn_sanitize(State(admin_state): State<AdminState>) -> Json<ProxyStatus> {
        let retention = Some(60);
        admin_state.read().unwrap().proxy.write().unwrap().sanitize(retention);
        return Json(admin_state.read().unwrap().proxy.read().unwrap().status());
    }

    #[utoipa::path(
        get,
        path = "/status",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = NordVpnStatusOutput)
        )
    )]
    async fn nordvpn_status(State(admin_state): State<AdminState>) -> Json<NordVpnStatusOutput> {
        Json(admin_state.read().unwrap().proxy.read().unwrap().nordvpn.status())
    }

    #[utoipa::path(
        post,
        path = "/daemon/restart",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_restart(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.read().unwrap().proxy.read().unwrap().nordvpn.daemon_restart(Some(30))))
    }

    #[utoipa::path(
        get,
        path = "/daemon/status",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_status(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.read().unwrap().proxy.read().unwrap().nordvpn.daemon_status().output))
    }

    #[utoipa::path(
        post,
        path = "/daemon/start",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_start(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.read().unwrap().proxy.read().unwrap().nordvpn.daemon_start(Some(30))))
    }

    #[utoipa::path(
        post,
        path = "/daemon/stop",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn nordvpn_daemon_stop(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", admin_state.read().unwrap().proxy.read().unwrap().nordvpn.daemon_stop()))
    }

    #[utoipa::path(
        get,
        path = "/health",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "Proxy is up", body = String),
            (status = 503, description = "Proxy is out of service", body = String)
        )
    )]
    async fn proxy_health(State(admin_state): State<AdminState>) -> (StatusCode, Json<String>) {
        match admin_state.read().unwrap().proxy.read().unwrap().drained {
            false => (StatusCode::OK, Json("Proxy is up".to_string())),
            true => (StatusCode::SERVICE_UNAVAILABLE, Json("Proxy is out of service".to_string())),
        }
    }

    #[utoipa::path(
        post,
        path = "/rotate",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = ProxyRotateResult),
            (status = 500, description = "List all todos successfully", body = ProxyRotateResult)
        )
    )]
    async fn proxy_rotate(State(admin_state): State<AdminState>) -> (StatusCode, Json<ProxyRotateResult>) {
        match admin_state.read().unwrap().proxy.write().unwrap().rotate() {
            Ok(result) => (StatusCode::OK ,Json(result)),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR ,Json(err))
        }
    }

    #[utoipa::path(
        get,
        path = "/settings",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_settings(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.read().unwrap().proxy.read().unwrap().settings).unwrap())
    }

    #[utoipa::path(
        post,
        path = "/settings/rotation/interval/{interval}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        ),
        params(
            ("interval"  = u16, Path, description = "Interval for rotation")
        )
    )]
    async fn proxy_settings_rotation_interval(Path(interval): Path<u16>, State(admin_state): State<AdminState>) -> Json<String> {
        let output = admin_state.read().unwrap().proxy.write().unwrap().set_rotation_interval(interval);
        Json(serde_json::to_string(&output).unwrap())
    }

    #[utoipa::path(
        post,
        path = "/settings/drain/{drain}",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = ProxySettingsDrainUpdate)
        ),
        params(
            ("drain"  = bool, Path, description = "Interval for rotation")
        )
    )]
    async fn proxy_settings_drain(Path(drain): Path<bool>, State(admin_state): State<AdminState>) -> Json<ProxySettingsDrainUpdate> {
        let before = admin_state.read().unwrap().proxy.read().unwrap().status().drained;
        if drain {
            admin_state.read().unwrap().proxy.write().unwrap().drain();
        } else {
            admin_state.read().unwrap().proxy.write().unwrap().activate();
        }
        Json(
            ProxySettingsDrainUpdate::new(  
                before,
                admin_state.read().unwrap().proxy.read().unwrap().status().drained
            )
        )
    }



    #[utoipa::path(
        get,
        path = "/status",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_status(State(admin_state): State<AdminState>) -> Json<String> {
        Json(format!("{:?}", serde_json::to_string(&admin_state.read().unwrap().proxy.read().unwrap().status()).unwrap()))
    }

    #[utoipa::path(
        post,
        path = "/status/purge",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_status_purge(State(admin_state): State<AdminState>) -> Json<String> {
        Json(serde_json::to_string(&admin_state.read().unwrap().proxy.write().unwrap().purge(None)).unwrap())
    }

    #[utoipa::path(
        get,
        path = "/status/compact",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [String])
        )
    )]
    async fn proxy_status_compact(State(admin_state): State<AdminState>) -> Json<ProxyStatusCompact> {
        return Json(admin_state.read().unwrap().proxy.read().unwrap().compact_status());
    }

    #[utoipa::path(
        get,
        path = "/settings/rotation",
        //tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = String)
        )
    )]
    async fn proxy_settings_rotation(State(admin_state): State<AdminState>) -> Json<ProxyRotationMode> {
        Json(admin_state.read().unwrap().proxy.read().unwrap().settings.rotation.clone())
    }

    #[utoipa::path(
        post,
        path = "/settings/rotation/requestcount/{count}",
        responses(
            (status = 200, description = "List all todos successfully", body = String)
        ),
        params(
            ("count"  = u16, Path, description = "Interval for rotation")
        )
    )]
    async fn proxy_settings_rotation_set_requestcount(Path(count): Path<u16>, State(admin_state): State<AdminState>) -> Json<ProxyRotationMode> {
        let rotation = ProxyRotationMode::RequestCount(RequestCountProxyRotation::new(count));
        admin_state.read().unwrap().proxy.write().unwrap().settings.rotation = rotation.clone();
        Json(rotation)
    }

}