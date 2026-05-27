use setu_message::TypeDefinition;
use type_id::TypeRegistry;

mod typescript;

pub fn generate<T: TypeDefinition>() {
    let mut r = TypeRegistry::new();
    let tys = T::type_definition(&mut r);
    
    println!("type_definition: {tys:#?}");
    println!("TypeRegistry: {r:#?}");
}
