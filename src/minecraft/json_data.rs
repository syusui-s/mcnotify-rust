extern crate serde;
extern crate serde_json;

pub mod status {
    #[derive(Serialize, Deserialize)]
    pub struct Status {
        pub version: Version,
        pub description: Description,
        pub players: Players,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Version {
        pub name: String,
        pub protocol: u32,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Description {
        pub text: String
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
