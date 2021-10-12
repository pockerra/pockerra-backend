import { Room, RoomName } from '../types';

let rooms: Array<Room> = [];

const addRoom = (roomName: RoomName) => {
  rooms.push({ name: roomName, hidden: true });
};

const removeRoom = (roomName: RoomName) => {
  rooms.filter((r) => r.name !== roomName);
};

const getRooms = (): Array<Room> => {
  return rooms;
};

const getRoomByName = (name: string): Room | undefined => {
  return rooms.find((room) => room.name === name);
};

const revealCards = (roomName: RoomName) => {
  rooms = rooms.map((r) => {
    if (r.name === roomName) {
      r.hidden = false;
    }

    return r;
  });
};

const startOver = (roomName: RoomName) => {
  rooms = rooms.map((r) => {
    if (r.name === roomName) {
      r.hidden = true;
    }

    return r;
  });
};

const isRoomCreated = (name: RoomName) => {
  return !!rooms.find((r) => r.name === name);
};

export { addRoom, removeRoom, revealCards, isRoomCreated, getRooms, startOver, getRoomByName };
