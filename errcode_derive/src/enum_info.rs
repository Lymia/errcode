use proc_macro2::Ident;
use venial::{Enum, Error, Fields};

pub struct EnumInfo {
    pub name: Ident,
    pub variants: Vec<EnumVariantInfo>,
}

pub struct EnumVariantInfo {
    pub name: Ident,
    pub repr: u32,
    pub message: Option<String>,
}

pub fn parse(item: &Enum) -> Result<EnumInfo, Error> {
    if item
        .generic_params
        .as_ref()
        .map_or(false, |x| !x.params.is_empty())
    {
        return Err(Error::new("#[derive(ErrorCode)] cannot be used on generic enums."));
    }

    let mut variants = Vec::new();
    for (i, (variant, _)) in item.variants.inner.iter().enumerate() {
        match &variant.fields {
            Fields::Unit => {}
            _ => {
                return Err(Error::new_at_span(
                    variant.span(),
                    "#[derive(EnumCode)] does not support variants with fields.",
                ));
            }
        }

        variants.push(EnumVariantInfo {
            name: variant.name.clone(),
            // TODO: Make sure repr matches the enum repr for optimization purposes.
            repr: i as u32,
            message: None,
        });
    }

    Ok(EnumInfo { name: item.name.clone(), variants })
}
