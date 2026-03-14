use crate::room::Room;

// Registry to manage rooms
struct RoomRegistry {
    rooms: Vec<Room>,
}

impl RoomRegistry {
    fn new() -> Self {
        RoomRegistry { rooms: Vec::new() }
    }

    fn create_room(&mut self, id: String, creator: String) -> &Room {
        let room = Room::new(id, creator);
        self.rooms.push(room);
        self.rooms.last().unwrap()
    }

    fn get_room(&self, id: &str) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id == id)
    }
}