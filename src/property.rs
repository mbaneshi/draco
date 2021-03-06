use crate::S;
use std::borrow::Cow;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys as web;

#[derive(Debug)]
pub struct Property {
    name: S,
    value: Value,
}

impl Property {
    pub fn new(name: impl Into<S>, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    String(S),
    Bool(bool),
}

impl Property {
    pub fn patch(&self, old_property: Option<&Property>, element: &web::Element) {
        let set = |name, value| js_sys::Reflect::set(element, &JsValue::from_str(name), &value);
        let set_bool = |name, value| set(name, JsValue::from_bool(value));
        let set_str = |name, value| set(name, JsValue::from_str(value));
        let get = |name| js_sys::Reflect::get(element, &JsValue::from_str(name)).ok();
        let get_bool = |name| get(name).and_then(|value| value.as_bool());
        let get_string = |name| get(name).and_then(|value| value.as_string());
        match (&*self.name, &self.value) {
            ("checked", Value::Bool(new_checked)) => {
                if let Some(old_checked) = get_bool("checked") {
                    if old_checked != *new_checked {
                        set_bool("checked", *new_checked).unwrap_throw();
                    }
                }
            }
            ("value", Value::String(new_string)) => {
                if let Some(old_string) = get_string("value") {
                    if &old_string != new_string {
                        set_str("value", new_string).unwrap_throw();
                    }
                }
            }
            (name, value) => {
                if Some(&self.value) != old_property.map(|p| &p.value) {
                    match value {
                        Value::Bool(bool) => {
                            set_bool(name, *bool).unwrap_throw();
                        }
                        Value::String(string) => {
                            set_str(name, string).unwrap_throw();
                        }
                    }
                }
            }
        }
    }

    pub fn remove(&self, element: &web::Element) {
        js_sys::Reflect::set(element, &JsValue::from_str(&self.name), &JsValue::UNDEFINED)
            .unwrap_throw();
    }
}

impl From<&'static str> for Value {
    fn from(str: &'static str) -> Self {
        Value::String(str.into())
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::String(string.into())
    }
}

impl From<Cow<'static, str>> for Value {
    fn from(s: Cow<'static, str>) -> Self {
        Value::String(s)
    }
}

impl From<bool> for Value {
    fn from(bool: bool) -> Self {
        Value::Bool(bool)
    }
}
