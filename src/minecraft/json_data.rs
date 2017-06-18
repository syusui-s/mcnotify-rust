extern crate serde;
extern crate serde_json;

mod status {
    #[derive(Serialize, Deserialize)]
    struct Version {
        name: String,
        protocol: i32,
    }
 
    #[derive(Serialize, Deserialize)]
    struct Players {
        name: String,
        protocol: i32,
    }

    #[derive(Serialize, Deserialize)]
    struct Description {
        text: String
    }

    #[derive(Serialize, Deserialize)]
    struct Status {
        version: Version,
        players: Players,
        description: Description,
    }
}
