extern crate serde;
extern crate serde_json;

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use self::serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = ()>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = ()>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, visitor: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

pub mod chat {
    use std::str::FromStr;

    #[derive(Serialize, Deserialize)]
    pub struct Chat {
        pub text: String,
    }

    impl FromStr for Chat {
        type Err = ();

        fn from_str(text: &str) -> Result<Self, ()> {
            Ok(Chat { text: text.to_owned() })
        }
    }
}

pub mod status {
    use super::chat::Chat;
    use super::string_or_struct;

    #[derive(Serialize, Deserialize)]
    pub struct Status {
        pub version: Version,

        #[serde(deserialize_with = "string_or_struct")]
        pub description: Chat,

        pub players: Players,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Version {
        pub name: String,
        pub protocol: u32,
    }

    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    pub struct Player {
        pub name: String,
        pub id: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Players {
        pub max: u32,
        pub online: u32,
        pub sample: Option<Vec<Player>>,
    }
}
