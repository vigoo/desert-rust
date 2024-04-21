use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;

use crate::deserializer::DeserializationContext;
use crate::error::Result;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer, Evolution};

mod deserializer;
mod serializer;

pub use deserializer::AdtDeserializer;
pub use serializer::AdtSerializer;

lazy_static! {
    pub static ref EMPTY_ADT_METADATA: AdtMetadata =
        AdtMetadata::new(vec![Evolution::InitialVersion]);
}

#[derive(Debug)]
pub struct AdtMetadata {
    version: u8,
    field_generations: HashMap<String, u8>,
    made_optional_at: HashMap<String, u8>,
    removed_fields: HashSet<String>,
    evolution_steps: Vec<Evolution>,
}

impl AdtMetadata {
    pub fn new(evolution_steps: Vec<Evolution>) -> Self {
        if evolution_steps.len() > 255 {
            panic!("Too many evolution steps");
        }

        let field_generations = evolution_steps
            .iter()
            .enumerate()
            .filter_map(|(idx, evolution)| {
                if let Evolution::FieldAdded { name, .. } = evolution {
                    Some((name.clone(), idx as u8))
                } else {
                    None
                }
            })
            .collect();

        let made_optional_at = evolution_steps
            .iter()
            .enumerate()
            .filter_map(|(idx, evolution)| {
                if let Evolution::FieldMadeOptional { name } = evolution {
                    Some((name.clone(), idx as u8))
                } else {
                    None
                }
            })
            .collect();

        let removed_fields = evolution_steps
            .iter()
            .filter_map(|evolution| {
                if let Evolution::FieldRemoved { name } = evolution {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        Self {
            version: (evolution_steps.len() - 1) as u8,
            field_generations,
            made_optional_at,
            removed_fields,
            evolution_steps,
        }
    }
}

pub trait DefaultValue<T> {
    fn default_value(&self) -> T;
}

struct ProvidedDefaultValue<T: Clone> {
    value: T,
}

impl<T: Clone> DefaultValue<T> for ProvidedDefaultValue<T> {
    fn default_value(&self) -> T {
        self.value.clone()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldPosition {
    pub chunk: u8,
    pub position: u8,
}

impl FieldPosition {
    pub fn new(chunk: u8, position: u8) -> Self {
        Self { chunk, position }
    }

    pub fn to_byte(&self) -> u8 {
        if self.chunk == 0 {
            (-(self.position as i8)) as u8
        } else {
            self.chunk
        }
    }
}

impl BinarySerializer for FieldPosition {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(self.to_byte());
        Ok(())
    }
}

impl BinaryDeserializer for FieldPosition {
    fn deserialize<Input: BinaryInput>(
        context: &mut DeserializationContext<Input>,
    ) -> Result<Self> {
        let byte = context.read_i8()?;
        if byte < 0 {
            Ok(FieldPosition::new(0, (-byte) as u8))
        } else {
            Ok(FieldPosition::new(byte as u8, 0))
        }
    }
}
