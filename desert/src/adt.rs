use std::marker::PhantomData;
use std::ops::IndexMut;

use bytes::Bytes;
use hashbrown::{HashMap, HashSet};

use crate::deserializer::DeserializationContext;
use crate::error::Result;
use crate::evolution::SerializedEvolutionStep;
use crate::serializer::SerializationContext;
use crate::state::State;
use crate::{
    BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer, Error, Evolution,
    DEFAULT_CAPACITY,
};

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

    fn record_field_index(&mut self, field_name: &str, chunk: u8);
    fn write_evolution_header(
        &mut self,
        evolution_steps: &[Evolution],
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

    fn record_field_index(&mut self, _field_name: &str, _chunk: u8) {}

    fn write_evolution_header(
        &mut self,
        _evolution_steps: &[Evolution],
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
        assert_eq!(metadata.version, 0); // TODO: try to do a type level constraint?
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
    pub fn write_field<T: BinarySerializer>(&mut self, field_name: &str, value: &T) -> Result<()> {
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

    pub fn finish(mut self) -> Result<()> {
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
    ) -> Result<()> {
        let mut context = ChunkedSerialization::new(&mut self.chunked_output, 0);
        context.output_mut().write_var_u32(constructor_idx);
        serialize_case(&mut context)
    }
}

pub trait ChunkedInput {
    type Input: BinaryInput;

    fn stored_version(&self) -> u8;

    fn input_for(&mut self, version: u8) -> Result<&mut Self::Input>;
    fn state(&self) -> &State;
    fn state_mut(&mut self) -> &mut State;

    fn made_optional_at(&self) -> &HashMap<FieldPosition, u8>;

    fn removed_fields(&self) -> &HashSet<String>;
}

pub struct NonChunkedInput<'a, Context: DeserializationContext> {
    context: &'a mut Context,
    made_optional_at: HashMap<FieldPosition, u8>,
    removed_fields: HashSet<String>,
}

impl<'a, Context: DeserializationContext> NonChunkedInput<'a, Context> {
    pub fn new(context: &'a mut Context) -> Self {
        Self {
            context,
            made_optional_at: HashMap::new(),
            removed_fields: HashSet::new(),
        }
    }
}

impl<'a, Context: DeserializationContext> ChunkedInput for NonChunkedInput<'a, Context> {
    type Input = Context::Input;

    fn stored_version(&self) -> u8 {
        0
    }

    fn input_for(&mut self, version: u8) -> Result<&mut Self::Input> {
        if version == 0 {
            Ok(self.context.input_mut())
        } else {
            Err(Error::DeserializingNonExistingChunk(version))
        }
    }

    fn state(&self) -> &State {
        self.context.state()
    }

    fn state_mut(&mut self) -> &mut State {
        self.context.state_mut()
    }

    fn made_optional_at(&self) -> &HashMap<FieldPosition, u8> {
        &self.made_optional_at
    }

    fn removed_fields(&self) -> &HashSet<String> {
        &self.removed_fields
    }
}

pub struct PreloadedChunkedInput<'a, Context: DeserializationContext> {
    context: &'a mut Context,
    stored_version: u8,
    made_optional_at: HashMap<FieldPosition, u8>,
    removed_fields: HashSet<String>,
    inputs: Vec<Bytes>,
}

impl<'a, Context: DeserializationContext> PreloadedChunkedInput<'a, Context> {
    pub fn new(context: &'a mut Context, stored_version: u8) -> Result<Self> {
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
                    let input = context.input_mut().read_bytes(*size as usize)?;
                    inputs.push(input.into());
                }
                SerializedEvolutionStep::FieldMadeOptional { position } => {
                    made_optional_at.insert(*position, idx as u8);
                    inputs.push(Bytes::new());
                }
                SerializedEvolutionStep::FieldRemoved { field_name } => {
                    removed_fields.insert(field_name.clone());
                    inputs.push(Bytes::new());
                }
                _ => {
                    inputs.push(Bytes::new());
                }
            }
        }

        Ok(Self {
            context,
            stored_version,
            made_optional_at,
            removed_fields,
            inputs,
        })
    }
}

impl<'a, Context: DeserializationContext> ChunkedInput for PreloadedChunkedInput<'a, Context> {
    type Input = Bytes;

    fn stored_version(&self) -> u8 {
        self.stored_version
    }

    fn input_for(&mut self, version: u8) -> Result<&mut Self::Input> {
        if version < self.inputs.len() as u8 {
            Ok(&mut self.inputs[version as usize])
        } else {
            Err(Error::DeserializingNonExistingChunk(version))
        }
    }

    fn state(&self) -> &State {
        self.context.state()
    }

    fn state_mut(&mut self) -> &mut State {
        self.context.state_mut()
    }

    fn made_optional_at(&self) -> &HashMap<FieldPosition, u8> {
        &self.made_optional_at
    }

    fn removed_fields(&self) -> &HashSet<String> {
        &self.removed_fields
    }
}

pub struct ChunkedDeserialization<'a, CI: ChunkedInput> {
    input: &'a mut CI,
    chunk: u8,
}

impl<'a, CI: ChunkedInput> ChunkedDeserialization<'a, CI> {
    pub fn new(input: &'a mut CI, chunk: u8) -> Self {
        Self { input, chunk }
    }
}

impl<'a, CI: ChunkedInput> DeserializationContext for ChunkedDeserialization<'a, CI> {
    type Input = CI::Input;

    fn input_mut(&mut self) -> &mut Self::Input {
        self.input.input_for(self.chunk).unwrap()
    }

    fn state(&self) -> &State {
        self.input.state()
    }

    fn state_mut(&mut self) -> &mut State {
        self.input.state_mut()
    }
}

pub struct AdtDeserializer<'a, Context: DeserializationContext, CI: ChunkedInput> {
    metadata: &'a AdtMetadata,
    chunked_input: CI,
    context: PhantomData<Context>,
    last_index_per_chunk: Vec<u8>,
    read_constructor_idx: Option<u32>,
}

impl<'a, 'b, Context: DeserializationContext>
    AdtDeserializer<'a, Context, NonChunkedInput<'b, Context>>
{
    pub fn new_v0(metadata: &'a AdtMetadata, context: &'b mut Context) -> Result<Self> {
        let chunked_input = NonChunkedInput::new(context);
        Ok(Self {
            metadata,
            chunked_input,
            context: PhantomData,
            last_index_per_chunk: vec![0u8; metadata.version as usize + 1],
            read_constructor_idx: None,
        })
    }
}

impl<'a, 'b, Context: DeserializationContext>
    AdtDeserializer<'a, Context, PreloadedChunkedInput<'b, Context>>
{
    pub fn new(
        metadata: &'a AdtMetadata,
        context: &'b mut Context,
        stored_version: u8,
    ) -> Result<Self> {
        let chunked_input = PreloadedChunkedInput::new(context, stored_version)?;
        Ok(Self {
            metadata,
            chunked_input,
            context: PhantomData,
            last_index_per_chunk: vec![0u8; metadata.version as usize + 1],
            read_constructor_idx: None,
        })
    }
}

impl<'a, Context: DeserializationContext, CI: ChunkedInput> AdtDeserializer<'a, Context, CI> {
    pub fn read_field<T: BinaryDeserializer>(
        &mut self,
        field_name: &str,
        field_default: Option<T>,
    ) -> Result<T> {
        if self.chunked_input.removed_fields().contains(field_name) {
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
            if self.chunked_input.stored_version() < chunk {
                // Field was not serialized
                match field_default {
                    Some(value) => Ok(value),
                    None => Err(Error::FieldWithoutDefaultValueIsMissing(
                        field_name.to_string(),
                    )),
                }
            } else {
                // Field was serialized

                if self
                    .chunked_input
                    .made_optional_at()
                    .contains_key(&field_position)
                {
                    // The field was made optional in a newer version, so we have to read Option<T>

                    let mut context = ChunkedDeserialization::new(&mut self.chunked_input, chunk);
                    let is_defined = bool::deserialize(&mut context)?;
                    if is_defined {
                        T::deserialize(&mut context)
                    } else {
                        Err(Error::NonOptionalFieldSerializedAsNone(
                            field_name.to_string(),
                        ))
                    }
                } else {
                    let mut context = ChunkedDeserialization::new(&mut self.chunked_input, chunk);

                    T::deserialize(&mut context)
                }
            }
        }
    }

    pub fn read_optional_field<T: BinaryDeserializer>(
        &mut self,
        field_name: &str,
        field_default: Option<Option<T>>,
    ) -> Result<Option<T>> {
        if self.chunked_input.removed_fields().contains(field_name) {
            Ok(None)
        } else {
            let chunk = *self
                .metadata
                .field_generations
                .get(field_name)
                .unwrap_or(&0);
            let opt_since = *self.metadata.made_optional_at.get(field_name).unwrap_or(&0);

            self.record_field_index(chunk);
            if self.chunked_input.stored_version() < chunk {
                // This field was not serialized
                match field_default {
                    Some(default_value) => Ok(default_value),
                    None => Err(Error::DeserializationFailure(format!(
                        "Field {field_name} is not in the stream and does not have a default value"
                    ))),
                }
            } else {
                // This field was serialized

                if self.chunked_input.stored_version() < opt_since {
                    let mut context = ChunkedDeserialization::new(&mut self.chunked_input, chunk);
                    Ok(Some(T::deserialize(&mut context)?))
                } else {
                    let mut context = ChunkedDeserialization::new(&mut self.chunked_input, chunk);
                    Option::<T>::deserialize(&mut context)
                }
            }
        }
    }

    pub fn read_constructor<T>(
        &mut self,
        case_idx: u32,
        deserialize_case: impl FnOnce(&mut ChunkedDeserialization<CI>) -> Result<T>,
    ) -> Result<Option<T>> {
        let constructor_idx = self.read_or_get_constructor_idx()?;
        if constructor_idx == case_idx {
            let mut context = ChunkedDeserialization::new(&mut self.chunked_input, 0);
            Ok(Some(deserialize_case(&mut context)?))
        } else {
            Ok(None)
        }
    }

    fn record_field_index(&mut self, chunk: u8) -> FieldPosition {
        let last_index = &mut self.last_index_per_chunk[chunk as usize];
        let new_index = *last_index + 1;
        let fp = FieldPosition::new(chunk, new_index);
        *last_index = new_index;
        fp
    }

    fn read_or_get_constructor_idx(&mut self) -> Result<u32> {
        match self.read_constructor_idx {
            Some(idx) => Ok(idx),
            None => {
                let input = self.chunked_input.input_for(0)?;
                let constructor_idx = input.read_var_u32()?;
                self.read_constructor_idx = Some(constructor_idx);
                Ok(constructor_idx)
            }
        }
    }
}
