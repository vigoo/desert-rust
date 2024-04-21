use hashbrown::{HashMap, HashSet};

use crate::adt::{AdtMetadata, FieldPosition};
use crate::evolution::SerializedEvolutionStep;
use crate::{
    BinaryOutput, BinarySerializer, Error, Evolution, Result, SerializationContext,
    DEFAULT_CAPACITY,
};

pub struct AdtSerializer<'a, 'b, Output: BinaryOutput> {
    metadata: &'a AdtMetadata,
    context: &'b mut SerializationContext<Output>,
    buffers: Vec<Option<Vec<u8>>>,
    last_index_per_chunk: HashMap<u8, u8>,
    field_indices: HashMap<String, FieldPosition>,
}

impl<'a, 'b, Output: BinaryOutput> AdtSerializer<'a, 'b, Output> {
    pub fn new_v0(
        metadata: &'a AdtMetadata,
        context: &'b mut SerializationContext<Output>,
    ) -> Self {
        assert_eq!(metadata.version, 0);
        context.write_u8(metadata.version);
        Self {
            metadata,
            context,
            buffers: Vec::new(),
            last_index_per_chunk: HashMap::new(),
            field_indices: HashMap::new(),
        }
    }

    pub fn new(metadata: &'a AdtMetadata, context: &'b mut SerializationContext<Output>) -> Self {
        context.write_u8(metadata.version);
        Self {
            metadata,
            context,
            buffers: (0..=metadata.version)
                .map(|_| Some(Vec::with_capacity(DEFAULT_CAPACITY)))
                .collect(),
            last_index_per_chunk: HashMap::new(),
            field_indices: HashMap::new(),
        }
    }

    pub fn write_field<T: BinarySerializer>(&mut self, field_name: &str, value: &T) -> Result<()> {
        let chunk = *self
            .metadata
            .field_generations
            .get(field_name)
            .unwrap_or(&0);
        let requires_buffer = !self.buffers.is_empty();
        if requires_buffer {
            self.context
                .push_buffer(self.buffers[chunk as usize].take().unwrap());
        }
        value.serialize(self.context)?;
        if requires_buffer {
            self.buffers[chunk as usize] = Some(self.context.pop_buffer());
            self.record_field_index(field_name, chunk);
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        if !self.buffers.is_empty() {
            self.write_evolution_header(
                &self.metadata.evolution_steps,
                &self.metadata.removed_fields,
            )?;
            self.write_ordered_chunks()
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
        match self.last_index_per_chunk.get_mut(&chunk) {
            Some(last_index) => {
                let new_index = *last_index + 1;
                *last_index = new_index;
                self.field_indices
                    .insert(field_name.to_string(), FieldPosition::new(chunk, new_index));
            }
            None => {
                self.last_index_per_chunk.insert(chunk, 0);
                self.field_indices
                    .insert(field_name.to_string(), FieldPosition::new(chunk, 0));
            }
        }
    }

    fn write_evolution_header(
        &mut self,
        evolution_steps: &[Evolution],
        removed_fields: &HashSet<String>,
    ) -> Result<()> {
        for (v, evolution) in evolution_steps.iter().enumerate() {
            let step = match evolution {
                Evolution::InitialVersion => {
                    let size = self.buffers[v].as_ref().unwrap().len().try_into()?;
                    Ok(SerializedEvolutionStep::FieldAddedToNewChunk { size })
                }
                Evolution::FieldAdded { .. } => {
                    let size = self.buffers[v].as_ref().unwrap().len().try_into()?;
                    Ok(SerializedEvolutionStep::FieldAddedToNewChunk { size })
                }
                Evolution::FieldMadeOptional { name } => match self.field_indices.get(name) {
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
                },
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

    fn write_ordered_chunks(&mut self) -> Result<()> {
        for buffer in &self.buffers {
            self.context.write_bytes(buffer.as_ref().unwrap());
        }
        Ok(())
    }
}
