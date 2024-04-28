use hashbrown::{HashMap, HashSet};

use crate::adt::{AdtMetadata, FieldPosition};
use crate::deserializer::InputRegion;
use crate::evolution::SerializedEvolutionStep;
use crate::{BinaryDeserializer, BinaryInput, DeserializationContext, Error, Result};

pub struct AdtDeserializer<'a, 'b, 'c> {
    metadata: &'a AdtMetadata,
    context: &'b mut DeserializationContext<'c>,
    last_index_per_chunk: Vec<i8>,
    read_constructor_idx: Option<u32>,

    stored_version: u8,
    made_optional_at: HashMap<FieldPosition, u8>,
    removed_fields: HashSet<String>,
    inputs: Vec<InputRegion>,
}

impl<'a, 'b, 'c> AdtDeserializer<'a, 'b, 'c> {
    pub fn new_v0(
        metadata: &'a AdtMetadata,
        context: &'b mut DeserializationContext<'c>,
    ) -> Result<Self> {
        Ok(Self {
            metadata,
            context,
            last_index_per_chunk: vec![-1i8; metadata.version as usize + 1],
            read_constructor_idx: None,
            stored_version: 0,
            made_optional_at: HashMap::new(),
            removed_fields: HashSet::new(),
            inputs: Vec::new(),
        })
    }

    pub fn new(
        metadata: &'a AdtMetadata,
        context: &'b mut DeserializationContext<'c>,
        stored_version: u8,
    ) -> Result<Self> {
        let mut serialized_evolution_steps = Vec::with_capacity(stored_version as usize + 1);
        for _ in 0..=stored_version {
            let serialized_evolution_step = SerializedEvolutionStep::deserialize(context)?;
            serialized_evolution_steps.push(serialized_evolution_step);
        }

        let mut inputs = Vec::with_capacity(serialized_evolution_steps.len());
        let mut made_optional_at = HashMap::new();
        let mut removed_fields = HashSet::new();

        for (idx, serialized_evolution_step) in serialized_evolution_steps.iter().enumerate() {
            match serialized_evolution_step {
                SerializedEvolutionStep::FieldAddedToNewChunk { size } => {
                    let start = context.pos();
                    let _ = context.read_bytes(*size as usize)?;
                    inputs.push(InputRegion::new(start, *size as usize));
                }
                SerializedEvolutionStep::FieldMadeOptional { position } => {
                    made_optional_at.insert(*position, idx as u8);
                    inputs.push(InputRegion::empty());
                }
                SerializedEvolutionStep::FieldRemoved { field_name } => {
                    removed_fields.insert(field_name.clone());
                    inputs.push(InputRegion::empty());
                }
                _ => {
                    inputs.push(InputRegion::empty());
                }
            }
        }

        Ok(Self {
            metadata,
            context,
            last_index_per_chunk: vec![-1i8; metadata.version as usize + 1],
            read_constructor_idx: None,
            stored_version,
            made_optional_at,
            removed_fields,
            inputs,
        })
    }

    pub fn read_field<T: BinaryDeserializer>(
        &mut self,
        field_name: &str,
        field_default: Option<T>,
    ) -> Result<T> {
        if self.removed_fields.contains(field_name) {
            Err(Error::FieldRemovedInSerializedVersion(
                field_name.to_string(),
            ))
        } else {
            let chunk = *self
                .metadata
                .field_generations
                .get(field_name)
                .unwrap_or(&0);
            let field_position = self.record_field_index(chunk);
            if self.stored_version < chunk {
                // Field was not serialized
                match field_default {
                    Some(value) => Ok(value),
                    None => Err(Error::FieldWithoutDefaultValueIsMissing(
                        field_name.to_string(),
                    )),
                }
            } else {
                // Field was serialized

                let has_inputs = !self.inputs.is_empty();
                if has_inputs {
                    self.context.push_region(self.inputs[chunk as usize]);
                }
                let result = if self.made_optional_at.contains_key(&field_position) {
                    // The field was made optional in a newer version, so we have to read Option<T>

                    let is_defined = bool::deserialize(self.context)?;
                    if is_defined {
                        T::deserialize(self.context)
                    } else {
                        Err(Error::NonOptionalFieldSerializedAsNone(
                            field_name.to_string(),
                        ))
                    }
                } else {
                    T::deserialize(self.context)
                };
                if has_inputs {
                    self.inputs[chunk as usize] = self.context.pop_region();
                }
                result
            }
        }
    }

    pub fn read_optional_field<T: BinaryDeserializer>(
        &mut self,
        field_name: &str,
        field_default: Option<Option<T>>,
    ) -> Result<Option<T>> {
        if self.removed_fields.contains(field_name) {
            Ok(None)
        } else {
            let chunk = *self
                .metadata
                .field_generations
                .get(field_name)
                .unwrap_or(&0);
            let opt_since = *self.metadata.made_optional_at.get(field_name).unwrap_or(&0);

            self.record_field_index(chunk);
            if self.stored_version < chunk {
                // This field was not serialized
                match field_default {
                    Some(default_value) => Ok(default_value),
                    None => Err(Error::DeserializationFailure(format!(
                        "Field {field_name} is not in the stream and does not have a default value"
                    ))),
                }
            } else {
                // This field was serialized

                let has_inputs = !self.inputs.is_empty();
                if has_inputs {
                    self.context.push_region(self.inputs[chunk as usize]);
                }
                let result = if self.stored_version < opt_since {
                    Ok(Some(T::deserialize(self.context)?))
                } else {
                    Option::<T>::deserialize(self.context)
                };
                if has_inputs {
                    self.inputs[chunk as usize] = self.context.pop_region();
                }
                result
            }
        }
    }

    pub fn read_constructor<T>(
        &mut self,
        case_idx: u32,
        deserialize_case: impl FnOnce(&mut DeserializationContext<'c>) -> Result<T>,
    ) -> Result<Option<T>> {
        let constructor_idx = self.read_or_get_constructor_idx()?;
        if constructor_idx == case_idx {
            let has_inputs = !self.inputs.is_empty();
            if has_inputs {
                self.context.push_region(self.inputs[0]);
            }
            let result = Ok(Some(deserialize_case(self.context)?));
            if has_inputs {
                self.inputs[0] = self.context.pop_region();
            }
            result
        } else {
            Ok(None)
        }
    }

    fn record_field_index(&mut self, chunk: u8) -> FieldPosition {
        let last_index = &mut self.last_index_per_chunk[chunk as usize];
        let new_index = *last_index + 1;
        let fp = FieldPosition::new(chunk, new_index as u8);
        *last_index = new_index;
        fp
    }

    fn read_or_get_constructor_idx(&mut self) -> Result<u32> {
        match self.read_constructor_idx {
            Some(idx) => Ok(idx),
            None => {
                let has_inputs = !self.inputs.is_empty();
                if has_inputs {
                    self.context.push_region(self.inputs[0]);
                }
                let constructor_idx = self.context.read_var_u32()?;
                if has_inputs {
                    self.inputs[0] = self.context.pop_region();
                }
                self.read_constructor_idx = Some(constructor_idx);
                Ok(constructor_idx)
            }
        }
    }
}
