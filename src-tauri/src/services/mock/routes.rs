//! Turns saved requests and their examples into routes, and matches incoming requests.
use crate::models::collection::SavedRequest;
use crate::models::example::ResponseExample;
use crate::models::mock::MockRouteInfo;

#[derive(Debug, Clone, PartialEq)]
enum Segment {
    Literal(String),
    Param(String),
}

#[derive(Debug, Clone)]
pub struct Route {
    pub method: String,
    segments: Vec<Segment>,
    pub request_name: String,
    pub examples: Vec<ResponseExample>,
}

/// The path part of a request URL: the scheme and host, or a leading `{{baseUrl}}`, and the
/// query are dropped. `{{id}}` and `:id` segments become parameters.
fn segments(url: &str) -> Vec<Segment> {
    let mut rest = url.trim();
    if rest.starts_with("{{")
        && let Some(end) = rest.find("}}")
    {
        rest = &rest[end + 2..];
    } else if let Some(after) = rest.split_once("://").map(|(_, r)| r) {
        rest = after.find('/').map_or("", |i| &after[i..]);
    }
    let path = rest.split(['?', '#']).next().unwrap_or_default();
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if let Some(name) = s.strip_prefix(':') {
                Segment::Param(name.into())
            } else if let Some(name) = s.strip_prefix("{{").and_then(|n| n.strip_suffix("}}")) {
                Segment::Param(name.trim().into())
            } else {
                Segment::Literal(s.into())
            }
        })
        .collect()
}

pub fn build(requests: &[SavedRequest], examples: &[ResponseExample]) -> Vec<Route> {
    let mut routes: Vec<Route> = requests
        .iter()
        .filter(|r| r.spec.mode == Default::default())
        .map(|r| {
            let mut own: Vec<ResponseExample> = examples
                .iter()
                .filter(|e| e.request_id == r.spec.id)
                .cloned()
                .collect();
            own.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order));
            Route {
                method: format!("{:?}", r.spec.method).to_uppercase(),
                segments: segments(&r.spec.url),
                request_name: r.spec.name.clone(),
                examples: own,
            }
        })
        .collect();
    // Routes with more fixed segments win, so /users/me beats /users/:id.
    routes.sort_by_key(|r| {
        std::cmp::Reverse(
            r.segments
                .iter()
                .filter(|s| matches!(s, Segment::Literal(_)))
                .count(),
        )
    });
    routes
}

pub fn info(routes: &[Route]) -> Vec<MockRouteInfo> {
    routes
        .iter()
        .map(|r| MockRouteInfo {
            method: r.method.clone(),
            path: format!(
                "/{}",
                r.segments
                    .iter()
                    .map(|s| match s {
                        Segment::Literal(t) => t.clone(),
                        Segment::Param(p) => format!(":{p}"),
                    })
                    .collect::<Vec<_>>()
                    .join("/")
            ),
            request_name: r.request_name.clone(),
            example: r.examples.first().map(|e| e.name.clone()),
            status: r.examples.first().map(|e| e.status),
        })
        .collect()
}

/// The route for a method and path; a matching path with another method is not a match.
pub fn find<'a>(routes: &'a [Route], method: &str, path: &str) -> Option<&'a Route> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    routes.iter().find(|r| {
        r.method.eq_ignore_ascii_case(method)
            && r.segments.len() == parts.len()
            && r.segments.iter().zip(&parts).all(|(s, p)| match s {
                Segment::Literal(t) => t == p,
                Segment::Param(_) => true,
            })
    })
}

/// The example to serve: the one named by `x-zorp-example`, otherwise the first.
pub fn pick<'a>(route: &'a Route, wanted: Option<&str>) -> Option<&'a ResponseExample> {
    wanted
        .and_then(|name| {
            route
                .examples
                .iter()
                .find(|e| e.name.eq_ignore_ascii_case(name.trim()))
        })
        .or_else(|| route.examples.first())
}
