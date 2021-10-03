import { Room, RoomName } from '../types';

let rooms: Array<Room> = [];

const addRoom = (roomName: RoomName) => {
  rooms.push({ name: roomName, hidden: true });
};

const removeRoom = (roomName: RoomName) => {
  rooms.filter((r) => r.name !== roomName);
};

const revealCards = (roomName: RoomName) => {
  rooms = rooms.map((r) => {
    if (r.name === roomName) {
      r.hidden = false;
    }

    return r;
  });
};

export { addRoom, removeRoom, revealCards };
