use super::format::{ExampleFile, Pair, RequestFile};
use crate::models::example::ResponseExample;
use crate::models::request::RequestSpec;

impl RequestFile {
    pub fn from_spec(spec: RequestSpec, order: f64, examples: &[&ResponseExample]) -> Self {
        Self {
            docs: spec.docs,
            examples: examples
                .iter()
                .map(|e| ExampleFile {
                    id: e.id,
                    name: e.name.clone(),
                    status: e.status,
                    status_text: e.status_text.clone(),
                    headers: e
                        .headers
                        .iter()
                        .map(|(key, value)| Pair {
                            key: key.clone(),
                            value: value.clone(),
                        })
                        .collect(),
                    body: e.body.clone(),
                })
                .collect(),
            id: spec.id,
            name: spec.name,
            order,
            method: spec.method,
            url: spec.url,
            mode: spec.mode,
            params: spec.params,
            headers: spec.headers,
            body: spec.body,
            auth: spec.auth,
            settings: spec.settings,
            scripts: spec.scripts,
        }
    }

    /// The request, its sort order and its examples in file order.
    pub fn into_spec(self) -> (RequestSpec, f64, Vec<ResponseExample>) {
        let examples = self
            .examples
            .into_iter()
            .enumerate()
            .map(|(i, e)| ResponseExample {
                id: e.id,
                request_id: self.id,
                name: e.name,
                status: e.status,
                status_text: e.status_text,
                headers: e.headers.into_iter().map(|p| (p.key, p.value)).collect(),
                body: e.body,
                sort_order: (i + 1) as f64,
            })
            .collect();
        let spec = RequestSpec {
            docs: self.docs,
            id: self.id,
            name: self.name,
            method: self.method,
            url: self.url,
            params: self.params,
            headers: self.headers,
            body: self.body,
            auth: self.auth,
            settings: self.settings,
            scripts: self.scripts,
            mode: self.mode,
        };
        (spec, self.order, examples)
    }
}
