use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub max_users: Option<i32>,
    pub max_licenses: Option<i32>,
    pub max_machines: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub owner_id: Option<String>,
}

impl From<keygen_rs::group::Group> for Group {
    fn from(g: keygen_rs::group::Group) -> Self {
        Group {
            id: g.id,
            name: g.name,
            max_users: g.max_users,
            max_licenses: g.max_licenses,
            max_machines: g.max_machines,
            metadata: g
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: g.created.to_rfc3339(),
            updated: g.updated.to_rfc3339(),
            account_id: g.account_id,
            owner_id: g.owner_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateGroupRequest {
    pub name: String,
    pub max_users: Option<i32>,
    pub max_licenses: Option<i32>,
    pub max_machines: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub max_users: Option<i32>,
    pub max_licenses: Option<i32>,
    pub max_machines: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListGroupsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
}

#[napi]
pub async fn create_group(request: CreateGroupRequest) -> Result<Group> {
    let req = keygen_rs::group::CreateGroupRequest {
        name: request.name,
        max_users: request.max_users,
        max_licenses: request.max_licenses,
        max_machines: request.max_machines,
        metadata: crate::opt_metadata(request.metadata)?,
    };
    keygen_rs::group::Group::create(req)
        .await
        .map(Group::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_groups(options: Option<ListGroupsOptions>) -> Result<Vec<Group>> {
    let opts = options.map(|o| keygen_rs::group::ListGroupsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });
    keygen_rs::group::Group::list(opts)
        .await
        .map(|list| list.into_iter().map(Group::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_group(id: String) -> Result<Group> {
    keygen_rs::group::Group::get(&id)
        .await
        .map(Group::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_group(id: String, request: UpdateGroupRequest) -> Result<Group> {
    let grp = keygen_rs::group::Group {
        id,
        ..Default::default()
    };
    let req = keygen_rs::group::UpdateGroupRequest {
        name: request.name,
        max_users: request.max_users,
        max_licenses: request.max_licenses,
        max_machines: request.max_machines,
        metadata: crate::opt_metadata(request.metadata)?,
    };
    grp.update(req)
        .await
        .map(Group::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_group(id: String) -> Result<()> {
    let grp = keygen_rs::group::Group {
        id,
        ..Default::default()
    };
    grp.delete().await.map_err(to_napi_error)
}
