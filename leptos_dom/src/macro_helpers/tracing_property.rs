#[cfg_attr(debug_assertions, macro_export)]
/// Use for tracing property
macro_rules! tracing_props {
    () => {
        String::from("[]")
    };
    ($($prop:tt),+ $(,)?) => {
        {
            use leptos::leptos_dom::tracing_property::{Match, SerializeMatch, DefalutMatch};
            let mut props = String::new();
            props.push('[');
            $(
                let prop = (&&Match {
                    name: stringify!{$prop},
                    value: std::cell::Cell::new(Some(&$prop))
                }).spez();
                props.push_str(&format!("{},", prop));
            )*
            props.pop();
            props.push(']');
            props
        }
    };
}

// Implementation based on spez
// see https://github.com/m-ou-se/spez

pub struct Match<T> {
    pub name: &'static str,
    pub value: std::cell::Cell<Option<T>>,
}

pub trait SerializeMatch {
    type Return;
    fn spez(&self) -> Self::Return;
}
impl<T: serde::Serialize> SerializeMatch for &Match<&T> {
    type Return = String;
    fn spez(&self) -> Self::Return {
        serde_json::to_string(self.value.get().unwrap()).map_or_else(
            |err| format!(r#"{{"name": "{}", "error": {}}}"#, self.name, err),
            |value| {
                format!(r#"{{"name": "{}", "value": {}}}"#, self.name, value)
            },
        )
    }
}

pub trait DefalutMatch {
    type Return;
    fn spez(&self) -> Self::Return;
}
impl<T> DefalutMatch for Match<&T> {
    type Return = String;
    fn spez(&self) -> Self::Return {
        format!(
            r#"{{"name": "{}", "error": "The trait `serde::Serialize` is not implemented"}}"#,
            self.name
        )
    }
}
