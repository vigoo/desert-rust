use crate::adt::{AdtMetadata, FieldPosition};
use crate::evolution::SerializedEvolutionStep;
use crate::state::State;
use crate::{
    BinaryOutput, BinarySerializer, Error, Evolution, Result, SerializationContext,
    DEFAULT_CAPACITY,
};
use hashbrown::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::IndexMut;

pub trait ChunkedOutput {
    type Output: BinaryOutput;

    fn output_for(&mut self, version: u8) -> &mut Self::Output;

    fn state_mut(&mut self) -> &mut State;

    fn record_field_index(&mut self, field_name: &str, chunk: u8);
    fn write_evolution_header(
        &mut self,
        evolution_steps: &[Evolution],
        removed_fields: &HashSet<String>,
    ) -> crate::Result<()>;
    fn write_ordered_chunks(&mut self) -> crate::Result<()>;
}

pub struct NonChunkedOutput<'a, Context: SerializationContext> {
    context: &'a mut Context,
}

impl<'a, Context: SerializationContext> NonChunkedOutput<'a, Context> {
    pub fn new(context: &'a mut Context) -> Self {
        Self { context }
    }
}

impl<'a, Context: SerializationContext> ChunkedOutput for NonChunkedOutput<'a, Context> {
    type Output = Context::Output;

    fn output_for(&mut self, _version: u8) -> &mut Self::Output {
        self.context.output_mut()
    }

    fn state_mut(&mut self) -> &mut State {
        self.context.state_mut()
    }

    fn record_field_index(&mut self, _field_name: &str, _chunk: u8) {}

    fn write_evolution_header(
        &mut self,
        _evolution_steps: &[Evolution],
        _removed_fields: &HashSet<String>,
    ) -> crate::Result<()> {
        Ok(())
    }

    fn write_ordered_chunks(&mut self) -> crate::Result<()> {
        Ok(())
    }
}

pub struct BufferingChunkedOutput<'a, Context: SerializationContext> {
    context: &'a mut Context,
    buffers: Vec<Vec<u8>>,
    last_index_per_chunk: HashMap<u8, u8>,
    field_indices: HashMap<String, FieldPosition>,
}

impl<'a, Context: SerializationContext> BufferingChunkedOutput<'a, Context> {
    pub fn new(context: &'a mut Context, version: u8) -> Self {
        Self {
            context,
            buffers: (0..version)
                .map(|_| Vec::with_capacity(DEFAULT_CAPACITY))
                .collect(),
            last_index_per_chunk: HashMap::new(),
            field_indices: HashMap::new(),
        }
    }
}

impl<'a, Context: SerializationContext> ChunkedOutput for BufferingChunkedOutput<'a, Context> {
    type Output = Vec<u8>;

    fn output_for(&mut self, version: u8) -> &mut Self::Output {
        self.buffers.index_mut(version as usize)
    }

    fn state_mut(&mut self) -> &mut State {
        self.context.state_mut()
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
    ) -> crate::Result<()> {
        for (v, evolution) in evolution_steps.iter().enumerate() {
            let step = match evolution {
                Evolution::InitialVersion => {
                    let size = self.buffers[v].len().try_into()?;
                    Ok(SerializedEvolutionStep::FieldAddedToNewChunk { size })
                }
                Evolution::FieldAdded { .. } => {
                    let size = self.buffers[v].len().try_into()?;
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

    fn write_ordered_chunks(&mut self) -> crate::Result<()> {
        for buffer in &self.buffers {
            self.context.output_mut().write_bytes(buffer);
        }
        Ok(())
    }
}

pub struct ChunkedSerialization<'a, CO: ChunkedOutput> {
    output: &'a mut CO,
    chunk: u8,
}

impl<'a, CO: ChunkedOutput> ChunkedSerialization<'a, CO> {
    pub fn new(output: &'a mut CO, chunk: u8) -> Self {
        Self { output, chunk }
    }
}

impl<'a, CO: ChunkedOutput> SerializationContext for ChunkedSerialization<'a, CO> {
    type Output = CO::Output;

    fn output_mut(&mut self) -> &mut Self::Output {
        self.output.output_for(self.chunk)
    }

    fn state_mut(&mut self) -> &mut State {
        self.output.state_mut()
    }
}

pub struct AdtSerializer<'a, Context: SerializationContext, CO: ChunkedOutput> {
    metadata: &'a AdtMetadata,
    chunked_output: CO,
    context: PhantomData<Context>,
}

impl<'a, 'b, Context: SerializationContext>
    AdtSerializer<'a, Context, NonChunkedOutput<'b, Context>>
{
    pub fn new_v0(metadata: &'a AdtMetadata, context: &'b mut Context) -> Self {
        assert_eq!(metadata.version, 0);
        context.output_mut().write_u8(metadata.version);
        let chunked_output = NonChunkedOutput::new(context);
        Self {
            metadata,
            chunked_output,
            context: PhantomData,
        }
    }
}

impl<'a, 'b, Context: SerializationContext>
    AdtSerializer<'a, Context, BufferingChunkedOutput<'b, Context>>
{
    pub fn new(metadata: &'a AdtMetadata, context: &'b mut Context) -> Self {
        context.output_mut().write_u8(metadata.version);
        let chunked_output = BufferingChunkedOutput::new(context, metadata.version);
        Self {
            metadata,
            chunked_output,
            context: PhantomData,
        }
    }
}

impl<'a, Context: SerializationContext, CO: ChunkedOutput> AdtSerializer<'a, Context, CO> {
    pub fn write_field<T: BinarySerializer>(
        &mut self,
        field_name: &str,
        value: &T,
    ) -> crate::Result<()> {
        let chunk = *self
            .metadata
            .field_generations
            .get(field_name)
            .unwrap_or(&0);
        let mut context = ChunkedSerialization::new(&mut self.chunked_output, chunk);
        value.serialize(&mut context)?;
        self.chunked_output.record_field_index(field_name, chunk);
        Ok(())
    }

    pub fn finish(mut self) -> crate::Result<()> {
        self.chunked_output.write_evolution_header(
            &self.metadata.evolution_steps,
            &self.metadata.removed_fields,
        )?;
        self.chunked_output.write_ordered_chunks()
    }

    pub fn write_constructor(
        &mut self,
        constructor_idx: u32,
        serialize_case: impl FnOnce(&mut ChunkedSerialization<CO>) -> Result<()>,
    ) -> crate::Result<()> {
        let mut context = ChunkedSerialization::new(&mut self.chunked_output, 0);
        context.output_mut().write_var_u32(constructor_idx);
        serialize_case(&mut context)
    }
}
