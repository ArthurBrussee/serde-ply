// use serde::de::{DeserializeSeed, MapAccess};

// use crate::{PlyError, PlyHeader};

// pub(crate) struct PlyFileDeserializer<R> {
//     pub reader: R,
//     header: Option<PlyHeader>,
// }

// impl<'de, R: Read> Deserializer<'de> for PlyFileDeserializer<R> {
//     type Error = PlyError;

//     fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: Visitor<'de>,
//     {
//         Err(PlyError::Serde(
//             "deserialize_any not supported - struct fields must have specific types".to_string(),
//         ))
//     }

//     fn deserialize_struct<V>(
//         mut self,
//         _name: &'static str,
//         _fields: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: Visitor<'de>,
//     {
//         if let Some(header) = self.header {
//         } else {
//         }

//         // TODO: Now here do the header parsing if needed first.
//         visitor.visit_map(AsciiRowMapAccess {
//             parent: &mut self,
//             current_property: 0,
//         })
//     }

//     fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_map(AsciiRowMapAccess {
//             parent: &mut self,
//             current_property: 0,
//         })
//     }

//     serde::forward_to_deserialize_any! {
//         bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
//         bytes byte_buf option unit unit_struct newtype_struct seq tuple
//         tuple_struct enum identifier ignored_any
//     }
// }

// // A map of the whole ply file.
// // { header: PlyHeader, elem1: Vec<Elem1>, ... }
// struct PlyFileMapAccess {}

// impl<'de> MapAccess<'de> for PlyFileMapAccess {
//     type Error = PlyError;

//     fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
//     where
//         K: DeserializeSeed<'de>,
//     {
//         // TODO: First deserialize all the fields of the header.
//         todo!()
//     }

//     fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
//     where
//         V: DeserializeSeed<'de>,
//     {
//         // TODO: First de
//         todo!()
//     }
// }

// struct PlyHeaderMapAccess;

// // Header
// // pub format: PlyFormat,
// // pub version: String,
// // pub elements: Vec<ElementDef>,
// // pub comments: Vec<String>,
// // pub obj_info: Vec<String>,
// impl<'de> MapAccess<'de> for PlyHeaderMapAccess {
//     type Error;

//     fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
//     where
//         K: DeserializeSeed<'de>,
//     {
//         todo!()
//     }

//     fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
//     where
//         V: DeserializeSeed<'de>,
//     {
//         todo!()
//     }
// }
