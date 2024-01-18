use crate::deserializer::DeserializationContext;
use crate::error::Result;
use crate::evolution::SerializedEvolutionStep;
use crate::serializer::SerializationContext;
use crate::state::State;
use crate::{BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer, Error, Evolution};
use bytes::{Bytes, BytesMut};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::IndexMut;
use std::sync::Arc;

pub struct AdtMetadata {
    version: u8,
    field_generations: HashMap<String, u8>,
    field_defaults: HashMap<String, Arc<dyn Any + Send + Sync>>,
    made_optional_at: HashMap<String, u8>,
    removed_fields: HashSet<String>,
    constructor_name_to_id: HashMap<String, u32>,
    constructor_id_to_name: HashMap<u32, String>,
    transient_fields: HashMap<String, Arc<dyn Any + Send + Sync>>,
    type_name: String,
    evolution_steps: Vec<Evolution>,
}

impl AdtMetadata {
    pub fn new(
        type_name: &str,
        evolution_steps: Vec<Evolution>,
        constructors: &[&str],
        transient_fields: HashMap<String, Arc<dyn Any + Send + Sync>>,
    ) -> Self {
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

        let field_defaults = evolution_steps
            .iter()
            .filter_map(|evolution| {
                if let Evolution::FieldAdded { name, default } = evolution {
                    Some((name.clone(), default.clone()))
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

        let constructor_name_to_id: HashMap<String, u32> = constructors
            .iter()
            .enumerate()
            .map(|(idx, name)| (name.to_string(), idx as u32))
            .collect();

        let constructor_id_to_name = constructor_name_to_id
            .iter()
            .map(|(name, id)| (*id, name.clone()))
            .collect();

        Self {
            type_name: type_name.to_string(),
            version: (evolution_steps.len() - 1) as u8,
            field_generations,
            field_defaults,
            made_optional_at,
            removed_fields,
            constructor_name_to_id,
            constructor_id_to_name,
            transient_fields,
            evolution_steps,
        }
    }
}

#[derive(Debug, Copy, Clone)]
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let byte = context.input_mut().read_i8()?;
        if byte < 0 {
            Ok(FieldPosition::new(0, (-byte) as u8))
        } else {
            Ok(FieldPosition::new(byte as u8, 0))
        }
    }
}

pub trait ChunkedOutput {
    type Output: BinaryOutput;

    fn output_for(&mut self, version: u8) -> &mut Self::Output;

    fn state_mut(&mut self) -> &mut State;

    fn write_evolution_header(
        &mut self,
        evolution_steps: &[Evolution],
        field_indices: &HashMap<String, FieldPosition>,
        removed_fields: &HashSet<String>,
    ) -> Result<()>;
    fn write_ordered_chunks(&mut self) -> Result<()>;
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

    fn write_evolution_header(
        &mut self,
        _evolution_steps: &[Evolution],
        _field_indices: &HashMap<String, FieldPosition>,
        _removed_fields: &HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }

    fn write_ordered_chunks(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct BufferingChunkedOutput<'a, Context: SerializationContext> {
    context: &'a mut Context,
    buffers: Vec<BytesMut>,
}

impl<'a, Context: SerializationContext> BufferingChunkedOutput<'a, Context> {
    pub fn new(context: &'a mut Context, version: u8) -> Self {
        Self {
            context,
            buffers: (0..version).map(|_| BytesMut::new()).collect(),
        }
    }
}

impl<'a, Context: SerializationContext> ChunkedOutput for BufferingChunkedOutput<'a, Context> {
    type Output = BytesMut;

    fn output_for(&mut self, version: u8) -> &mut Self::Output {
        self.buffers.index_mut(version as usize)
    }

    fn state_mut(&mut self) -> &mut State {
        self.context.state_mut()
    }

    fn write_evolution_header(
        &mut self,
        evolution_steps: &[Evolution],
        field_indices: &HashMap<String, FieldPosition>,
        removed_fields: &HashSet<String>,
    ) -> Result<()> {
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
                Evolution::FieldMadeOptional { name } => match field_indices.get(name) {
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
        let final_buffers: Vec<Bytes> = self
            .buffers
            .iter_mut()
            .map(|b| {
                let mut bytes = BytesMut::new();
                std::mem::swap(b, &mut bytes);
                bytes.freeze()
            })
            .collect();
        for buffer in final_buffers {
            println!("writing chunk of size {}", buffer.len());
            self.context.output_mut().write_bytes(&buffer);
        }
        Ok(())
    }
}

pub struct ChunkedSerialization<'a, CO: ChunkedOutput> {
    output: &'a mut CO,
    chunk: u8
}

impl<'a, CO: ChunkedOutput> ChunkedSerialization<'a, CO> {
    pub fn new(output: &'a mut CO, chunk: u8) -> Self {
        Self {
            output,
            chunk
        }
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
    last_index_per_chunk: HashMap<u8, u8>,
    field_indices: HashMap<String, FieldPosition>,
}

impl<'a, 'b, Context: SerializationContext> AdtSerializer<'a, Context, NonChunkedOutput<'b, Context>> {
    pub fn new_v0(metadata: &'a AdtMetadata, context: &'b mut Context) -> Self {
        assert_eq!(metadata.version, 0); // TODO: try to do a type level constraint?
        context.output_mut().write_u8(metadata.version);
        let chunked_output = NonChunkedOutput::new(context);
        Self {
            metadata,
            chunked_output,
            context: PhantomData,
            last_index_per_chunk: HashMap::new(),
            field_indices: HashMap::new(),
        }
    }
}

impl<'a, 'b, Context: SerializationContext> AdtSerializer<'a, Context, BufferingChunkedOutput<'b, Context>> {
    pub fn new(metadata: &'a AdtMetadata, context: &'b mut Context) -> Self {
        context.output_mut().write_u8(metadata.version);
        let chunked_output = BufferingChunkedOutput::new(context, metadata.version);
        Self {
            metadata,
            chunked_output,
            context: PhantomData,
            last_index_per_chunk: HashMap::new(),
            field_indices: HashMap::new(),
        }
    }
}

impl<'a, 'b, Context: SerializationContext, CO: ChunkedOutput> AdtSerializer<'a, Context, CO> {
    pub fn write_field<T: BinarySerializer>(&mut self, field_name: &str, value: &T) -> Result<()> {
        let chunk = *self.metadata.field_generations.get(field_name).unwrap_or(&0);
        let mut context = ChunkedSerialization::new(&mut self.chunked_output, chunk);
        value.serialize(&mut context)?;
        self.record_field_index(field_name, chunk);
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.chunked_output.write_evolution_header(
            &self.metadata.evolution_steps, &self.field_indices, &self.metadata.removed_fields,
        )?;
        self.chunked_output.write_ordered_chunks()
    }

    pub fn write_constructor<T: BinarySerializer>(&mut self, constructor_name: &str, value: &T) -> Result<()> {
        let constructor_id = self.get_constructor_id(constructor_name)?;
        let mut context = ChunkedSerialization::new(&mut self.chunked_output, 0);
        context.output_mut().write_var_u32(constructor_id);
        value.serialize(&mut context)?;
        Ok(())
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
                self.field_indices.insert(field_name.to_string(), FieldPosition::new(chunk, 0));
            }
        }
    }

    fn get_constructor_id(&mut self, constructor_name: &str) -> Result<u32> {
        match self.metadata.constructor_name_to_id.get(constructor_name) {
            Some(id) => Ok(*id),
            None => Err(Error::InvalidConstructorName { constructor_name: constructor_name.to_string(), type_name: self.metadata.type_name.clone() }),
        }
    }
}

pub trait ChunkedInput {
    type Input: BinaryInput;

    fn stored_version(&self) -> u8;

    fn input_for(&mut self, version: u8) -> &mut Self::Input;
}