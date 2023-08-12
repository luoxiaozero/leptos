#[cfg_attr(debug_assertions, macro_export)]
/// Use for tracing property
macro_rules! tracing_props {
    ($($prop:tt),+ $(,)?) => {
        {
            use leptos::leptos_dom::tracing_property::{Serialize, ser, spez};
            let mut props = String::new();
            props.push('[');
            $(
                let prop = spez! {
                    for &$prop;
                    match<T: Serialize> &T  -> String {
                        ser(self.0.get().unwrap()).map_or_else(
                            |err| format!(r#"{{"name": "{}", "error": {}}}"#, stringify!{$prop}, err),
                            |prop| format!(r#"{{"name": "{}", "value": {}}}"#, stringify!{$prop}, prop),
                        )
                    }
                    match<T> &T -> String {
                        format!(r#"{{"name": "{}", "error": "The trait `serde::Serialize` is not implemented"}}"#, stringify!{$prop})
                    }

                };
                props.push_str(&format!("{},", prop));
            )*
            if props.len() > 1 {
                props.pop();
            }
            props.push(']');
            props
        }
    };
}

pub use serde::Serialize;
pub use spez::spez;
/// ser
pub fn ser<T>(value: &T) -> Result<std::string::String, serde_json::Error>
where
    T: ?Sized + Serialize,
{
    serde_json::to_string(value)
}
