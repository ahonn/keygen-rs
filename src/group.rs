#[cfg(feature = "token")]
use crate::client::Client;
#[cfg(feature = "token")]
use crate::errors::Error;
#[cfg(feature = "token")]
use crate::insert_optional;
#[cfg(feature = "token")]
use crate::keygen_client::KeygenClient;
#[cfg(feature = "token")]
use crate::license::{License, LicenseAttributes, PaginationOptions};
#[cfg(feature = "token")]
use crate::machine::{Machine, MachineAttributes};
#[cfg(feature = "token")]
use crate::pagination::{list_query, ListDocument, Page};
#[cfg(feature = "token")]
use crate::user::{User, UserAttributes};
use crate::KeygenResponseData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAttributes {
    pub name: String,
    #[serde(rename = "maxUsers")]
    pub max_users: Option<i32>,
    #[serde(rename = "maxLicenses")]
    pub max_licenses: Option<i32>,
    #[serde(rename = "maxMachines")]
    pub max_machines: Option<i32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GroupResponse {
    pub data: KeygenResponseData<GroupAttributes>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    #[serde(rename = "maxUsers")]
    pub max_users: Option<i32>,
    #[serde(rename = "maxLicenses")]
    pub max_licenses: Option<i32>,
    #[serde(rename = "maxMachines")]
    pub max_machines: Option<i32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListGroupsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Cursor returned by [`Page::next_cursor`](crate::pagination::Page::next_cursor).
    /// Ignored when `page_number` is set.
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "maxUsers", skip_serializing_if = "Option::is_none")]
    pub max_users: Option<i32>,
    #[serde(rename = "maxLicenses", skip_serializing_if = "Option::is_none")]
    pub max_licenses: Option<i32>,
    #[serde(rename = "maxMachines", skip_serializing_if = "Option::is_none")]
    pub max_machines: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Group {
    pub id: String,
    pub name: String,
    pub max_users: Option<i32>,
    pub max_licenses: Option<i32>,
    pub max_machines: Option<i32>,
    #[cfg_attr(
        feature = "specta",
        specta(type = Option<crate::specta_support::JsonMetadata>)
    )]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
    pub owner_id: Option<String>,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            max_users: None,
            max_licenses: None,
            max_machines: None,
            metadata: None,
            created: Utc::now(),
            updated: Utc::now(),
            account_id: None,
            owner_id: None,
        }
    }
}

impl Group {
    #[allow(dead_code)]
    pub(crate) fn from(data: KeygenResponseData<GroupAttributes>) -> Group {
        Group {
            id: data.id,
            name: data.attributes.name,
            max_users: data.attributes.max_users,
            max_licenses: data.attributes.max_licenses,
            max_machines: data.attributes.max_machines,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
            owner_id: data
                .relationships
                .owner
                .as_ref()
                .and_then(|o| o.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a new group
    #[cfg(feature = "token")]
    pub async fn create(request: CreateGroupRequest) -> Result<Group, Error> {
        Self::create_with_client(&Client::from_global_config()?, request).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn create_with_client(
        client: &Client,
        request: CreateGroupRequest,
    ) -> Result<Group, Error> {
        let mut attributes = serde_json::Map::new();
        attributes.insert("name".to_string(), serde_json::Value::String(request.name));

        insert_optional(&mut attributes, "maxUsers", request.max_users)?;
        insert_optional(&mut attributes, "maxLicenses", request.max_licenses)?;
        insert_optional(&mut attributes, "maxMachines", request.max_machines)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = serde_json::json!({
            "data": {
                "type": "groups",
                "attributes": attributes
            }
        });

        let response = client.post("groups", Some(&body), None::<&()>).await?;
        let group_response: GroupResponse = serde_json::from_value(response.body)?;
        Ok(Group::from(group_response.data))
    }

    /// List groups with optional pagination and filtering
    #[cfg(feature = "token")]
    pub async fn list(options: Option<ListGroupsOptions>) -> Result<Vec<Group>, Error> {
        let page = Self::list_with_client(&Client::from_global_config()?, options.as_ref()).await?;
        Ok(page.into_items())
    }

    #[cfg(feature = "token")]
    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListGroupsOptions>,
    ) -> Result<Page<Group>, Error> {
        let query = list_query(options)?;
        let response = client.get("groups", query.as_ref()).await?;
        let document: ListDocument<GroupAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(Group::from))
    }

    /// Get a group by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<Group, Error> {
        Self::get_with_client(&Client::from_global_config()?, id).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<Group, Error> {
        let endpoint = format!("groups/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let group_response: GroupResponse = serde_json::from_value(response.body)?;
        Ok(Group::from(group_response.data))
    }

    /// Update a group
    #[cfg(feature = "token")]
    pub async fn update(&self, request: UpdateGroupRequest) -> Result<Group, Error> {
        Self::update_with_client(&Client::from_global_config()?, &self.id, request).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn update_with_client(
        client: &Client,
        id: &str,
        request: UpdateGroupRequest,
    ) -> Result<Group, Error> {
        let endpoint = format!("groups/{id}");

        let mut attributes = serde_json::Map::new();
        insert_optional(&mut attributes, "name", request.name)?;
        insert_optional(&mut attributes, "maxUsers", request.max_users)?;
        insert_optional(&mut attributes, "maxLicenses", request.max_licenses)?;
        insert_optional(&mut attributes, "maxMachines", request.max_machines)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = serde_json::json!({
            "data": {
                "type": "groups",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let group_response: GroupResponse = serde_json::from_value(response.body)?;
        Ok(Group::from(group_response.data))
    }

    /// Delete a group
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        Self::delete_with_client(&Client::from_global_config()?, &self.id).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn delete_with_client(client: &Client, id: &str) -> Result<(), Error> {
        let endpoint = format!("groups/{id}");
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    #[cfg(feature = "token")]
    async fn list_related<A, T>(
        client: &Client,
        path: &str,
        options: Option<&PaginationOptions>,
        convert: impl FnMut(KeygenResponseData<A>) -> T,
    ) -> Result<Page<T>, Error>
    where
        A: serde::de::DeserializeOwned,
    {
        let query = list_query(options)?;
        let response = client.get(path, query.as_ref()).await?;
        let document: ListDocument<A> = serde_json::from_value(response.body)?;
        Ok(document.into_page(convert))
    }

    #[cfg(feature = "token")]
    pub(crate) async fn owners_with_client(
        client: &Client,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<User>, Error> {
        Self::list_related::<UserAttributes, _>(
            client,
            &format!("groups/{id}/owners"),
            options,
            User::from,
        )
        .await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn users_with_client(
        client: &Client,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<User>, Error> {
        Self::list_related::<UserAttributes, _>(
            client,
            &format!("groups/{id}/users"),
            options,
            User::from,
        )
        .await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn licenses_with_client(
        client: &Client,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<License>, Error> {
        Self::list_related::<LicenseAttributes, _>(
            client,
            &format!("groups/{id}/licenses"),
            options,
            License::from,
        )
        .await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn machines_with_client(
        client: &Client,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<Machine>, Error> {
        Self::list_related::<MachineAttributes, _>(
            client,
            &format!("groups/{id}/machines"),
            options,
            Machine::from,
        )
        .await
    }

    /// List group owners.
    #[cfg(feature = "token")]
    pub async fn owners(&self, options: Option<&PaginationOptions>) -> Result<Vec<User>, Error> {
        let page =
            Self::owners_with_client(&Client::from_global_config()?, &self.id, options).await?;
        Ok(page.into_items())
    }

    /// List group users.
    #[cfg(feature = "token")]
    pub async fn users(&self, options: Option<&PaginationOptions>) -> Result<Vec<User>, Error> {
        let page =
            Self::users_with_client(&Client::from_global_config()?, &self.id, options).await?;
        Ok(page.into_items())
    }

    /// List group licenses.
    #[cfg(feature = "token")]
    pub async fn licenses(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<Vec<License>, Error> {
        let page =
            Self::licenses_with_client(&Client::from_global_config()?, &self.id, options).await?;
        Ok(page.into_items())
    }

    /// List group machines.
    #[cfg(feature = "token")]
    pub async fn machines(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<Vec<Machine>, Error> {
        let page =
            Self::machines_with_client(&Client::from_global_config()?, &self.id, options).await?;
        Ok(page.into_items())
    }
}

/// Group operations bound to one [`KeygenClient`].
#[cfg(feature = "token")]
pub struct GroupService<'a> {
    client: &'a KeygenClient,
}

#[cfg(feature = "token")]
impl<'a> GroupService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateGroupRequest) -> Result<Group, Error> {
        Group::create_with_client(&self.client.transport, request).await
    }

    pub async fn list(&self, options: Option<&ListGroupsOptions>) -> Result<Page<Group>, Error> {
        Group::list_with_client(&self.client.transport, options).await
    }

    pub async fn get(&self, id: &str) -> Result<Group, Error> {
        Group::get_with_client(&self.client.transport, id).await
    }

    pub async fn update(&self, id: &str, request: UpdateGroupRequest) -> Result<Group, Error> {
        Group::update_with_client(&self.client.transport, id, request).await
    }

    pub async fn delete(&self, id: &str) -> Result<(), Error> {
        Group::delete_with_client(&self.client.transport, id).await
    }

    pub async fn owners(
        &self,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<User>, Error> {
        Group::owners_with_client(&self.client.transport, id, options).await
    }

    pub async fn users(
        &self,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<User>, Error> {
        Group::users_with_client(&self.client.transport, id, options).await
    }

    /// Licenses of the group. Each license is bound to this client.
    pub async fn licenses(
        &self,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<License>, Error> {
        let page = Group::licenses_with_client(&self.client.transport, id, options).await?;
        Ok(page.map(|license| {
            license.with_client(self.client.transport_arc(), self.client.config_arc())
        }))
    }

    /// Machines of the group. Each machine carries this client's configuration.
    pub async fn machines(
        &self,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<Machine>, Error> {
        let page = Group::machines_with_client(&self.client.transport, id, options).await?;
        Ok(page.map(|machine| machine.with_config_arc(self.client.config_arc())))
    }
}

#[cfg(all(test, feature = "token"))]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_group_relationships() {
        let group_data = KeygenResponseData {
            id: "test-group-id".to_string(),
            r#type: "groups".to_string(),
            attributes: GroupAttributes {
                name: "Premium Team".to_string(),
                max_users: Some(10),
                max_licenses: Some(50),
                max_machines: Some(100),
                metadata: Some({
                    let mut map = HashMap::new();
                    map.insert(
                        "tier".to_string(),
                        serde_json::Value::String("premium".to_string()),
                    );
                    map
                }),
                created: "2023-01-01T00:00:00Z".parse().unwrap(),
                updated: "2023-01-01T00:00:00Z".parse().unwrap(),
            },
            relationships: KeygenRelationships {
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                owner: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "users".to_string(),
                        id: "test-owner-id".to_string(),
                    }),
                    links: None,
                }),
                ..Default::default()
            },
        };

        let group = Group::from(group_data);

        assert_eq!(group.account_id, Some("test-account-id".to_string()));
        assert_eq!(group.owner_id, Some("test-owner-id".to_string()));
        assert_eq!(group.id, "test-group-id");
        assert_eq!(group.name, "Premium Team");
        assert_eq!(group.max_users, Some(10));
        assert_eq!(group.max_licenses, Some(50));
        assert_eq!(group.max_machines, Some(100));
        assert!(group.metadata.is_some());
    }

    #[test]
    fn test_group_without_relationships() {
        let group_data = KeygenResponseData {
            id: "test-group-id".to_string(),
            r#type: "groups".to_string(),
            attributes: GroupAttributes {
                name: "Basic Group".to_string(),
                max_users: None,
                max_licenses: None,
                max_machines: None,
                metadata: None,
                created: "2023-01-01T00:00:00Z".parse().unwrap(),
                updated: "2023-01-01T00:00:00Z".parse().unwrap(),
            },
            relationships: KeygenRelationships::default(),
        };

        let group = Group::from(group_data);

        assert_eq!(group.account_id, None);
        assert_eq!(group.owner_id, None);
        assert_eq!(group.name, "Basic Group");
        assert_eq!(group.max_users, None);
        assert_eq!(group.max_licenses, None);
        assert_eq!(group.max_machines, None);
        assert_eq!(group.metadata, None);
    }

    #[test]
    fn test_create_group_request_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "department".to_string(),
            serde_json::Value::String("engineering".to_string()),
        );

        let request = CreateGroupRequest {
            name: "Engineering Team".to_string(),
            max_users: Some(25),
            max_licenses: Some(100),
            max_machines: Some(200),
            metadata: Some(metadata),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"name\":\"Engineering Team\""));
        assert!(serialized.contains("\"maxUsers\":25"));
        assert!(serialized.contains("\"maxLicenses\":100"));
        assert!(serialized.contains("\"maxMachines\":200"));
        assert!(serialized.contains("\"metadata\""));
    }

    #[test]
    fn test_update_group_request_serialization() {
        let request = UpdateGroupRequest {
            name: Some("Updated Team Name".to_string()),
            max_users: Some(30),
            max_licenses: None,
            max_machines: Some(250),
            metadata: None,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"name\":\"Updated Team Name\""));
        assert!(serialized.contains("\"maxUsers\":30"));
        assert!(serialized.contains("\"maxMachines\":250"));
        // Should not contain null fields
        assert!(!serialized.contains("\"maxLicenses\":null"));
        assert!(!serialized.contains("\"metadata\":null"));
    }

    #[test]
    fn test_list_groups_options_serialization() {
        let options = ListGroupsOptions {
            limit: Some(20),
            page_size: Some(10),
            page_number: Some(3),
            page_cursor: None,
        };

        let serialized = serde_json::to_string(&options).unwrap();
        assert!(serialized.contains("\"limit\":20"));
        assert!(serialized.contains("\"page[size]\":10"));
        assert!(serialized.contains("\"page[number]\":3"));
    }

    #[test]
    fn test_group_attributes_serde() {
        // Test serialization/deserialization of GroupAttributes
        let attributes = GroupAttributes {
            name: "Test Group".to_string(),
            max_users: Some(15),
            max_licenses: Some(75),
            max_machines: Some(150),
            metadata: Some({
                let mut map = HashMap::new();
                map.insert(
                    "region".to_string(),
                    serde_json::Value::String("us-east".to_string()),
                );
                map
            }),
            created: "2023-01-01T00:00:00Z".parse().unwrap(),
            updated: "2023-01-02T00:00:00Z".parse().unwrap(),
        };

        let serialized = serde_json::to_string(&attributes).unwrap();
        let deserialized: GroupAttributes = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, attributes.name);
        assert_eq!(deserialized.max_users, attributes.max_users);
        assert_eq!(deserialized.max_licenses, attributes.max_licenses);
        assert_eq!(deserialized.max_machines, attributes.max_machines);
        assert_eq!(deserialized.metadata, attributes.metadata);
    }

    #[test]
    fn test_group_default() {
        let default_group = Group::default();

        assert_eq!(default_group.id, "");
        assert_eq!(default_group.name, "");
        assert_eq!(default_group.max_users, None);
        assert_eq!(default_group.max_licenses, None);
        assert_eq!(default_group.max_machines, None);
        assert_eq!(default_group.metadata, None);
        assert_eq!(default_group.account_id, None);
        assert_eq!(default_group.owner_id, None);
        // created and updated should be set to current time
        assert!(default_group.created <= Utc::now());
        assert!(default_group.updated <= Utc::now());
    }

    #[test]
    fn test_group_default_in_struct_syntax() {
        // Test that Default works with struct update syntax (useful for license_file.rs)
        let group = Group {
            id: "test-id".to_string(),
            name: "Test Group".to_string(),
            max_users: Some(5),
            ..Default::default()
        };

        assert_eq!(group.id, "test-id");
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.max_users, Some(5));
        assert_eq!(group.max_licenses, None);
        assert_eq!(group.max_machines, None);
        assert_eq!(group.metadata, None);
        assert_eq!(group.account_id, None);
        assert_eq!(group.owner_id, None);
    }
}
