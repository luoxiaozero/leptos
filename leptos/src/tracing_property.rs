#[cfg_attr(debug_assertions, macro_export)]
/// Use for tracing property
macro_rules! tracing_property {
    ($($prop:tt),+ $(,)?) => {
        {
            use leptos::Serializable;
            let mut props = vec![];
            $(
                let prop = leptos::spez! {
                    for &$prop;
                    match<T: Serializable> &T  -> String {
                        self.0.get().unwrap().ser().map_or_else(
                            |err| format!("\"{}\"", err),
                            |prop| format!("{{\"name\": \"{}\", \"value\": \"{}\"}}", stringify!{$prop}, prop),
                        )
                    }
                    match<T> &T -> String {
                        format!("{{\"name\": \"{}\", \"value\": \"None\"}}", stringify!{$prop})
                    }
                    match<T> T -> String {
                        format!("{{\"name\": \"{}\", \"value\": \"None\"}}", stringify!{$prop})
                    }
                };
                props.push(prop);
            )*
            props.ser().unwrap()
        }
    };
}
