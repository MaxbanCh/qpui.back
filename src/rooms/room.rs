// Websocket room for users to communicate
struct Room {
    id: String,
    clients: Vec<String>, // List of client IDs
    creator: String, // ID of the user who created the room
}

impl Room {
    fn new(id: String, creator: String) -> Self {
        Room {
            id,
            clients: Vec::new(),
            creator,
        }
    }
}