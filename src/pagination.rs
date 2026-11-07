//! Cursor pagination shared by every list endpoint.
//!
//! Every list call made through a [`KeygenClient`](crate::KeygenClient) service returns a
//! [`Page`]. Pass [`Page::next_cursor`] back as the `page_cursor` list option to fetch the
//! next page.

use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Smallest page size accepted by the Keygen API.
pub const MIN_PAGE_SIZE: u32 = 1;
/// Largest page size accepted by the Keygen API.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Limit and cursor pagination for relationship lists.
#[derive(Debug, Default, Serialize)]
pub struct PaginationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
}

/// One page of a list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    /// The resources on this page.
    pub data: Vec<T>,
    /// The raw `meta` object of the response.
    pub meta: Option<Value>,
    /// The raw `links` object of the response. Use [`Page::typed_links`] to read it.
    pub links: Option<Value>,
}

impl<T> Page<T> {
    /// The resources on this page.
    pub fn items(&self) -> &[T] {
        &self.data
    }

    /// Consumes the page and returns its resources.
    pub fn into_items(self) -> Vec<T> {
        self.data
    }

    pub fn typed_links(&self) -> Result<Option<PageLinks>, serde_json::Error> {
        self.links.clone().map(serde_json::from_value).transpose()
    }

    /// The cursor of the next page, or `None` on the last page.
    pub fn next_cursor(&self) -> Option<String> {
        self.typed_links().ok().flatten()?.next_cursor()
    }

    /// Whether the response links to a next page.
    pub fn has_more(&self) -> bool {
        self.typed_links()
            .ok()
            .flatten()
            .is_some_and(|links| links.has_next_page())
    }

    #[cfg(feature = "token")]
    pub(crate) fn map<U>(self, f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            data: self.data.into_iter().map(f).collect(),
            meta: self.meta,
            links: self.links,
        }
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PageLinks {
    #[serde(rename = "self", default, skip_serializing_if = "Option::is_none")]
    self_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    previous: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    first: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last: Option<String>,
    #[serde(flatten)]
    extensions: Map<String, Value>,
}

impl PageLinks {
    pub fn self_url(&self) -> Option<&str> {
        self.self_url.as_deref()
    }

    pub fn next_url(&self) -> Option<&str> {
        self.next.as_deref()
    }

    pub fn previous_url(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    pub fn first_url(&self) -> Option<&str> {
        self.first.as_deref()
    }

    pub fn last_url(&self) -> Option<&str> {
        self.last.as_deref()
    }

    pub fn extensions(&self) -> &Map<String, Value> {
        &self.extensions
    }

    pub fn has_next_page(&self) -> bool {
        self.next.is_some()
    }

    pub fn next_cursor(&self) -> Option<String> {
        let query = self.next.as_deref()?.split_once('?')?.1;
        url::form_urlencoded::parse(query.as_bytes())
            .find_map(|(key, value)| (key == "page[cursor]").then(|| value.into_owned()))
    }
}

/// A JSON:API list document, before its resources are converted.
#[derive(Debug, Deserialize)]
pub(crate) struct ListDocument<A> {
    pub data: Vec<KeygenResponseData<A>>,
    #[serde(default)]
    pub meta: Option<Value>,
    #[serde(default)]
    pub links: Option<Value>,
}

impl<A> ListDocument<A> {
    pub(crate) fn into_page<T>(self, f: impl FnMut(KeygenResponseData<A>) -> T) -> Page<T> {
        Page {
            data: self.data.into_iter().map(f).collect(),
            meta: self.meta,
            links: self.links,
        }
    }
}

pub(crate) fn validate_page_size(page_size: i64) -> Result<(), Error> {
    if (i64::from(MIN_PAGE_SIZE)..=i64::from(MAX_PAGE_SIZE)).contains(&page_size) {
        Ok(())
    } else {
        Err(Error::InvalidPageSize(page_size))
    }
}

/// Builds the `limit`, `page[size]` and `page[cursor]` query parameters.
///
/// Keygen starts cursor pagination with an empty cursor, so `page[cursor]` is sent
/// whenever a page size or a cursor is given. Fails when the page size is out of range.
pub(crate) fn cursor_pagination_query(
    limit: Option<i32>,
    page_size: Option<i32>,
    page_cursor: Option<&str>,
    default_limit: Option<i32>,
) -> Result<Value, Error> {
    if let Some(page_size) = page_size {
        validate_page_size(page_size.into())?;
    }
    let mut query = json!({});
    if let Some(limit) = limit.or(default_limit) {
        query["limit"] = json!(limit);
    }
    if page_size.is_some() || page_cursor.is_some() {
        query["page[cursor]"] = json!(page_cursor.unwrap_or_default());
    }
    if let Some(page_size) = page_size {
        query["page[size]"] = json!(page_size);
    }
    Ok(query)
}

/// Builds the query of a list request from options that serialize to a flat map.
///
/// The options keep their filter keys. `page[size]` and `page[cursor]` are rewritten
/// with [`cursor_pagination_query`]. When `page[number]` is set, the request keeps
/// page-number pagination and no cursor is added. Returns `None` when no parameter is set,
/// so the request URL carries no empty query string.
pub(crate) fn list_query<O: Serialize>(options: Option<&O>) -> Result<Option<Value>, Error> {
    let mut query = match options {
        Some(options) => serde_json::to_value(options)?,
        None => json!({}),
    };
    let map = query.as_object_mut().ok_or_else(|| {
        Error::UnexpectedError("list options must serialize to a map".to_string())
    })?;

    let page_size = map.get("page[size]").and_then(Value::as_i64);
    if let Some(page_size) = page_size {
        validate_page_size(page_size)?;
    }
    if map.contains_key("page[number]") {
        map.remove("page[cursor]");
        return Ok(Some(query));
    }

    map.remove("page[size]");
    let page_cursor = map
        .remove("page[cursor]")
        .and_then(|value| value.as_str().map(str::to_owned));
    // The size was validated above, so it fits in an i32.
    let page_size = page_size.map(|size| size as i32);
    if let Value::Object(pagination) =
        cursor_pagination_query(None, page_size, page_cursor.as_deref(), None)?
    {
        map.extend(pagination);
    }
    Ok((!map.is_empty()).then_some(query))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Options {
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
        #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
        page_size: Option<u32>,
        #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
        page_number: Option<u32>,
        #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
        page_cursor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        product: Option<String>,
    }

    fn options() -> Options {
        Options {
            limit: None,
            page_size: None,
            page_number: None,
            page_cursor: None,
            product: None,
        }
    }

    #[test]
    fn list_query_starts_cursor_pagination_with_an_empty_cursor() {
        let query = list_query(Some(&Options {
            page_size: Some(25),
            product: Some("prod".into()),
            ..options()
        }))
        .unwrap();

        assert_eq!(
            query.unwrap(),
            json!({ "page[size]": 25, "page[cursor]": "", "product": "prod" })
        );
    }

    #[test]
    fn list_query_passes_the_cursor() {
        let query = list_query(Some(&Options {
            page_size: Some(10),
            page_cursor: Some("abc".into()),
            ..options()
        }))
        .unwrap();

        assert_eq!(
            query.unwrap(),
            json!({ "page[size]": 10, "page[cursor]": "abc" })
        );
    }

    #[test]
    fn list_query_keeps_page_number_pagination() {
        let query = list_query(Some(&Options {
            page_size: Some(10),
            page_number: Some(2),
            limit: Some(5),
            ..options()
        }))
        .unwrap();

        assert_eq!(
            query.unwrap(),
            json!({ "page[size]": 10, "page[number]": 2, "limit": 5 })
        );
    }

    #[test]
    fn list_query_rejects_out_of_range_page_sizes() {
        for size in [0, 101] {
            let result = list_query(Some(&Options {
                page_size: Some(size),
                ..options()
            }));
            assert!(matches!(result, Err(Error::InvalidPageSize(s)) if s == i64::from(size)));
        }
    }

    #[test]
    fn list_query_without_options_is_empty() {
        assert_eq!(list_query::<Options>(None).unwrap(), None);
    }

    #[test]
    fn page_reports_next_cursor_and_has_more() {
        let first: Page<u8> = Page {
            data: vec![1],
            meta: None,
            links: Some(json!({ "next": "/v1/products?page[size]=1&page[cursor]=p1" })),
        };
        let last: Page<u8> = Page {
            data: vec![2],
            meta: None,
            links: Some(json!({ "next": null })),
        };

        assert_eq!(first.next_cursor().as_deref(), Some("p1"));
        assert!(first.has_more());
        assert_eq!(last.next_cursor(), None);
        assert!(!last.has_more());
        assert_eq!(last.into_iter().collect::<Vec<_>>(), vec![2]);
    }
}
