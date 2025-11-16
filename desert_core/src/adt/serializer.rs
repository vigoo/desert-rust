use hashbrown::{HashMap, HashSet};
use std::array;

use crate::adt::{AdtMetadata, FieldPosition};
use crate::evolution::SerializedEvolutionStep;
use crate::{
    BinaryOutput, BinarySerializer, Error, Evolution, Result, SerializationContext,
    DEFAULT_CAPACITY,
};

pub struct AdtSerializer<'a, 'b, Output: BinaryOutput, const V: usize> {
    metadata: &'a AdtMetadata,
    context: &'b mut SerializationContext<Output>,
    buffers: Option<[Option<Vec<u8>>; V]>,
    last_index_per_chunk: Option<[i8; V]>,
    field_indices: Option<HashMap<String, FieldPosition>>,
}

impl<'a, 'b, Output: BinaryOutput, const V: usize> AdtSerializer<'a, 'b, Output, V> {
    pub fn new_v0(
        metadata: &'a AdtMetadata,
        context: &'b mut SerializationContext<Output>,
    ) -> Self {
        assert_eq!(metadata.version, 0);
        context.write_u8(metadata.version);
        Self {
            metadata,
            context,
            buffers: None,
            last_index_per_chunk: None,
            field_indices: None,
        }
    }

    pub fn new(metadata: &'a AdtMetadata, context: &'b mut SerializationContext<Output>) -> Self {
        context.write_u8(metadata.version);
        let contains_field_made_optional = !metadata.made_optional_at.is_empty();
        Self {
            metadata,
            context,
            buffers: Some(array::from_fn(|_| {
                Some(Vec::with_capacity(DEFAULT_CAPACITY))
            })),
            last_index_per_chunk: if contains_field_made_optional {
                Some([-1; V])
            } else {
                None
            },
            field_indices: if contains_field_made_optional {
                Some(HashMap::new())
            } else {
                None
            },
        }
    }

    pub fn write_field<T: BinarySerializer>(&mut self, field_name: &str, value: &T) -> Result<()> {
        let chunk = *self
            .metadata
            .field_generations
            .get(field_name)
            .unwrap_or(&0);
        let mut requires_buffer = false;
        if let Some(buffers) = self.buffers.as_mut() {
            self.context
                .push_buffer(buffers[chunk as usize].take().unwrap());
            requires_buffer = true;
        }
        value.serialize(self.context)?;
        if requires_buffer {
            self.buffers.as_mut().unwrap()[chunk as usize] = Some(self.context.pop_buffer());
            self.record_field_index(field_name, chunk);
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        if let Some(buffers) = self.buffers.take() {
            self.write_evolution_header(
                &buffers,
                &self.metadata.evolution_steps,
                &self.metadata.removed_fields,
            )?;
            self.write_ordered_chunks(buffers)
        } else {
            Ok(())
        }
    }

    pub fn write_constructor(
        &mut self,
        constructor_idx: u32,
        serialize_case: impl FnOnce(&mut SerializationContext<Output>) -> Result<()>,
    ) -> Result<()> {
        self.context.write_var_u32(constructor_idx);
        serialize_case(self.context)
    }

    fn record_field_index(&mut self, field_name: &str, chunk: u8) {
        if let Some(last_index_per_chunk) = self.last_index_per_chunk.as_mut() {
            match &mut last_index_per_chunk[chunk as usize] {
                last_index => {
                    let new_index = *last_index + 1;
                    *last_index = new_index;
                    self.field_indices.as_mut().unwrap().insert(
                        field_name.to_string(),
                        FieldPosition::new(chunk, new_index as u8),
                    );
                }
            }
        }
    }

    fn write_evolution_header(
        &mut self,
        buffers: &[Option<Vec<u8>>],
        evolution_steps: &[Evolution],
        removed_fields: &HashSet<String>,
    ) -> Result<()> {
        for (v, evolution) in evolution_steps.iter().enumerate() {
            let step = match evolution {
                Evolution::InitialVersion => {
                    let size = buffers[v].as_ref().unwrap().len().try_into()?;
                    Ok(SerializedEvolutionStep::FieldAddedToNewChunk { size })
                }
                Evolution::FieldAdded { .. } => {
                    let size = buffers[v].as_ref().unwrap().len().try_into()?;
                    Ok(SerializedEvolutionStep::FieldAddedToNewChunk { size })
                }
                Evolution::FieldMadeOptional { name } => {
                    match self.field_indices.as_ref().unwrap().get(name) {
                        Some(field_position) => Ok(SerializedEvolutionStep::FieldMadeOptional {
                            position: *field_position,
                        }),
                        None => {
                            if removed_fields.contains(name) {
                                Ok(SerializedEvolutionStep::FieldRemoved {
                                    field_name: name.clone(),
                                })
                            } else {
                                Err(Error::UnknownFieldReferenceInEvolutionStep(name.clone()))
                            }
                        }
                    }
                }
                Evolution::FieldRemoved { name } => Ok(SerializedEvolutionStep::FieldRemoved {
                    field_name: name.clone(),
                }),
                Evolution::FieldMadeTransient { name } => {
                    Ok(SerializedEvolutionStep::FieldRemoved {
                        field_name: name.clone(),
                    })
                }
            }?;
            step.serialize(self.context)?;
        }
        Ok(())
    }

    fn write_ordered_chunks(&mut self, buffers: [Option<Vec<u8>>; V]) -> Result<()> {
        for buffer in buffers {
            self.context.write_bytes(&buffer.unwrap());
        }
        Ok(())
    }
}
