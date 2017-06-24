extern crate serde;
extern crate serde_json;

pub mod status {
    #[derive(Serialize, Deserialize)]
    pub struct Version {
        name: String,
        protocol: u32,
    }

    impl Version {
        pub fn get_name(&self) -> String {
            self.name.clone()
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Player {
        name: String,
        id: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Players {
        max: u32,
        online: u32,
        sample: Option<Vec<Player>>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Description {
        text: String
    }

    #[derive(Serialize, Deserialize)]
    pub struct Status {
        description: Description,
        players: Players,
        version: Version,
    }

    impl Status {
        pub fn get_version(&self) -> &Version {
            &self.version
        }

        pub fn get_players(&self) -> &Players {
            &self.players
        }

        pub fn get_description(&self) -> &Description {
            &self.description
        }
    }
}
