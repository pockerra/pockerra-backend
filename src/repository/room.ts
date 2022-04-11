import { Room, RoomName } from '../types';
import { roomModel } from '../model/room';

const addRoom = async (roomName: RoomName) => {
  await roomModel.create({ name: roomName, hidden: true });
};

const removeRoom = async (roomName: RoomName) => {
  await roomModel.deleteOne({ name: roomName });
};

const getRooms = async (): Promise<Array<Room>> => {
  return roomModel.find();
};

const getRoomByName = async (name: string): Promise<Room | null> => {
  return roomModel.findOne({ name }, {});
};

const revealCards = (roomName: RoomName) => {
  roomModel.updateOne({ name: roomName }, { hidden: false });
};

const startOver = (roomName: RoomName) => {
  roomModel.updateOne({ name: roomName }, { hidden: true });
};

const isRoomCreated = (name: RoomName) => {
  return roomModel.findOne({ name }, {});
};

export { addRoom, removeRoom, revealCards, isRoomCreated, getRooms, startOver, getRoomByName };
