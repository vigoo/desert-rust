use crate::adt::FieldPosition;
use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer, DeduplicatedString};

// TODO: this does not have to be public because we cannot use it in the attribute - need to just document it
#[derive(Debug)]
pub enum Evolution {
    InitialVersion,

    /// Add a new field to a struct.
    ///
    /// New version can still read the old with the use of `default`. Old version can only read if the type is `Option<T>` and the value is `None`.
    FieldAdded {
        name: String,
    },

    /// Existing non-option field is made optional.
    ///
    /// New version can read old data by wrapping with `Some`. Old version can read new data if it is not `None`.
    FieldMadeOptional {
        name: String,
    },

    /// Field removed from a struct.
    ///
    /// New version can read old data by skipping the field. Old version can read new data only if it was `Option<T>`.
    FieldRemoved {
        name: String,
    },

    /// Field made transient.
    ///
    /// An alias for `FieldRemoved`.
    FieldMadeTransient {
        name: String,
    },
}

pub(crate) enum SerializedEvolutionStep {
    FieldAddedToNewChunk { size: i32 },
    FieldMadeOptional { position: FieldPosition },
    FieldRemoved { field_name: String },
    Unknown,
}

const UNKNOWN: i32 = 0;
const FIELD_MADE_OPTIONAL: i32 = -1;
const FIELD_REMOVED: i32 = -2;

impl BinarySerializer for SerializedEvolutionStep {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> crate::Result<()> {
        match self {
            SerializedEvolutionStep::FieldAddedToNewChunk { size } => {
                println!("field added to new chunk size: {size}");
                context.output_mut().write_var_i32(*size);
                Ok(())
            }
            SerializedEvolutionStep::FieldMadeOptional { position } => {
                println!("field made optional position: {position:?}");
                context.output_mut().write_var_i32(FIELD_MADE_OPTIONAL);
                position.serialize(context)
            }
            SerializedEvolutionStep::FieldRemoved { field_name } => {
                println!("field removed field_name: {field_name}");
                context.output_mut().write_var_i32(FIELD_REMOVED);
                DeduplicatedString(field_name.clone()).serialize(context)
            }
            SerializedEvolutionStep::Unknown => {
                println!("unknown");
                context.output_mut().write_var_i32(UNKNOWN);
                Ok(())
            }
        }
    }
}

impl BinaryDeserializer for SerializedEvolutionStep {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let code_or_size = context.input_mut().read_var_i32()?;
        if code_or_size == UNKNOWN {
            return Ok(SerializedEvolutionStep::Unknown);
        }
        if code_or_size == FIELD_MADE_OPTIONAL {
            let position = FieldPosition::deserialize(context)?;
            return Ok(SerializedEvolutionStep::FieldMadeOptional { position });
        }
        if code_or_size == FIELD_REMOVED {
            let field_name = DeduplicatedString::deserialize(context)?.0;
            return Ok(SerializedEvolutionStep::FieldRemoved { field_name });
        }
        Ok(SerializedEvolutionStep::FieldAddedToNewChunk { size: code_or_size })
    }
}
