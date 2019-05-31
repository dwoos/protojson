use failure::Error;
use serde::de::Deserialize;
use serde_json::Value as JValue;
use serde_protobuf::de::Deserializer;
use serde_protobuf::descriptor::Descriptors;
use std::fs;
use std::io::BufRead;

mod proto;

pub fn read_protobuf_message_as_json(
    descriptors_path: String,
    message: String,
    input: &mut BufRead,
) -> Result<JValue, Error> {
    let mut file = fs::File::open(descriptors_path)?;
    let proto = protobuf::parse_from_reader(&mut file)?;
    let descriptors = Descriptors::from_proto(&proto);
    let input = protobuf::CodedInputStream::from_buffered_reader(input);
    let mut deserializer = Deserializer::for_named_message(&descriptors, &message, input)?;
    let value = JValue::deserialize(&mut deserializer)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_simple_message() -> Result<(), failure::Error> {
        use protobuf::Message;
        use serde_json::json;
        use serde_json::Value::{Array, Null};
        use std::io::BufReader;
        let mut person = crate::proto::basic::Person::new();
        person.set_email("email@example.com".into());
        let mut person_bytes: Vec<u8> = Vec::new();
        {
            let mut stream = protobuf::CodedOutputStream::new(&mut person_bytes);
            person.write_to(&mut stream)?;
            stream.flush()?;
        }
        let descriptors_path = "examples/basic.pb";
        let message = ".Person";
        let mut input = BufReader::new(&person_bytes[..]);
        let value = crate::read_protobuf_message_as_json(
            descriptors_path.into(),
            message.into(),
            &mut input,
        )?;
        assert_eq!(
            value,
            json!({ "email": "email@example.com", "id":  Null, "name": Null, "phones": [] })
        );
        Ok(())
    }
}
