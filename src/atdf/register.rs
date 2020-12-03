use crate::atdf;
use crate::chip;
use crate::util;
use crate::ElementExt;
use std::collections::BTreeMap;

pub fn parse(
    el: &xmltree::Element,
    offset: usize,
    values: &atdf::values::ValueGroups,
) -> crate::Result<chip::Register> {
    let name = el.attr("name")?.clone();

    let description = el
        .attributes
        .get("caption")
        .and_then(|d| if !d.is_empty() { Some(d) } else { None })
        .cloned();

    // XMEGA chips and onwards use "rw" as access arrtibute
    let access = el.attributes.get("ocd-rw").or(el.attributes.get("rw"));

    let access = if let Some(access) = access {
        match access.as_ref() {
            "R" => chip::AccessMode::ReadOnly,
            "W" => chip::AccessMode::WriteOnly,
            "RW" => chip::AccessMode::ReadWrite,
            "" => {
                log::warn!("empty access-mode on {}", el.debug());
                chip::AccessMode::ReadWrite
            }
            _ => panic!("unknown access mode {:?}", access),
        }
    } else {
        chip::AccessMode::ReadWrite
    };

    let fields: BTreeMap<String, chip::Field> = el
        .children
        .iter()
        .filter(|c| c.name == "bitfield")
        .map(|e| atdf::field::parse(e, values))
        .map(|r| r.map(|f| (f.name.clone(), f)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    Ok(chip::Register {
        name,
        description,
        address: util::parse_int(el.attr("offset")?)? + offset,
        size: util::parse_int(el.attr("size")?)?,
        access,
        restriction: if fields.is_empty() {
            chip::ValueRestriction::Any
        } else {
            chip::ValueRestriction::Unsafe
        },
        fields,
    })
}
