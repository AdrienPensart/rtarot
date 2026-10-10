//! The card faces, images embedded in the binary and uploaded to the GPU once.

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use rtarot_core::{Card, Normal, Suit};

/// Every face, in slot order: excuse, trumps 1 to 21, then each suit ace to king.
const FACES: [&[u8]; 78] = [
    include_bytes!("../assets/cards/excuse.png"),
    include_bytes!("../assets/cards/trump_01.png"),
    include_bytes!("../assets/cards/trump_02.png"),
    include_bytes!("../assets/cards/trump_03.png"),
    include_bytes!("../assets/cards/trump_04.png"),
    include_bytes!("../assets/cards/trump_05.png"),
    include_bytes!("../assets/cards/trump_06.png"),
    include_bytes!("../assets/cards/trump_07.png"),
    include_bytes!("../assets/cards/trump_08.png"),
    include_bytes!("../assets/cards/trump_09.png"),
    include_bytes!("../assets/cards/trump_10.png"),
    include_bytes!("../assets/cards/trump_11.png"),
    include_bytes!("../assets/cards/trump_12.png"),
    include_bytes!("../assets/cards/trump_13.png"),
    include_bytes!("../assets/cards/trump_14.png"),
    include_bytes!("../assets/cards/trump_15.png"),
    include_bytes!("../assets/cards/trump_16.png"),
    include_bytes!("../assets/cards/trump_17.png"),
    include_bytes!("../assets/cards/trump_18.png"),
    include_bytes!("../assets/cards/trump_19.png"),
    include_bytes!("../assets/cards/trump_20.png"),
    include_bytes!("../assets/cards/trump_21.png"),
    include_bytes!("../assets/cards/heart_01.png"),
    include_bytes!("../assets/cards/heart_02.png"),
    include_bytes!("../assets/cards/heart_03.png"),
    include_bytes!("../assets/cards/heart_04.png"),
    include_bytes!("../assets/cards/heart_05.png"),
    include_bytes!("../assets/cards/heart_06.png"),
    include_bytes!("../assets/cards/heart_07.png"),
    include_bytes!("../assets/cards/heart_08.png"),
    include_bytes!("../assets/cards/heart_09.png"),
    include_bytes!("../assets/cards/heart_10.png"),
    include_bytes!("../assets/cards/heart_11.png"),
    include_bytes!("../assets/cards/heart_12.png"),
    include_bytes!("../assets/cards/heart_13.png"),
    include_bytes!("../assets/cards/heart_14.png"),
    include_bytes!("../assets/cards/spade_01.png"),
    include_bytes!("../assets/cards/spade_02.png"),
    include_bytes!("../assets/cards/spade_03.png"),
    include_bytes!("../assets/cards/spade_04.png"),
    include_bytes!("../assets/cards/spade_05.png"),
    include_bytes!("../assets/cards/spade_06.png"),
    include_bytes!("../assets/cards/spade_07.png"),
    include_bytes!("../assets/cards/spade_08.png"),
    include_bytes!("../assets/cards/spade_09.png"),
    include_bytes!("../assets/cards/spade_10.png"),
    include_bytes!("../assets/cards/spade_11.png"),
    include_bytes!("../assets/cards/spade_12.png"),
    include_bytes!("../assets/cards/spade_13.png"),
    include_bytes!("../assets/cards/spade_14.png"),
    include_bytes!("../assets/cards/diamond_01.png"),
    include_bytes!("../assets/cards/diamond_02.png"),
    include_bytes!("../assets/cards/diamond_03.png"),
    include_bytes!("../assets/cards/diamond_04.png"),
    include_bytes!("../assets/cards/diamond_05.png"),
    include_bytes!("../assets/cards/diamond_06.png"),
    include_bytes!("../assets/cards/diamond_07.png"),
    include_bytes!("../assets/cards/diamond_08.png"),
    include_bytes!("../assets/cards/diamond_09.png"),
    include_bytes!("../assets/cards/diamond_10.png"),
    include_bytes!("../assets/cards/diamond_11.png"),
    include_bytes!("../assets/cards/diamond_12.png"),
    include_bytes!("../assets/cards/diamond_13.png"),
    include_bytes!("../assets/cards/diamond_14.png"),
    include_bytes!("../assets/cards/club_01.png"),
    include_bytes!("../assets/cards/club_02.png"),
    include_bytes!("../assets/cards/club_03.png"),
    include_bytes!("../assets/cards/club_04.png"),
    include_bytes!("../assets/cards/club_05.png"),
    include_bytes!("../assets/cards/club_06.png"),
    include_bytes!("../assets/cards/club_07.png"),
    include_bytes!("../assets/cards/club_08.png"),
    include_bytes!("../assets/cards/club_09.png"),
    include_bytes!("../assets/cards/club_10.png"),
    include_bytes!("../assets/cards/club_11.png"),
    include_bytes!("../assets/cards/club_12.png"),
    include_bytes!("../assets/cards/club_13.png"),
    include_bytes!("../assets/cards/club_14.png"),
];

/// The slot of a card in [`FACES`].
const fn slot(card: Card) -> usize {
    match card {
        Card::Trump(trump) => trump as usize,
        Card::Normal(normal) => 22 + suit_slot(*normal.suit()) * 14 + value_slot(normal),
    }
}

const fn suit_slot(suit: Suit) -> usize {
    match suit {
        Suit::Heart => 0,
        Suit::Spade => 1,
        Suit::Diamond => 2,
        Suit::Club => 3,
    }
}

const fn value_slot(normal: Normal) -> usize {
    *normal.value() as usize - 1
}

fn decode(ctx: &Context, name: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    Some(ctx.load_texture(name, color, TextureOptions::LINEAR))
}

pub struct Textures {
    faces: Vec<Option<TextureHandle>>,
}

impl Textures {
    /// Decode every face. One that fails to decode is simply drawn by hand.
    pub fn load(ctx: &Context) -> Self {
        let faces = FACES
            .iter()
            .enumerate()
            .map(|(index, bytes)| decode(ctx, &format!("card{index}"), bytes))
            .collect();
        Self { faces }
    }

    pub fn face(&self, card: Card) -> Option<&TextureHandle> {
        self.faces.get(slot(card)).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::{FACES, slot};
    use rtarot_core::Deck;
    use std::collections::BTreeSet;

    #[test]
    fn every_card_has_its_own_face() {
        let slots: BTreeSet<usize> = Deck::full().iter().map(|card| slot(*card)).collect();
        assert_eq!(slots.len(), FACES.len());
        assert_eq!(slots.iter().next_back(), Some(&(FACES.len() - 1)));
    }

    #[test]
    fn every_face_decodes() {
        let textures = super::Textures::load(&egui::Context::default());
        let missing: Vec<String> = Deck::full()
            .iter()
            .filter(|card| textures.face(**card).is_none())
            .map(ToString::to_string)
            .collect();
        assert!(missing.is_empty(), "undecodable faces: {missing:?}");
    }

    #[test]
    fn trump_slots_follow_the_file_names() {
        use rtarot_core::{Card, Suit, SuitValue, Trump};
        assert_eq!(slot(Card::Trump(Trump::Fool)), 0);
        assert_eq!(slot(Card::Trump(Trump::_21)), 21);
        assert_eq!(slot(Card::normal(Suit::Heart, SuitValue::_1)), 22);
        assert_eq!(slot(Card::normal(Suit::Club, SuitValue::King)), 77);
    }
}
