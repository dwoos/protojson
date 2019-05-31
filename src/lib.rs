use failure::Error;
use serde::de::Deserialize;
use serde_json::Value as JValue;
use serde_protobuf::de::Deserializer;
use serde_protobuf::descriptor::Descriptors;
use std::fs;
use std::io::BufRead;

mod proto;

pub struct ProtobufToJson {
    descriptors: Descriptors,
}

impl ProtobufToJson {
    pub fn new(descriptors_path: &str) -> Result<Self, Error> {
        let mut file = fs::File::open(descriptors_path)?;
        let proto = protobuf::parse_from_reader(&mut file)?;
        let descriptors = Descriptors::from_proto(&proto);
        Ok(Self { descriptors })
    }

    pub fn to_json(&self, message_type: &str, input: &mut BufRead) -> Result<JValue, Error> {
        let input = protobuf::CodedInputStream::from_buffered_reader(input);
        let mut deserializer =
            Deserializer::for_named_message(&self.descriptors, message_type, input)?;
        let value = JValue::deserialize(&mut deserializer)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_simple_message() -> Result<(), failure::Error> {
        use protobuf::Message;
        use serde_json::json;
        use serde_json::Value::Null;
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

        let p_to_j = crate::ProtobufToJson::new(descriptors_path)?;

        let value = p_to_j.to_json(message, &mut input)?;
        assert_eq!(
            value,
            json!({ "email": "email@example.com", "id":  Null, "name": Null, "phones": [] })
        );
        Ok(())
    }
}
