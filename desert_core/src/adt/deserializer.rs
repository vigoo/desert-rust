use crate::adt::{AdtMetadata, FieldPosition};
use crate::evolution::SerializedEvolutionStep;
use crate::state::State;
use crate::{BinaryDeserializer, BinaryInput, DeserializationContext, Error, OwnedInput, Result};
use hashbrown::{HashMap, HashSet};
use std::marker::PhantomData;

pub trait ChunkedInput {
    type Input: BinaryInput;

    fn stored_version(&self) -> u8;

    fn input_for(&mut self, version: u8) -> crate::Result<&mut Self::Input>;
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

    fn input_for(&mut self, version: u8) -> crate::Result<&mut Self::Input> {
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
    inputs: Vec<OwnedInput>,
}

impl<'a, Context: DeserializationContext> PreloadedChunkedInput<'a, Context> {
    pub fn new(context: &'a mut Context, stored_version: u8) -> crate::Result<Self> {
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
                    inputs.push(OwnedInput::new(input.to_vec()));
                }
                SerializedEvolutionStep::FieldMadeOptional { position } => {
                    made_optional_at.insert(*position, idx as u8);
                    inputs.push(OwnedInput::new(vec![]));
                }
                SerializedEvolutionStep::FieldRemoved { field_name } => {
                    removed_fields.insert(field_name.clone());
                    inputs.push(OwnedInput::new(vec![]));
                }
                _ => {
                    inputs.push(OwnedInput::new(vec![]));
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
    type Input = OwnedInput;

    fn stored_version(&self) -> u8 {
        self.stored_version
    }

    fn input_for(&mut self, version: u8) -> crate::Result<&mut Self::Input> {
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
    pub fn new_v0(metadata: &'a AdtMetadata, context: &'b mut Context) -> crate::Result<Self> {
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
    ) -> crate::Result<Self> {
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
    ) -> crate::Result<T> {
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
    ) -> crate::Result<Option<T>> {
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
    ) -> crate::Result<Option<T>> {
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

    fn read_or_get_constructor_idx(&mut self) -> crate::Result<u32> {
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
