//! Procedural macros for `rkyv`.

#![deny(broken_intra_doc_links)]
#![deny(missing_docs)]
#![deny(missing_crate_level_docs)]

mod archive;
mod attributes;
mod deserialize;
mod repr;
mod serialize;
mod util;
mod with;

extern crate proc_macro;

use syn::{parse_macro_input, DeriveInput};

/// Derives `Archive` for the labeled type.
///
/// # Attributes
///
/// Additional arguments can be specified using the `#[archive(...)]` and `#[archive_attr(...)]`
/// attributes.
///
/// `#[archive(...)]` takes the following arguments:
///
/// - `name`, `name = "..."`: Exposes the archived type with the given name. If used without a name
///   assignment, uses the name `"Archived" + name`.
/// - `repr(...)`: Sets the representation for the archived type to the given representation.
///   Available representation options may vary depending on features and type layout.
/// - `compare(...)`: Implements common comparison operators between the original and archived
///   types. Supported comparisons are `PartialEq` and `PartialOrd` (i.e.
///   `#[archive(compare(PartialEq, PartialOrd))]`).
/// - `bound(...)`: Adds additional bounds to the `Serialize` and `Deserialize` implementations.
///   This can be especially useful when dealing with recursive structures, where bounds may need to
///   be omitted to prevent recursive type definitions.
///
/// `#[archive_attr(...)]` adds the attributes passed as arguments as attributes to the generated
/// type. This is commonly used with attributes like `derive(...)` to derive trait implementations
/// for the archived type.
///
/// # Recursive types
///
/// This derive macro automatically adds a type bound `field: Archive` for each field type. This can
/// cause an overflow while evaluating trait bounds if the structure eventually references its own
/// type, as the implementation of `Archive` for a struct depends on each field type implementing it
/// as well. Adding the attribute `#[omit_bounds]` to a field will suppress this trait bound and
/// allow recursive structures. This may be too coarse for some types, in which case additional type
/// bounds may be required with `bound(...)`.
///
/// # Wrappers
///
/// Wrappers transparently customize archived types by providing different implementations of core
/// traits. For example, references cannot be archived, but the `Inline` wrapper serializes a
/// reference as if it were a field of the struct. Wrappers can be applied to fields using the
/// `#[with(...)]` attribute. Mutliple wrappers can be used, and they are applied in reverse order
/// (i.e. `#[with(A, B, C)]` will archive `MyType` as `With<With<With<MyType, C>, B, A>`).
#[proc_macro_derive(Archive, attributes(archive, archive_attr, omit_bounds, with))]
pub fn derive_archive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match archive::derive(parse_macro_input!(input as DeriveInput)) {
        Ok(result) => result.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Derives `Serialize` for the labeled type.
///
/// This macro also supports the `#[archive]`, `#[omit_bounds]`, and `#[with]` attributes. See
/// [`Archive`] for more information.
#[proc_macro_derive(Serialize, attributes(archive, omit_bounds, with))]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match serialize::derive(parse_macro_input!(input as DeriveInput)) {
        Ok(result) => result.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Derives `Deserialize` for the labeled type.
///
/// This macro also supports the `#[archive]`, `#[omit_bounds]`, and `#[with]` attributes. See
/// [`Archive`] for more information.
#[proc_macro_derive(Deserialize, attributes(archive, omit_bounds, with))]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match deserialize::derive(parse_macro_input!(input as DeriveInput)) {
        Ok(result) => result.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
