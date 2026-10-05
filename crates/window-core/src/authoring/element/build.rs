use super::{ElementDto, convert_children, convert_layout_children};
use crate::authoring::flex::parse_auto_flow;
use crate::inventory::{InventorySlotSection, SlotRectClaim};
use crate::model::{Element, FlexBox, SlotSection};
use crate::{Error, Result};

impl ElementDto {
    pub(in crate::authoring) fn into_element(self) -> Result<Element> {
        self.validate_fields()?;
        match self.kind.as_str() {
            "panel" => self.build_panel(),
            "row" => self.build_row(),
            "column" => self.build_column(),
            "sprite" => self.build_sprite(),
            "sprite_slot" => self.build_sprite_slot(),
            "button" => self.build_button(),
            "hotspot" => self.build_hotspot(),
            "item" => self.build_item(),
            "collection" => self.build_collection(),
            "anvil_input" => self.build_anvil_input(),
            "slot_rects" => self.build_slot_rects(),
            "repeater" => self.build_repeater(),
            "label" => self.build_label(),
            "slot" => self.build_slot(),
            "flex" => self.build_flex(),
            "section" => self.build_section(),
            other => Err(Error::Validation(format!("unknown element type `{other}`"))),
        }
    }

    fn build_panel(self) -> Result<Element> {
        Ok(Element::Panel {
            frame: self.required("frame")?,
            pos: self.pos()?,
            size: self.size()?,
            padding: self.padding,
            children: convert_children(self.children)?,
        })
    }

    fn build_row(self) -> Result<Element> {
        Ok(Element::Row {
            pos: self.pos()?,
            gap: self.gap,
            padding: self.padding,
            align: self.cross_align()?,
            children: convert_children(self.children)?,
        })
    }

    fn build_column(self) -> Result<Element> {
        Ok(Element::Column {
            pos: self.pos()?,
            gap: self.gap,
            padding: self.padding,
            align: self.cross_align()?,
            children: convert_children(self.children)?,
        })
    }

    fn build_sprite(self) -> Result<Element> {
        Ok(Element::Sprite { name: self.required("name")?, pos: self.pos()? })
    }

    fn build_sprite_slot(self) -> Result<Element> {
        Ok(Element::SpriteSlot {
            name: self.required("name")?,
            size: self.size()?,
            pos: self.pos()?,
            align: self.text_align()?,
            sprite: self.sprite.clone(),
        })
    }

    fn build_button(self) -> Result<Element> {
        Ok(Element::Button {
            name: self.required("name")?,
            frame: self.frame.clone(),
            pos: self.pos()?,
            size: self.optional_size()?,
            slots: self.slots()?,
            pattern: self.pattern()?,
            padding: self.padding,
            default: self.button_default()?,
            tooltip: self.tooltip()?,
            states: self.states()?,
            children: convert_children(self.children)?,
        })
    }

    fn build_hotspot(self) -> Result<Element> {
        Ok(Element::Hotspot {
            name: self.required("name")?,
            pos: self.pos()?,
            size: self.optional_size()?,
            slots: self.slots()?,
            pattern: self.pattern()?,
            tooltip: self.tooltip()?,
            states: self.states()?,
        })
    }

    fn build_item(self) -> Result<Element> {
        Ok(Element::Item {
            name: self.required("name")?,
            slots: self.slots()?,
            pattern: self.pattern()?,
            cell_slot: self.cell_slot()?,
        })
    }

    fn build_collection(self) -> Result<Element> {
        Ok(Element::Collection {
            name: self.required("name")?,
            slots: self.slots()?,
            pattern: self.pattern()?,
            frame: self.frame.clone(),
            selected_sprite: self.selected_sprite.clone(),
            action: self.action.unwrap_or(true),
        })
    }

    fn build_anvil_input(self) -> Result<Element> {
        Ok(Element::AnvilInput {
            name: self.required("name")?,
            initial: self.initial.unwrap_or_default(),
            item_model: self.item_model,
        })
    }

    fn build_slot_rects(self) -> Result<Element> {
        Ok(Element::SlotRects {
            name: self.required("name")?,
            frame: self.frame.clone(),
            pattern: self.required_pattern()?,
            claim: self.claim()?,
        })
    }

    fn build_repeater(self) -> Result<Element> {
        Ok(Element::Repeater {
            name: self.required("name")?,
            pattern: self.required_pattern()?,
            frame: self.frame.clone(),
            padding: self.padding,
            children: convert_children(self.children)?,
        })
    }

    fn build_label(self) -> Result<Element> {
        Ok(Element::Label {
            text: self.required("text")?,
            width: self.width,
            pos: self.pos()?,
            style: self.text_style()?,
        })
    }

    fn build_slot(self) -> Result<Element> {
        Ok(Element::Slot {
            name: self.required("name")?,
            width: self.width,
            pos: self.pos()?,
            style: self.text_style()?,
        })
    }

    fn build_flex(self) -> Result<Element> {
        Ok(Element::Flex(Box::new(FlexBox {
            pos: self.pos()?,
            frame: self.frame.clone(),
            style: self.style.clone().unwrap_or_default().into_style()?,
            children: convert_layout_children(self.children)?,
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
            frame: self.frame.clone(),
            outset: self.outset.map(|o| o.into_insets()).unwrap_or_default(),
            claim,
            flow: self.flow.as_deref().map(parse_auto_flow).transpose()?.unwrap_or_default(),
            children: convert_layout_children(self.children)?,
        })))
    }
}
