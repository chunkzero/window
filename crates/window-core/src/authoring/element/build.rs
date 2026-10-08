use super::{ElementDto, convert_layout_children};
use crate::authoring::art::ArtRefDto;
use crate::authoring::flex::parse_auto_flow;
use crate::authoring::parse::validate_name;
use crate::inventory::{InventorySlotSection, SlotRectClaim};
use crate::model::{Element, FlexBox, Region, SlotSection, Switch, SwitchCase};
use crate::{Error, Result};

impl ElementDto {
    pub(in crate::authoring) fn into_element(self) -> Result<Element> {
        let debug_name = self.debug_name.clone();
        self.build().map_err(|error| error.with_debug_name(debug_name.as_deref()))
    }

    fn build(self) -> Result<Element> {
        self.validate_fields()?;
        match self.kind.as_str() {
            "sprite" => self.build_sprite(),
            "sprite_slot" => self.build_sprite_slot(),
            "item" => self.build_item(),
            "collection" => self.build_collection(),
            "anvil_input" => self.build_anvil_input(),
            "label" => self.build_label(),
            "slot" => self.build_slot(),
            "flex" => self.build_flex(),
            "section" => self.build_section(),
            "switch" => self.build_switch(),
            "region" => self.build_region(),
            "case" => Err(Error::Validation("case element is only valid as a direct child of a switch".into())),
            other => Err(Error::Validation(format!("unknown element type `{other}`"))),
        }
    }

    fn build_sprite(self) -> Result<Element> {
        Ok(Element::Sprite { name: self.required("art")?, pos: self.pos()?, debug_name: self.debug_name })
    }

    fn build_sprite_slot(self) -> Result<Element> {
        Ok(Element::SpriteSlot {
            name: self.entry()?,
            size: self.size()?,
            pos: self.pos()?,
            align: self.text_align()?,
            debug_name: self.debug_name,
        })
    }

    fn build_item(self) -> Result<Element> {
        Ok(Element::Item {
            name: self.entry()?,
            slots: self.slots()?,
            pattern: self.pattern()?,
            debug_name: self.debug_name,
        })
    }

    fn build_collection(self) -> Result<Element> {
        Ok(Element::Collection {
            name: self.entry()?,
            slots: self.slots()?,
            pattern: self.pattern()?,
            frame: art_name(&self.frame)?,
            selected_sprite: art_name(&self.selected_sprite)?,
            action: self.action.unwrap_or(true),
            debug_name: self.debug_name,
        })
    }

    fn build_anvil_input(self) -> Result<Element> {
        Ok(Element::AnvilInput {
            name: self.entry()?,
            initial: self.initial.unwrap_or_default(),
            item_model: self.item_model,
            debug_name: self.debug_name,
        })
    }

    fn build_label(self) -> Result<Element> {
        Ok(Element::Label {
            text: self.required("text")?,
            width: self.width,
            pos: self.pos()?,
            style: self.text_style()?,
            debug_name: self.debug_name,
        })
    }

    fn build_slot(self) -> Result<Element> {
        Ok(Element::Slot {
            name: self.entry()?,
            width: self.width,
            pos: self.pos()?,
            style: self.text_style()?,
            fit: self.text_fit()?,
            debug_name: self.debug_name,
        })
    }

    fn build_flex(self) -> Result<Element> {
        Ok(Element::Flex(Box::new(FlexBox {
            pos: self.pos()?,
            frame: art_name(&self.frame)?,
            style: self.style.clone().unwrap_or_default().into_style()?,
            children: convert_layout_children(self.children)?,
            debug_name: self.debug_name,
        })))
    }

    fn build_section(self) -> Result<Element> {
        let section = match self.section.as_deref() {
            Some("container") => InventorySlotSection::Container,
            Some("player") => InventorySlotSection::Player,
            Some("hotbar") => InventorySlotSection::Hotbar,
            Some(other) => {
                return Err(Error::Validation(format!(
                    "section element has unknown section `{other}`; valid sections: container, player, hotbar"
                )));
            }
            None => return Err(Error::Validation("section element requires `section`".into())),
        };
        let claim = if self.claim.is_some() { self.claim()? } else { SlotRectClaim::Unowned };
        Ok(Element::Section(Box::new(SlotSection {
            section,
            frame: art_name(&self.frame)?,
            outset: self.outset.map(|o| o.into_insets()).unwrap_or_default(),
            claim,
            flow: self.flow.as_deref().map(parse_auto_flow).transpose()?.unwrap_or_default(),
            children: convert_layout_children(self.children)?,
            debug_name: self.debug_name,
        })))
    }

    fn build_switch(self) -> Result<Element> {
        let name = self.entry()?;
        let pos = self.pos()?;
        let mut cases: Vec<SwitchCase> = Vec::with_capacity(self.children.len());
        for case in self.children {
            if case.kind != "case" {
                return Err(Error::Validation(format!(
                    "switch `{name}` children must be case elements, found {}",
                    case.kind
                )));
            }
            case.validate_fields()?;
            if case.layout.is_some() {
                return Err(Error::Validation(format!("switch `{name}` case sets `layout`; set it on the switch")));
            }
            let value = case.required("value")?;
            validate_name(&value, &format!("switch `{name}` case"))?;
            if cases.iter().any(|c| c.value == value) {
                return Err(Error::Validation(format!("switch `{name}` has duplicate case `{value}`")));
            }
            let debug_name = case.debug_name.clone();
            let body = FlexBox {
                pos: None,
                frame: art_name(&case.frame).map_err(|error| error.with_debug_name(debug_name.as_deref()))?,
                style: case.style.clone().unwrap_or_default().into_style()?,
                children: convert_layout_children(case.children)
                    .map_err(|error| error.with_debug_name(debug_name.as_deref()))?,
                debug_name,
            };
            cases.push(SwitchCase { value, body });
        }
        if cases.is_empty() {
            return Err(Error::Validation(format!("switch `{name}` requires at least one case")));
        }
        Ok(Element::Switch(Box::new(Switch { name, pos, cases, debug_name: self.debug_name })))
    }

    fn build_region(self) -> Result<Element> {
        Ok(Element::Region(Box::new(Region {
            size: self.optional_size()?,
            tooltip: self.tooltip()?,
            default_action: self.default_action,
            item_model: self.item_model,
            name: self.name,
            debug_name: self.debug_name,
        })))
    }
}

/// The interned name of `art`.
fn art_name(art: &Option<ArtRefDto>) -> Result<Option<String>> {
    art.as_ref().map(ArtRefDto::name).transpose()
}
