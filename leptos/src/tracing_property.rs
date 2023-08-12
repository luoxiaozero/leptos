#[cfg_attr(debug_assertions, macro_export)]
/// Use for tracing property
macro_rules! tracing_property {
    ($($prop:tt),+ $(,)?) => {
        {
            use leptos::{Serialize, ser};
            let mut props = vec![];
            $(
                let prop = leptos::spez! {
                    for &$prop;
                    match<T: Serialize> &T  -> String {
                        ser(self.0.get().unwrap()).map_or_else(
                            |err| format!("{}", err),
                            |prop| format!(r#"{{"name": "{}", "value": {}}}"#, stringify!{$prop}, prop),
                        )
                    }
                    match<T> &T -> String {
                        format!(r#"{{"name": "{}"}}"#, stringify!{$prop})
                    }

                };
                props.push(prop);
            )*
            ser(&props).unwrap()
        }
    };
}
