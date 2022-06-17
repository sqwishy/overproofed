// #![no_std]
#![allow(unused)]

use js_sys::Array as JsArray;
use wasm_bindgen::prelude::*;

use core::marker::PhantomData;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &wasm_bindgen::JsValue);
}

macro_rules! butt {
    ($($arg:tt)*) => {
        $crate::log(&JsValue::from(format!($($arg)*)))
    };
}

/// ripped from std impl of dbg! to use butt! instead of eprintln!
#[macro_export]
macro_rules! dbg {
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                butt!("{} = {:#?}", stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[wasm_bindgen]
pub fn set_console_panic_hook() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// mod de {
//     use serde::{Deserialize, Serialize};
//     use wasm_bindgen::JsValue;

//     use super::{Amounts, Item, Mix, NullableAmounts, Recipe};
//     use ::overproofed::{Grams, Percent};

//     #[derive(Serialize, Deserialize)]
//     #[serde(remote = "Recipe")]
//     pub struct CompactRecipe(
//         #[serde(getter = "Recipe::items", with = "CompactItemVec")] Vec<Item>,
//         #[serde(getter = "Recipe::total", with = "CompactMix")] Mix,
//         #[serde(getter = "Recipe::mixes", with = "CompactMixVec")] Vec<Mix>,
//         #[serde(getter = "Recipe::finals")] Vec<Grams>,
//     );

//     #[derive(Serialize, Deserialize)]
//     #[serde(remote = "Item")]
//     pub struct CompactItem(
//         #[serde(getter = "Item::name")] String,
//         #[serde(getter = "Item::is_flour")] bool,
//     );

//     #[allow(non_snake_case)]
//     mod CompactItemVec {
//         use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

//         use super::{CompactItem, Item};

//         pub fn serialize<S>(vec: &Vec<Item>, er: S) -> Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//         {
//             let mut seq = er.serialize_seq(Some(vec.len()))?;
//             for element in vec {
//                 seq.serialize_element(&_Item(element))?;
//             }
//             return seq.end();

//             #[derive(Serialize)]
//             struct _Item<'i>(#[serde(with = "CompactItem")] &'i Item);
//         }

//         pub fn deserialize<'de, D>(er: D) -> Result<Vec<Item>, D::Error>
//         where
//             D: Deserializer<'de>,
//         {
//             return Ok(Vec::<_Item>::deserialize(er)?
//                 .into_iter()
//                 .map(|_Item(i)| i)
//                 .collect());

//             #[derive(Deserialize)]
//             struct _Item(#[serde(with = "CompactItem")] Item);
//         }
//     }

//     #[derive(Deserialize, Serialize)]
//     #[serde(remote = "Mix")]
//     pub struct CompactMix(
//         #[serde(getter = "Mix::name")] String,
//         #[serde(getter = "Mix::amounts", with = "CompactOptionAmountsVec")] Vec<JsValue>,
//         #[serde(getter = "Mix::flour", with = "CompactAmounts")] Amounts,
//         #[serde(getter = "Mix::total", with = "CompactAmounts")] Amounts,
//     );

//     #[allow(non_snake_case)]
//     mod CompactMixVec {
//         use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

//         use super::{CompactMix, Mix};

//         pub fn serialize<S>(vec: &Vec<Mix>, er: S) -> Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//         {
//             let mut seq = er.serialize_seq(Some(vec.len()))?;
//             for element in vec {
//                 seq.serialize_element(&_Mix(element))?;
//             }
//             return seq.end();

//             #[derive(serde::Serialize)]
//             struct _Mix<'i>(#[serde(with = "CompactMix")] &'i Mix);
//         }

//         pub fn deserialize<'de, D>(er: D) -> Result<Vec<Mix>, D::Error>
//         where
//             D: Deserializer<'de>,
//         {
//             return Ok(Vec::<_Mix>::deserialize(er)?
//                 .into_iter()
//                 .map(|_Mix(m)| m)
//                 .collect());

//             #[derive(Deserialize)]
//             struct _Mix(#[serde(with = "CompactMix")] Mix);
//         }
//     }

//     /* shitty code generation + shitty wasm_bindgen bindings = nightmare */
//     #[allow(non_snake_case)]
//     mod CompactOptionAmountsVec {
//         use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
//         use wasm_bindgen::JsValue;

//         use super::{Amounts, CompactAmounts, Mix};
//         use crate::NullableAmounts;

//         pub fn serialize<S>(
//             vec: &Vec<JsValue /* Option<Amounts> */>,
//             er: S,
//         ) -> Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//         {
//             let mut seq = er.serialize_seq(Some(vec.len()))?;
//             for element in vec {
//                 seq.serialize_element(
//                     &NullableAmounts::from(element.clone())
//                         .into_option()
//                         .as_ref()
//                         .map(_Amounts),
//                 )?;
//             }
//             return seq.end();

//             #[derive(Serialize)]
//             struct _Amounts<'i>(#[serde(with = "CompactAmounts")] &'i Amounts);
//         }

//         pub fn deserialize<'de, D>(er: D) -> Result<Vec<JsValue /* Option<Amounts> */>, D::Error>
//         where
//             D: Deserializer<'de>,
//         {
//             return Ok(Vec::<Option<_Amounts>>::deserialize(er)?
//                 .into_iter()
//                 .map(|option| option.map(|_Amounts(a)| a))
//                 .map(NullableAmounts::from)
//                 .map(NullableAmounts::into_jsvalue)
//                 .collect());

//             #[derive(Deserialize)]
//             struct _Amounts(#[serde(with = "CompactAmounts")] Amounts);
//         }
//     }

//     #[derive(Deserialize, Serialize)]
//     #[serde(remote = "Amounts")]
//     pub struct CompactAmounts(
//         #[serde(getter = "Amounts::bakers")] Percent,
//         #[serde(getter = "Amounts::weight")] Grams,
//         #[serde(getter = "Amounts::in_other")] Percent,
//     );

//     impl From<CompactRecipe> for Recipe {
//         fn from(c: CompactRecipe) -> Self {
//             let CompactRecipe(items, total, mixes, finals) = c;
//             let mut recipe = Self::default();
//             recipe.set_items(items);
//             recipe.set_total(total.into());
//             recipe.set_mixes(mixes);
//             recipe.set_finals(finals);
//             recipe
//         }
//     }

//     impl From<CompactItem> for Item {
//         fn from(c: CompactItem) -> Self {
//             let CompactItem(name, is_flour) = c;
//             let mut item = Self::default();
//             item.set_name(name);
//             item.set_is_flour(is_flour);
//             item
//         }
//     }

//     impl From<CompactMix> for Mix {
//         fn from(c: CompactMix) -> Self {
//             let CompactMix(name, amounts, flour, total) = c;
//             let mut mix = Self::default();
//             mix.set_name(name);
//             mix.set_amounts(amounts);
//             mix.set_flour(flour);
//             mix.set_total(total);
//             mix
//         }
//     }

//     impl From<CompactAmounts> for Amounts {
//         fn from(c: CompactAmounts) -> Self {
//             let CompactAmounts(bakers, weight, in_other) = c;
//             let mut amounts = Self::default();
//             amounts.set_bakers(bakers);
//             amounts.set_weight(weight);
//             amounts.set_in_other(in_other);
//             amounts
//         }
//     }
// }

#[wasm_bindgen]
pub fn base64_expand(s: &str) -> Option<String> {
    let smol = base64::decode_config(&s, base64::URL_SAFE_NO_PAD).ok()?;
    let big = lz4_flex::decompress_size_prepended(&smol).ok()?;
    String::from_utf8(big).ok()
}

#[wasm_bindgen]
pub fn compact_base64(s: &str) -> String {
    let smol = lz4_flex::block::compress_prepend_size(s.as_bytes());
    dbg!((
        s.as_bytes().len(),
        smol.len(),
        base64::encode_config(&smol, base64::URL_SAFE_NO_PAD).len(),
    ));
    base64::encode_config(&smol, base64::URL_SAFE_NO_PAD)
}

pub fn default<T: Default>() -> T {
    Default::default()
}

use ::overproofed as wrapped;
use ::overproofed::{Index, Value};

#[wasm_bindgen]
pub struct RecipeWriter {
    values: wrapped::Values,
    recipe: wrapped::Recipe,
    cursor: (MixCursor, ItemCursor),
}

#[wasm_bindgen]
pub struct NewItemFlags(u8);

#[wasm_bindgen]
impl NewItemFlags {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(default())
    }

    pub fn flour(&mut self) {
        use new_item_flags::*;

        self.0 = (self.0 & !IS_FLOUR_MASK) | FLOUR;
    }

    pub fn nonflour(&mut self) {
        use new_item_flags::*;

        self.0 = (self.0 & !IS_FLOUR_MASK) | NONFLOUR;
    }

    pub fn hole(&mut self) {
        use new_item_flags::*;

        self.0 = (self.0 & !ITEM_KIND_MASK) | HOLE;
    }

    pub fn total_item(&mut self) {
        use new_item_flags::*;

        self.0 = (self.0 & !ITEM_KIND_MASK) | TOTAL_ITEM;
    }

    pub fn mix_item(&mut self) {
        use new_item_flags::*;

        self.0 = (self.0 & !ITEM_KIND_MASK) | MIX_ITEM;
    }
}

pub mod new_item_flags {
    pub const IS_FLOUR_MASK: u8 /*_*/ = 0b1;
    pub const FLOUR: u8         /*_*/ = 0b0;
    pub const NONFLOUR: u8      /*_*/ = 0b1;
    pub const ITEM_KIND_MASK: u8 /*_*/ = 0b110;
    pub const HOLE: u8           /*_*/ = 0b000;
    pub const TOTAL_ITEM: u8     /*_*/ = 0b010;
    pub const MIX_ITEM: u8       /*_*/ = 0b100;
}

#[derive(Debug, Copy, Clone)]
enum MixCursor {
    Dough,
    Mix(u16),
}

#[derive(Debug, Copy, Clone)]
enum ItemCursor {
    Total,
    Flour,
    NonFlour,
    Flours(u16),
    NonFlours(u16),
}

impl RecipeWriter {
    /// returns the Item from the RecipeWriter's Recipe pointed to by the RecipeWriter's cursor
    fn cursor_item(&mut self) -> Option<&mut wrapped::Item> {
        let (mix, item) = self.cursor;

        let mix = match mix {
            MixCursor::Dough => &mut self.recipe.dough,
            MixCursor::Mix(i) => self.recipe.mixes.get_mut(i as usize)?,
        };

        match item {
            ItemCursor::Total => Some(&mut mix.total),
            ItemCursor::Flour => Some(&mut mix.flour),
            ItemCursor::NonFlour => Some(&mut mix.nonflour),
            ItemCursor::Flours(i) => mix.flours.get_mut(i as usize)?.as_mut(),
            ItemCursor::NonFlours(i) => mix.nonflours.get_mut(i as usize)?.as_mut(),
        }
    }

    fn map_cursor<F: FnOnce((MixCursor, ItemCursor)) -> (MixCursor, ItemCursor)>(&mut self, f: F) {
        self.cursor = f(self.cursor);
    }

    fn map_item_cursor<F: FnOnce(ItemCursor) -> ItemCursor>(&mut self, f: F) {
        let (mix, item) = self.cursor;
        self.cursor = (mix, f(item));
    }
}

#[wasm_bindgen]
impl RecipeWriter {
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> Self {
        let mut values = wrapped::Values::from(Vec::with_capacity(capacity));
        let recipe = values.minimal_recipe();
        let cursor = (MixCursor::Dough, ItemCursor::Total);
        Self { values, recipe, cursor }
    }

    pub fn set(&mut self, i: Index, v: Value) {
        *self.values.value_mut(i) = v;
    }

    pub fn solve(&mut self) -> Option<JsArray> {
        let Self { recipe, values, .. } = self;

        if values.did_overflow() {
            butt!("Values.did_overflow() {:?}", values.how_overflow());
            return None;
        }

        let results = JsArray::new();
        let mut solver = wrapped::solve::Solver::new(recipe, values);

        while let Some((index, value, _math)) = solver.step(values) {
            // let math = solver.math(_math).unwrap();
            // let line = math.line();
            // let math_display = math.display(&values);
            // butt!("{index:>3}\tL{line:>3}\t{math_display}");

            results.set(index as u32, value.into());
        }

        Some(results)
    }

    pub fn dough(&mut self) {
        self.map_cursor(|(_, _)| (MixCursor::Dough, ItemCursor::Total))
    }

    /// fails silently if there already `u16::MAX` mixes.
    pub fn new_mix(&mut self) {
        let Some(i) = u16::try_from(self.recipe.mixes.len()).ok() else {
            return;
        };

        self.recipe.mixes.push(self.values.minimal_mix());
        self.map_cursor(|_| (MixCursor::Mix(i), ItemCursor::Total))
    }

    /// fails silently if there are already `u16::MAX` items.
    pub fn new_item(&mut self, NewItemFlags(flags): NewItemFlags) {
        use new_item_flags::*;

        let (mix, _) = self.cursor;

        let mix = match mix {
            MixCursor::Dough => &mut self.recipe.dough,
            MixCursor::Mix(i) => match self.recipe.mixes.get_mut(i as usize) {
                None => return,
                Some(mix) => mix,
            },
        };

        let list = match flags & IS_FLOUR_MASK {
            NONFLOUR => &mut mix.nonflours,
            _ => &mut mix.flours,
        };

        let Some(i) = u16::try_from(list.len()).ok() else {
            return;
        };

        list.push(match flags & ITEM_KIND_MASK {
            TOTAL_ITEM => Some(self.values.new_item().into()),
            MIX_ITEM => Some(self.values.new_mix_item().into()),
            _ => None,
        });

        let newcursor = match flags & IS_FLOUR_MASK {
            NONFLOUR => ItemCursor::NonFlours(i),
            _ => ItemCursor::Flours(i),
        };
        self.map_item_cursor(|_| newcursor)
    }

    pub fn total(&mut self) {
        self.map_item_cursor(|_| ItemCursor::Total)
    }

    pub fn flour(&mut self) {
        self.map_item_cursor(|_| ItemCursor::Flour)
    }

    pub fn nonflour(&mut self) {
        self.map_item_cursor(|_| ItemCursor::NonFlour)
    }

    pub fn weight(&mut self) -> Index {
        self.cursor_item()
            .map(|item| item.weight)
            .unwrap_or(wrapped::OVERFLOW_INDEX)
    }

    pub fn bakers(&mut self) -> Index {
        self.cursor_item()
            .map(|item| item.bakers)
            .unwrap_or(wrapped::OVERFLOW_INDEX)
    }

    pub fn weight_in_mixes(&mut self) -> Index {
        self.cursor_item()
            .and_then(|item| item.with_mixes())
            .map(|item| item.weight_in_mixes)
            .unwrap_or(wrapped::OVERFLOW_INDEX)
    }

    pub fn weight_less_mixes(&mut self) -> Index {
        self.cursor_item()
            .and_then(|item| item.with_mixes())
            .map(|item| item.weight_less_mixes)
            .unwrap_or(wrapped::OVERFLOW_INDEX)
    }

    pub fn percent_in_mixes(&mut self) -> Index {
        self.cursor_item()
            .and_then(|item| item.with_mixes())
            .map(|item| item.percent_in_mixes)
            .unwrap_or(wrapped::OVERFLOW_INDEX)
    }

    pub fn percent_less_mixes(&mut self) -> Index {
        self.cursor_item()
            .and_then(|item| item.with_mixes())
            .map(|item| item.percent_less_mixes)
            .unwrap_or(wrapped::OVERFLOW_INDEX)
    }

    pub fn percent_of_total(&mut self) -> Index {
        self.cursor_item()
            .and_then(|item| item.in_mix())
            .map(|item| item.percent_of_total)
            .unwrap_or(wrapped::OVERFLOW_INDEX)
    }
}
