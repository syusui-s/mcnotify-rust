extern crate serde;
extern crate serde_json;

pub mod status {
    #[derive(Serialize, Deserialize)]
    pub struct Version {
        name: String,
        protocol: i32,
    }
 
    #[derive(Serialize, Deserialize)]
    pub struct Players {
        name: String,
        protocol: i32,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Description {
        text: String
    }

    #[derive(Serialize, Deserialize)]
    pub struct Status {
        version: Version,
        players: Players,
        description: Description,
    }
}
