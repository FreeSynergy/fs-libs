// fsn-bus/src/transform.rs — Event payload transformation.

use crate::error::BusError;
use crate::event::Event;

// ── Transform trait ───────────────────────────────────────────────────────────

/// Transforms one [`Event`] into another — useful for payload mapping,
/// enrichment, or format conversion between pipeline stages.
///
/// Implement this trait to plug custom transformation logic into the bus.
pub trait Transform: Send + Sync {
    /// Apply this transform to `event`, returning the (possibly modified) event.
    fn transform(&self, event: &Event) -> Result<Event, BusError>;
}

// ── ChainTransform ────────────────────────────────────────────────────────────

/// Chains multiple [`Transform`]s, applying them left-to-right.
pub struct ChainTransform {
    transforms: Vec<Box<dyn Transform>>,
}

impl ChainTransform {
    /// Build a chain from a list of transforms.
    pub fn new(transforms: Vec<Box<dyn Transform>>) -> Self {
        Self { transforms }
    }
}

impl Transform for ChainTransform {
    fn transform(&self, event: &Event) -> Result<Event, BusError> {
        let mut current = event.clone();
        for t in &self.transforms {
            current = t.transform(&current)?;
        }
        Ok(current)
    }
}

// ── TeraTransform (feature: tera-transform) ───────────────────────────────────

/// Transforms the JSON payload of an event through a Tera template.
///
/// The template receives the event payload fields as variables.
/// The rendered output becomes the new `payload` as a JSON string.
///
/// Requires the `tera-transform` feature.
#[cfg(feature = "tera-transform")]
pub struct TeraTransform {
    /// Tera instance with the template pre-registered.
    engine: tera::Tera,
    /// Name of the template to render.
    template_name: String,
}

#[cfg(feature = "tera-transform")]
impl TeraTransform {
    /// Build a [`TeraTransform`] from a template string.
    ///
    /// `name` is used to identify the template inside the Tera engine.
    pub fn new(name: impl Into<String>, template: &str) -> Result<Self, BusError> {
        let name = name.into();
        let mut engine = tera::Tera::default();
        engine
            .add_raw_template(&name, template)
            .map_err(|e| BusError::transform(e.to_string()))?;
        Ok(Self { engine, template_name: name })
    }
}

#[cfg(feature = "tera-transform")]
impl Transform for TeraTransform {
    fn transform(&self, event: &Event) -> Result<Event, BusError> {
        let ctx = tera::Context::from_value(event.payload.clone())
            .map_err(|e| BusError::transform(e.to_string()))?;

        let rendered = self
            .engine
            .render(&self.template_name, &ctx)
            .map_err(|e| BusError::transform(e.to_string()))?;

        let new_payload = serde_json::Value::String(rendered);
        let mut new_event = event.clone();
        new_event.payload = new_payload;
        Ok(new_event)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;

    struct UpperPayload;
    impl Transform for UpperPayload {
        fn transform(&self, event: &Event) -> Result<Event, BusError> {
            let mut e = event.clone();
            if let serde_json::Value::String(s) = &event.payload {
                e.payload = serde_json::Value::String(s.to_uppercase());
            }
            Ok(e)
        }
    }

    #[test]
    fn chain_applies_in_order() {
        let chain = ChainTransform::new(vec![Box::new(UpperPayload), Box::new(UpperPayload)]);
        let mut ev = Event::new("test", "t", "hello").unwrap();
        ev.payload = serde_json::Value::String("hello".into());
        let out = chain.transform(&ev).unwrap();
        assert_eq!(out.payload, serde_json::Value::String("HELLO".into()));
    }

    #[cfg(feature = "tera-transform")]
    #[test]
    fn tera_transform_renders_template() {
        let t = TeraTransform::new("greet", "Hello {{ name }}!").unwrap();
        let ev = Event::new("greet", "test", serde_json::json!({ "name": "World" })).unwrap();
        let out = t.transform(&ev).unwrap();
        assert_eq!(out.payload, serde_json::Value::String("Hello World!".into()));
    }
}
