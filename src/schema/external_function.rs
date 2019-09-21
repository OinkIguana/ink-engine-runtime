use inventory::collect;

use super::Value;

pub trait TryAsRef<T> where T: ?Sized {
    type Error;
    fn try_as_ref(&self) -> Result<&T, Self::Error>;
}

pub struct ExternalFunction {
    pub name: &'static str,
    pub handler: fn(&[Value]) -> Value,
}

collect!(ExternalFunction);

#[macro_export]
macro_rules! ink_external {
    { fn $name:ident($($param:ident : &$type:ty),*$(,)?) -> $ret:ty $body:block } => {
        fn $name(mut params: &[crate::schema::Value]) -> crate::schema::Value {
            $(
                if params.len() == 0 {
                    panic!("Too few arguments passed to EXTERNAL Ink function $name");
                }
                let $param: &$type = params[0].try_as_ref().expect(&format!("Invalid passed to EXTERNAL Ink function $name: Expected $type, received {:?}", params[0]));
                params = &params[1..];
            )*

            if !params.is_empty() {
                panic!("Extra arguments passed to EXTERNAL Ink function $name: {} extra", params.len());
            }
            let result: $ret = $body;
            crate::schema::Value::from(result)
        }

        inventory::submit! {
            crate::schema::ExternalFunction {
                name: stringify!($name),
                handler: $name,
            }
        }
    }
}
